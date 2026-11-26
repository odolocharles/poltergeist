use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent, Key, RelativeAxisType,
    UinputAbsSetup,
};
use poltergeist_core::{Button, Error, InputBackend, PowerBackend, Result};
use std::process::{Child, Command, Stdio};

const ABS_MAX: i32 = 32767;

fn be<E: std::fmt::Display>(e: E) -> Error {
    Error::Backend(e.to_string())
}

/// A virtual absolute pointer (like a USB tablet, the same kind virtual machines
/// use) created through /dev/uinput. Events enter the kernel input stack exactly
/// like hardware events, so X11, Wayland compositors and the console all see a
/// normal pointer with real press/release events. No network is involved.
/// Needs access to /dev/uinput (see packaging/linux/setup-uinput.sh).
pub struct UinputInput {
    dev: VirtualDevice,
}

impl UinputInput {
    pub fn new() -> Result<Self> {
        let mut keys = AttributeSet::<Key>::new();
        keys.insert(Key::BTN_LEFT);
        keys.insert(Key::BTN_RIGHT);
        keys.insert(Key::BTN_MIDDLE);
        keys.insert(Key::KEY_LEFTSHIFT);
        let mut rel = AttributeSet::<RelativeAxisType>::new();
        rel.insert(RelativeAxisType::REL_WHEEL); // helps some stacks classify it as a mouse-like device
        let x = UinputAbsSetup::new(AbsoluteAxisType::ABS_X, AbsInfo::new(0, 0, ABS_MAX, 0, 0, 0));
        let y = UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, AbsInfo::new(0, 0, ABS_MAX, 0, 0, 0));

        let dev = VirtualDeviceBuilder::new()
            .map_err(|e| {
                Error::Backend(format!(
                    "cannot open /dev/uinput ({e}). Run packaging/linux/setup-uinput.sh or use root"
                ))
            })?
            .name("Poltergeist Virtual Pointer")
            .with_keys(&keys)
            .map_err(be)?
            .with_relative_axes(&rel)
            .map_err(be)?
            .with_absolute_axis(&x)
            .map_err(be)?
            .with_absolute_axis(&y)
            .map_err(be)?
            .build()
            .map_err(be)?;
        std::thread::sleep(std::time::Duration::from_millis(500)); // let the system register it
        Ok(Self { dev })
    }

    fn key(&mut self, key: Key, value: i32) -> Result<()> {
        self.dev
            .emit(&[InputEvent::new(EventType::KEY, key.code(), value)])
            .map_err(be)
    }
}

fn btn(b: Button) -> Key {
    match b {
        Button::Left => Key::BTN_LEFT,
        Button::Right => Key::BTN_RIGHT,
        Button::Middle => Key::BTN_MIDDLE,
    }
}

impl InputBackend for UinputInput {
    fn move_to(&mut self, x: f32, y: f32) -> Result<()> {
        let ax = (x.clamp(0.0, 1.0) * ABS_MAX as f32) as i32;
        let ay = (y.clamp(0.0, 1.0) * ABS_MAX as f32) as i32;
        self.dev
            .emit(&[
                InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_X.0, ax),
                InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_Y.0, ay),
            ])
            .map_err(be)
    }

    fn press(&mut self, button: Button) -> Result<()> {
        self.key(btn(button), 1)
    }

    fn release(&mut self, button: Button) -> Result<()> {
        self.key(btn(button), 0)
    }

    fn tap_idle_key(&mut self) -> Result<()> {
        self.key(Key::KEY_LEFTSHIFT, 1)?;
        self.key(Key::KEY_LEFTSHIFT, 0)
    }
}

/// Holds a systemd-logind inhibitor lock by keeping `systemd-inhibit` running.
/// Fully local. A D-Bus implementation can replace this later.
#[derive(Default)]
pub struct SystemdInhibit {
    child: Option<Child>,
}

impl PowerBackend for SystemdInhibit {
    fn inhibit(&mut self) -> Result<()> {
        if self.child.is_some() {
            return Ok(());
        }
        let child = Command::new("systemd-inhibit")
            .args([
                "--what=idle:sleep",
                "--who=Poltergeist",
                "--why=Keep the machine awake",
                "--mode=block",
                "sleep",
                "infinity",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| Error::Backend(format!("systemd-inhibit not available: {e}")))?;
        self.child = Some(child);
        Ok(())
    }

    fn release(&mut self) -> Result<()> {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        Ok(())
    }
}

impl Drop for SystemdInhibit {
    fn drop(&mut self) {
        let _ = self.release();
    }
}
