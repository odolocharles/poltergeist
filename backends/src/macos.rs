//! macOS backend. The pointer and keys go through CoreGraphics events; keep-awake uses
//! the system's own `caffeinate` tool. The C declarations are written out by hand, so
//! there is no binding crate to break.
//!
//! macOS only delivers synthetic events to apps that have the Accessibility
//! permission (System Settings, Privacy & Security, Accessibility).

use poltergeist_core::{Button, Error, InputBackend, PowerBackend, Result};
use std::{
    ffi::c_void,
    process::{Child, Command, Stdio},
};

#[repr(C)]
#[derive(Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct CGSize {
    width: f64,
    height: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

type CGEventRef = *mut c_void;

#[cfg_attr(target_os = "macos", link(name = "CoreGraphics", kind = "framework"))]
extern "C" {
    fn CGEventCreateMouseEvent(source: *const c_void, kind: u32, pos: CGPoint, button: u32) -> CGEventRef;
    fn CGEventCreateKeyboardEvent(source: *const c_void, key: u16, down: bool) -> CGEventRef;
    fn CGEventSetIntegerValueField(event: CGEventRef, field: u32, value: i64);
    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CGMainDisplayID() -> u32;
    fn CGDisplayBounds(display: u32) -> CGRect;
}
#[cfg_attr(target_os = "macos", link(name = "CoreFoundation", kind = "framework"))]
extern "C" {
    fn CFRelease(cf: *const c_void);
}
#[cfg_attr(target_os = "macos", link(name = "ApplicationServices", kind = "framework"))]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

const HID_TAP: u32 = 0; // kCGHIDEventTap: events enter at the hardware level
const MOUSE_MOVED: u32 = 5;
const CLICK_STATE_FIELD: u32 = 1; // kCGMouseEventClickState
const KEY_SHIFT: u16 = 0x38;

fn post(event: CGEventRef) -> Result<()> {
    if event.is_null() {
        return Err(Error::Backend("macOS could not create the input event".into()));
    }
    // SAFETY: `event` is a valid, owned CGEvent; it is released right after posting.
    unsafe {
        CGEventPost(HID_TAP, event);
        CFRelease(event as *const c_void);
    }
    Ok(())
}

/// Pointer and keys via CoreGraphics. Coordinates are normalized to the main display.
pub struct MacInput {
    pos: (f32, f32),
}

impl MacInput {
    pub fn new() -> Result<Self> {
        // SAFETY: plain query with no arguments.
        if !unsafe { AXIsProcessTrusted() } {
            // Take the user straight to the right settings page.
            let _ = Command::new("open")
                .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                .spawn();
            return Err(Error::Backend(
                "grant Poltergeist the Accessibility permission (System Settings, Privacy & Security), then press Start again"
                    .into(),
            ));
        }
        Ok(Self { pos: (0.5, 0.5) })
    }

    fn point(&self) -> CGPoint {
        // SAFETY: plain queries; bounds are in points, which is what events use.
        let b = unsafe { CGDisplayBounds(CGMainDisplayID()) };
        CGPoint {
            x: b.origin.x + self.pos.0.clamp(0.0, 1.0) as f64 * (b.size.width - 1.0).max(0.0),
            y: b.origin.y + self.pos.1.clamp(0.0, 1.0) as f64 * (b.size.height - 1.0).max(0.0),
        }
    }

    fn button_event(&self, button: Button, down: bool) -> Result<()> {
        let (kind, idx) = match (button, down) {
            (Button::Left, true) => (1, 0),
            (Button::Left, false) => (2, 0),
            (Button::Right, true) => (3, 1),
            (Button::Right, false) => (4, 1),
            (Button::Middle, true) => (25, 2),
            (Button::Middle, false) => (26, 2),
        };
        // SAFETY: null source is allowed; the event is checked and released in `post`.
        let ev = unsafe { CGEventCreateMouseEvent(std::ptr::null(), kind, self.point(), idx) };
        if !ev.is_null() {
            // SAFETY: `ev` is non-null and valid.
            unsafe { CGEventSetIntegerValueField(ev, CLICK_STATE_FIELD, 1) };
        }
        post(ev)
    }
}

impl InputBackend for MacInput {
    fn move_to(&mut self, x: f32, y: f32) -> Result<()> {
        self.pos = (x, y);
        // SAFETY: as in `button_event`.
        post(unsafe { CGEventCreateMouseEvent(std::ptr::null(), MOUSE_MOVED, self.point(), 0) })
    }

    fn press(&mut self, button: Button) -> Result<()> {
        self.button_event(button, true)
    }

    fn release(&mut self, button: Button) -> Result<()> {
        self.button_event(button, false)
    }

    fn tap_idle_key(&mut self) -> Result<()> {
        for down in [true, false] {
            // SAFETY: as above.
            post(unsafe { CGEventCreateKeyboardEvent(std::ptr::null(), KEY_SHIFT, down) })?;
        }
        Ok(())
    }
}

/// Keep-awake by running the system's `caffeinate` tool for as long as it is wanted.
#[derive(Default)]
pub struct MacPower {
    child: Option<Child>,
}

impl PowerBackend for MacPower {
    fn inhibit(&mut self) -> Result<()> {
        if self.child.is_some() {
            return Ok(());
        }
        let child = Command::new("caffeinate")
            .args(["-d", "-i"]) // no display sleep, no idle sleep
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| Error::Backend(format!("caffeinate not available: {e}")))?;
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

impl Drop for MacPower {
    fn drop(&mut self) {
        let _ = self.release();
    }
}
