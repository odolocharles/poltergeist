use crate::{Config, InputBackend, MovementStyle, Mode, PowerBackend, Result};
use rand::Rng;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

pub struct Engine<I: InputBackend, P: PowerBackend> {
    cfg: Config,
    input: I,
    power: P,
    stop: Arc<AtomicBool>,
}

impl<I: InputBackend, P: PowerBackend> Engine<I, P> {
    pub fn new(cfg: Config, input: I, power: P, stop: Arc<AtomicBool>) -> Self {
        Self { cfg, input, power, stop }
    }

    /// Runs until the stop flag is set. Sleeps in short slices so shutdown is instant.
    pub fn run(&mut self) -> Result<()> {
        if matches!(self.cfg.mode, Mode::KeepAwake | Mode::Both) {
            self.power.inhibit()?;
        }
        let result = self.run_loop();
        let _ = self.power.release();
        result
    }

    fn run_loop(&mut self) -> Result<()> {
        let mut rng = rand::thread_rng();
        let mut angle: f64 = 0.0;

        while !self.stop.load(Ordering::Relaxed) {
            if matches!(self.cfg.mode, Mode::Cursor | Mode::Both) {
                let (dx, dy) = match self.cfg.style {
                    MovementStyle::Random => {
                        let d = self.cfg.max_distance_px;
                        (rng.gen_range(-d..=d), rng.gen_range(-d..=d))
                    }
                    MovementStyle::Jitter => (rng.gen_range(-3..=3), rng.gen_range(-3..=3)),
                    MovementStyle::Circle => {
                        angle += 0.6;
                        let r = self.cfg.max_distance_px as f64 / 2.0;
                        ((r * angle.cos()) as i32, (r * angle.sin()) as i32)
                    }
                };
                self.glide(dx, dy, 20)?;
                if rng.gen_bool(self.cfg.idle_key_chance) {
                    self.input.tap_idle_key()?;
                }
            }
            let wait = rng.gen_range(self.cfg.min_interval_s..=self.cfg.max_interval_s);
            self.sleep_interruptible(wait);
        }
        Ok(())
    }

    /// Moves in small steps so the motion looks continuous. Uses cumulative
    /// interpolation so no pixels are lost to integer division (matters for jitter).
    fn glide(&mut self, dx: i32, dy: i32, steps: i32) -> Result<()> {
        for i in 1..=steps {
            if self.stop.load(Ordering::Relaxed) {
                break;
            }
            let (sx, sy) = (dx * i / steps - dx * (i - 1) / steps, dy * i / steps - dy * (i - 1) / steps);
            self.input.move_relative(sx, sy)?;
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }

    fn sleep_interruptible(&self, secs: f64) {
        let end = Instant::now() + Duration::from_secs_f64(secs);
        while Instant::now() < end && !self.stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(100));
        }
    }
}
