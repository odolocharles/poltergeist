//! Per-platform backends. Each OS gets its own module behind `cfg`, so the
//! workspace compiles everywhere and a broken backend never affects another OS.

pub mod dry_run;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(not(target_os = "linux"))]
pub mod unsupported;

#[cfg(target_os = "linux")]
pub use linux::{SystemdInhibit as NativePower, UinputInput as NativeInput};
#[cfg(not(target_os = "linux"))]
pub use unsupported::{UnsupportedInput as NativeInput, UnsupportedPower as NativePower};
