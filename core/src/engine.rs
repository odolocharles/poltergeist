use crate::{
    state::ClickEvent, Config, InputBackend, Mode, MovementStyle, PowerBackend, Result, SharedConfig,
    SharedState,
};
use rand::{rngs::ThreadRng, Rng};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

const STEP_MS: u64 = 8;

pub struct Engine {
    cfg: SharedConfig,
    input: Box<dyn InputBackend + Send>,
    power: Box<dyn PowerBackend + Send>,
    stop: Arc<AtomicBool>,
    state: SharedState,
    verbose: bool,
    awake: bool,
    pos: (f32, f32),
    angle: f32,
    clicks_done: u32,
}

impl Engine {
    pub fn new(
        cfg: SharedConfig,
        input: Box<dyn InputBackend + Send>,
        power: Box<dyn PowerBackend + Send>,
        stop: Arc<AtomicBool>,
        state: SharedState,
    ) -> Self {
        Self {
            cfg,
            input,
            power,
            stop,
            state,
            verbose: false,
            awake: false,
            pos: (0.5, 0.5),
            angle: 0.0,
            clicks_done: 0,
        }
    }

    /// Also print log lines to stderr (used by the CLI).
    pub fn verbose(mut self, on: bool) -> Self {
        self.verbose = on;
        self
    }

    /// Runs until the stop flag is set. Config edits made while running apply live.
    pub fn run(&mut self) -> Result<()> {
        {
            let mut s = self.lock();
            s.running = true;
            s.started_at = Some(Instant::now());
            s.error = None;
        }
        self.log("engine started");
        let result = self.run_loop();
        if self.awake {
            let _ = self.power.release();
            self.awake = false;
        }
        self.lock().running = false;
        self.log("engine stopped");
        result
    }

