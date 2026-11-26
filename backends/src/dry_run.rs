use poltergeist_core::{Button, InputBackend, PowerBackend, Result};

/// Simulation backend: performs nothing, optionally prints. Needs no permissions
/// and works on every OS, so the UI and CLI can always be tried safely.
#[derive(Default)]
pub struct DryRunInput {
    quiet: bool,
    moves: u64,
}
#[derive(Default)]
pub struct DryRunPower {
    quiet: bool,
}

impl DryRunInput {
    pub fn quiet() -> Self {
        Self { quiet: true, moves: 0 }
    }
}
impl DryRunPower {
    pub fn quiet() -> Self {
        Self { quiet: true }
    }
}

impl InputBackend for DryRunInput {
    fn move_to(&mut self, x: f32, y: f32) -> Result<()> {
        self.moves += 1;
        if !self.quiet && self.moves % 40 == 0 {
            println!("[dry-run] pointer at {:.1}%, {:.1}%", x * 100.0, y * 100.0);
        }
        Ok(())
    }
    fn press(&mut self, button: Button) -> Result<()> {
        if !self.quiet {
            println!("[dry-run] press {button:?}");
        }
        Ok(())
    }
    fn release(&mut self, button: Button) -> Result<()> {
        if !self.quiet {
            println!("[dry-run] release {button:?}");
        }
        Ok(())
    }
    fn tap_idle_key(&mut self) -> Result<()> {
        if !self.quiet {
            println!("[dry-run] idle key tap");
        }
        Ok(())
    }
}

impl PowerBackend for DryRunPower {
    fn inhibit(&mut self) -> Result<()> {
        if !self.quiet {
            println!("[dry-run] keep-awake ON");
        }
        Ok(())
    }
    fn release(&mut self) -> Result<()> {
        if !self.quiet {
            println!("[dry-run] keep-awake OFF");
        }
        Ok(())
    }
}
