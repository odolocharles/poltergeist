//! Per-platform backends. Each OS gets its own module behind `cfg`, so the
//! workspace compiles everywhere and a broken backend never affects another OS.

pub mod dry_run;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(windows)]
pub mod windows;
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
pub mod unsupported;

#[cfg(target_os = "linux")]
pub use linux::{SystemdInhibit as NativePower, UinputInput as NativeInput};
#[cfg(target_os = "macos")]
pub use macos::{MacInput as NativeInput, MacPower as NativePower};
#[cfg(windows)]
pub use windows::{WindowsInput as NativeInput, WindowsPower as NativePower};
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
pub use unsupported::{UnsupportedInput as NativeInput, UnsupportedPower as NativePower};
