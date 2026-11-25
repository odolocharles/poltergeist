use poltergeist_core::{Button, Error, InputBackend, PowerBackend, Result};

fn nope<T>() -> Result<T> {
    Err(Error::Backend("this platform's backend is not implemented yet (use --dry-run)".into()))
}

pub struct UnsupportedInput;
pub struct UnsupportedPower;

impl UnsupportedInput {
    pub fn new() -> Result<Self> {
        nope()
    }
}
impl Default for UnsupportedPower {
    fn default() -> Self {
        Self
    }
}

impl InputBackend for UnsupportedInput {
    fn move_relative(&mut self, _: i32, _: i32) -> Result<()> { nope() }
    fn click(&mut self, _: Button) -> Result<()> { nope() }
    fn tap_idle_key(&mut self) -> Result<()> { nope() }
}

impl PowerBackend for UnsupportedPower {
    fn inhibit(&mut self) -> Result<()> { nope() }
    fn release(&mut self) -> Result<()> { Ok(()) }
}
