use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Button {
    Left,
    Right,
    Middle,
}

/// Moves the pointer and presses buttons. One implementation per platform.
/// Positions are normalized: (0.0, 0.0) is the top-left of the screen and
/// (1.0, 1.0) the bottom-right, so the same config works at any resolution.
pub trait InputBackend {
    fn move_to(&mut self, x: f32, y: f32) -> Result<()>;
    fn press(&mut self, button: Button) -> Result<()>;
    fn release(&mut self, button: Button) -> Result<()>;
    /// Harmless key tap that resets idle timers ignoring the mouse.
    fn tap_idle_key(&mut self) -> Result<()>;
}

/// Prevents sleep / screen blanking while held.
pub trait PowerBackend {
    fn inhibit(&mut self) -> Result<()>;
    fn release(&mut self) -> Result<()>;
}
