use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, EventType, InputEvent, Key, RelativeAxisType,
};
use poltergeist_core::{Button, Error, InputBackend, PowerBackend, Result};
use std::process::{Child, Command, Stdio};

fn be<E: std::fmt::Display>(e: E) -> Error {
    Error::Backend(e.to_string())
}

/// Virtual mouse/keyboard via /dev/uinput. Works on X11, Wayland and the bare
/// console, with no network. Needs root or access to /dev/uinput.
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
        rel.insert(RelativeAxisType::REL_X);
        rel.insert(RelativeAxisType::REL_Y);

        let dev = VirtualDeviceBuilder::new()
            .map_err(|e| Error::Backend(format!("cannot open /dev/uinput (run as root, `modprobe uinput`): {e}")))?
            .name("poltergeist-virtual-input")
            .with_keys(&keys)
            .map_err(be)?
            .with_relative_axes(&rel)
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

impl InputBackend for UinputInput {
    fn move_relative(&mut self, dx: i32, dy: i32) -> Result<()> {
        self.dev
            .emit(&[
                InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_X.0, dx),
                InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_Y.0, dy),
            ])
            .map_err(be)
    }

    fn click(&mut self, button: Button) -> Result<()> {
        let key = match button {
            Button::Left => Key::BTN_LEFT,
            Button::Right => Key::BTN_RIGHT,
            Button::Middle => Key::BTN_MIDDLE,
        };
        self.key(key, 1)?;
        self.key(key, 0)
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
