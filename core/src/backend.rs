use crate::Result;

#[derive(Debug, Clone, Copy)]
pub enum Button {
    Left,
    Right,
    Middle,
}

/// Moves the pointer and clicks. One implementation per platform.
pub trait InputBackend {
    fn move_relative(&mut self, dx: i32, dy: i32) -> Result<()>;
    fn click(&mut self, button: Button) -> Result<()>;
    /// Harmless key tap that resets idle timers ignoring the mouse.
    fn tap_idle_key(&mut self) -> Result<()>;
}

/// Prevents sleep / screen blanking while held.
pub trait PowerBackend {
    fn inhibit(&mut self) -> Result<()>;
    fn release(&mut self) -> Result<()>;
}