    fn snapshot_cfg(&self) -> Config {
        self.cfg.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, crate::Shared> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn log(&self, msg: &str) {
        if self.verbose {
            eprintln!("poltergeist: {msg}");
        }
        self.lock().push_log(msg.to_string());
    }

    fn stopped(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    fn run_loop(&mut self) -> Result<()> {
        let mut rng = rand::thread_rng();
        let mut next_move = Instant::now();
        let mut next_click = Instant::now();
        let mut was_clicking = false;
        let mut positioned = false;

        while !self.stopped() {
            let cfg = self.snapshot_cfg();
            self.sync_awake(&cfg);

            let moves = matches!(cfg.mode, Mode::Cursor | Mode::Both);
            let clicks = cfg.click.enabled;

            if (moves || clicks) && !positioned {
                self.input.move_to(0.5, 0.5)?;
                self.set_pos((0.5, 0.5));
                positioned = true;
            }

            if clicks && !was_clicking {
                self.clicks_done = 0;
                next_click = Instant::now() + secs(&mut rng, cfg.click.min_interval_s, cfg.click.max_interval_s);
                self.log("auto-click armed");
            }
            was_clicking = clicks;

            if moves && Instant::now() >= next_move {
                self.do_move(&cfg, &mut rng)?;
                next_move = Instant::now() + secs(&mut rng, cfg.min_interval_s, cfg.max_interval_s);
            }

            if clicks && Instant::now() >= next_click {
                self.do_click(&cfg, &mut rng)?;
                next_click = Instant::now() + secs(&mut rng, cfg.click.min_interval_s, cfg.click.max_interval_s);
                if cfg.click.max_clicks > 0 && self.clicks_done >= cfg.click.max_clicks {
                    self.cfg.write().unwrap_or_else(|e| e.into_inner()).click.enabled = false;
                    self.log("max clicks reached, auto-click disabled");
                }
            }

            // Idle (keep-awake only) ticks slowly to save battery.
            thread::sleep(Duration::from_millis(if moves || clicks { 20 } else { 250 }));
        }
        Ok(())
    }

    /// Takes or drops the keep-awake lock to match the current mode.
    fn sync_awake(&mut self, cfg: &Config) {
        let want = matches!(cfg.mode, Mode::KeepAwake | Mode::Both);
        if want && !self.awake {
            match self.power.inhibit() {
                Ok(()) => {
                    self.awake = true;
                    self.log("keep-awake ON");
                }
                Err(e) => {
                    // Avoid retrying and logging every tick.
                    self.awake = true;
                    self.log(&format!("keep-awake unavailable: {e}"));
                }
            }
        } else if !want && self.awake {
            let _ = self.power.release();
            self.awake = false;
            self.log("keep-awake OFF");
        }
    }

    fn set_pos(&mut self, p: (f32, f32)) {
        self.pos = p;
        let mut s = self.lock();
        s.pos = p;
    }

    fn do_move(&mut self, cfg: &Config, rng: &mut ThreadRng) -> Result<()> {
        let r = cfg.radius;
        let target = match cfg.style {
            MovementStyle::Random => (
                self.pos.0 + rng.gen_range(-r..=r),
                self.pos.1 + rng.gen_range(-r..=r),
            ),
            MovementStyle::Jitter => (
                self.pos.0 + rng.gen_range(-0.004..=0.004),
                self.pos.1 + rng.gen_range(-0.004..=0.004),
            ),
            MovementStyle::Circle => {
                self.angle += 0.7;
                (0.5 + r * self.angle.cos(), 0.5 + r * self.angle.sin())
            }
        };
        let target = (target.0.clamp(0.02, 0.98), target.1.clamp(0.02, 0.98));
        self.travel(target, cfg.humanize, rng)?;
        self.lock().move_count += 1;
        if rng.gen_bool(cfg.idle_key_chance) {
            self.input.tap_idle_key()?;
        }
        Ok(())
    }

    fn do_click(&mut self, cfg: &Config, rng: &mut ThreadRng) -> Result<()> {
        let c = &cfg.click;
        let mut t = (c.x, c.y);
        if c.scatter > 0.0 {
            t.0 += rng.gen_range(-c.scatter..=c.scatter);
            t.1 += rng.gen_range(-c.scatter..=c.scatter);
        }
        let t = (t.0.clamp(0.0, 1.0), t.1.clamp(0.0, 1.0));

        self.travel(t, cfg.humanize, rng)?;
        // Real hands pause briefly after arriving, before pressing.
        self.nap(rng.gen_range(50..=160));

        self.input.press(c.button)?;
        self.lock().pressed = true;
        // Always release, even if stop was requested mid-press.
        thread::sleep(Duration::from_millis(rng.gen_range(c.hold_ms_min..=c.hold_ms_max)));
        let released = self.input.release(c.button);
        {
            let mut s = self.lock();
            s.pressed = false;
            s.click_count += 1;
            s.recent_clicks.push(ClickEvent { x: t.0, y: t.1, at: Instant::now() });
            if s.recent_clicks.len() > 24 {
                s.recent_clicks.remove(0);
            }
        }
        self.clicks_done += 1;
        self.log(&format!("click at {:.1}%, {:.1}%", t.0 * 100.0, t.1 * 100.0));
        released
    }

    /// Glides the pointer to `to`. With `humanize`, the path bows slightly and
    /// eases in and out instead of travelling in a perfectly straight line.
    fn travel(&mut self, to: (f32, f32), humanize: bool, rng: &mut ThreadRng) -> Result<()> {
        let from = self.pos;
        let (dx, dy) = (to.0 - from.0, to.1 - from.1);
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 0.0005 {
            return Ok(());
        }
        let dur_ms = (150.0 + dist * 900.0).clamp(120.0, 1200.0);
        let steps = (dur_ms / STEP_MS as f32) as u32;
        let bow = if humanize { rng.gen_range(-0.25..0.25f32) * dist } else { 0.0 };
        let (nx, ny) = (-dy / dist, dx / dist);

        for i in 1..=steps {
            if self.stopped() {
                return Ok(());
            }
            let t = i as f32 / steps as f32;
            let e = if humanize { t * t * (3.0 - 2.0 * t) } else { t };
            let arc = 4.0 * e * (1.0 - e) * bow;
            let p = (
                (from.0 + dx * e + nx * arc).clamp(0.0, 1.0),
                (from.1 + dy * e + ny * arc).clamp(0.0, 1.0),
            );
            self.input.move_to(p.0, p.1)?;
            self.set_pos(p);
            thread::sleep(Duration::from_millis(STEP_MS));
        }
        // Land exactly on the target.
        self.input.move_to(to.0, to.1)?;
        self.set_pos(to);
        Ok(())
    }

    fn nap(&self, ms: u64) {
        thread::sleep(Duration::from_millis(ms));
    }
}

fn secs(rng: &mut ThreadRng, lo: f64, hi: f64) -> Duration {
    let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
    Duration::from_secs_f64(rng.gen_range(lo..=hi))
}
