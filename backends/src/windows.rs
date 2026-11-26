//! Windows backend. Pointer and keys go through `SendInput`; keep-awake uses
//! `SetThreadExecutionState`. The Win32 declarations are written out by hand so there is
//! no binding-crate version to break.
//!
//! Notes: Windows marks synthetic input as "injected", which hooks and some
//! security software can see, and a non-elevated process cannot send input to an
//! elevated (administrator) window.

use poltergeist_core::{Button, Error, InputBackend, PowerBackend, Result};

#[repr(C)]
#[derive(Clone, Copy)]
struct MouseInput {
    dx: i32,
    dy: i32,
    mouse_data: u32,
    flags: u32,
    time: u32,
    extra: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct KeybdInput {
    vk: u16,
    scan: u16,
    flags: u32,
    time: u32,
    extra: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
union InputUnion {
    mi: MouseInput,
    ki: KeybdInput,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Input {
    kind: u32,
    u: InputUnion,
}

#[link(name = "user32")]
extern "system" {
    fn SendInput(count: u32, inputs: *const Input, size: i32) -> u32;
}
#[link(name = "kernel32")]
extern "system" {
    fn SetThreadExecutionState(flags: u32) -> u32;
}

const INPUT_MOUSE: u32 = 0;
const INPUT_KEYBOARD: u32 = 1;
const MOUSEEVENTF_MOVE: u32 = 0x0001;
const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0008;
const MOUSEEVENTF_RIGHTUP: u32 = 0x0010;
const MOUSEEVENTF_MIDDLEDOWN: u32 = 0x0020;
const MOUSEEVENTF_MIDDLEUP: u32 = 0x0040;
const MOUSEEVENTF_ABSOLUTE: u32 = 0x8000;
const KEYEVENTF_KEYUP: u32 = 0x0002;
const VK_SHIFT: u16 = 0x10;
const ES_CONTINUOUS: u32 = 0x8000_0000;
const ES_SYSTEM_REQUIRED: u32 = 0x0000_0001;
const ES_DISPLAY_REQUIRED: u32 = 0x0000_0002;

fn mouse(flags: u32, dx: i32, dy: i32) -> Input {
    Input { kind: INPUT_MOUSE, u: InputUnion { mi: MouseInput { dx, dy, mouse_data: 0, flags, time: 0, extra: 0 } } }
}

fn key(vk: u16, up: bool) -> Input {
    let flags = if up { KEYEVENTF_KEYUP } else { 0 };
    Input { kind: INPUT_KEYBOARD, u: InputUnion { ki: KeybdInput { vk, scan: 0, flags, time: 0, extra: 0 } } }
}

fn send(inputs: &[Input]) -> Result<()> {
    // SAFETY: `inputs` is a valid slice of correctly laid out INPUT structs.
    let sent = unsafe { SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<Input>() as i32) };
    if sent as usize == inputs.len() {
        Ok(())
    } else {
        Err(Error::Backend(
            "Windows refused the input (the target window may be running as administrator)".into(),
        ))
    }
}

/// Moves the pointer and clicks with `SendInput`. Coordinates are normalized to the
/// primary monitor.
pub struct WindowsInput;

impl WindowsInput {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl InputBackend for WindowsInput {
    fn move_to(&mut self, x: f32, y: f32) -> Result<()> {
        let ax = (x.clamp(0.0, 1.0) * 65535.0) as i32;
        let ay = (y.clamp(0.0, 1.0) * 65535.0) as i32;
        send(&[mouse(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, ax, ay)])
    }

    fn press(&mut self, button: Button) -> Result<()> {
        let f = match button {
            Button::Left => MOUSEEVENTF_LEFTDOWN,
            Button::Right => MOUSEEVENTF_RIGHTDOWN,
            Button::Middle => MOUSEEVENTF_MIDDLEDOWN,
        };
        send(&[mouse(f, 0, 0)])
    }

    fn release(&mut self, button: Button) -> Result<()> {
        let f = match button {
            Button::Left => MOUSEEVENTF_LEFTUP,
            Button::Right => MOUSEEVENTF_RIGHTUP,
            Button::Middle => MOUSEEVENTF_MIDDLEUP,
        };
        send(&[mouse(f, 0, 0)])
    }

    fn tap_idle_key(&mut self) -> Result<()> {
        send(&[key(VK_SHIFT, false), key(VK_SHIFT, true)])
    }
}

/// Keep-awake. The execution state belongs to the calling thread, and the engine takes
/// and drops it on its own thread, so this works as written.
#[derive(Default)]
pub struct WindowsPower;

impl PowerBackend for WindowsPower {
    fn inhibit(&mut self) -> Result<()> {
        // SAFETY: plain Win32 call with flag constants.
        let prev = unsafe { SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED) };
        if prev == 0 {
            Err(Error::Backend("SetThreadExecutionState failed".into()))
        } else {
            Ok(())
        }
    }

    fn release(&mut self) -> Result<()> {
        // SAFETY: as above.
        unsafe { SetThreadExecutionState(ES_CONTINUOUS) };
        Ok(())
    }
}
