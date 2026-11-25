use poltergeist_core::{Button, InputBackend, PowerBackend, Result};

/// Prints what would happen. Needs no permissions, handy for testing anywhere.
pub struct DryRunInput;
pub struct DryRunPower;

impl InputBackend for DryRunInput {
    fn move_relative(&mut self, dx: i32, dy: i32) -> Result<()> {
        if dx != 0 || dy != 0 {
            println!("[dry-run] move {dx:+},{dy:+}");
        }
        Ok(())
    }
    fn click(&mut self, button: Button) -> Result<()> {
        println!("[dry-run] click {button:?}");
        Ok(())
    }
    fn tap_idle_key(&mut self) -> Result<()> {
        println!("[dry-run] idle key tap");
        Ok(())
    }
}

impl PowerBackend for DryRunPower {
    fn inhibit(&mut self) -> Result<()> {
        println!("[dry-run] keep-awake ON");
        Ok(())
    }
    fn release(&mut self) -> Result<()> {
        println!("[dry-run] keep-awake OFF");
        Ok(())
    }
}
