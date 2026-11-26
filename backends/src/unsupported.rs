use poltergeist_core::{Button, Error, InputBackend, PowerBackend, Result};

fn nope<T>() -> Result<T> {
    Err(Error::Backend("this platform's backend is not implemented yet (use simulation)".into()))
}

pub struct UnsupportedInput;
#[derive(Default)]
pub struct UnsupportedPower;

impl UnsupportedInput {
    pub fn new() -> Result<Self> {
        nope()
    }
}

impl InputBackend for UnsupportedInput {
    fn move_to(&mut self, _: f32, _: f32) -> Result<()> { nope() }
    fn press(&mut self, _: Button) -> Result<()> { nope() }
    fn release(&mut self, _: Button) -> Result<()> { nope() }
    fn tap_idle_key(&mut self) -> Result<()> { nope() }
}

impl PowerBackend for UnsupportedPower {
    fn inhibit(&mut self) -> Result<()> { nope() }
    fn release(&mut self) -> Result<()> { Ok(()) }
}
