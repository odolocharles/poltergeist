use poltergeist_core::{
    state::new_state, Button, Config, Engine, InputBackend, Mode, PowerBackend, Result,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::{thread, time::Duration};

#[derive(Debug, Clone, PartialEq)]
enum Ev {
    Move(f32, f32),
    Press,
    Release,
}

struct Rec(Arc<Mutex<Vec<Ev>>>);
impl InputBackend for Rec {
    fn move_to(&mut self, x: f32, y: f32) -> Result<()> {
        self.0.lock().unwrap().push(Ev::Move(x, y));
        Ok(())
    }
    fn press(&mut self, _: Button) -> Result<()> {
        self.0.lock().unwrap().push(Ev::Press);
        Ok(())
    }
    fn release(&mut self, _: Button) -> Result<()> {
        self.0.lock().unwrap().push(Ev::Release);
        Ok(())
    }
    fn tap_idle_key(&mut self) -> Result<()> {
        Ok(())
    }
}
struct NoPower;
impl PowerBackend for NoPower {
    fn inhibit(&mut self) -> Result<()> { Ok(()) }
    fn release(&mut self) -> Result<()> { Ok(()) }
}

#[test]
fn click_arrives_at_target_then_presses_and_releases_in_order() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut cfg = Config::default();
    cfg.mode = Mode::Cursor;
    cfg.min_interval_s = 100.0; // no random wandering during the test
    cfg.max_interval_s = 100.0;
    cfg.click.enabled = true;
    cfg.click.x = 0.25;
    cfg.click.y = 0.75;
    cfg.click.min_interval_s = 0.1;
    cfg.click.max_interval_s = 0.1;
    cfg.click.max_clicks = 3;
    let shared_cfg = cfg.shared();
    let state = new_state();
    let stop = Arc::new(AtomicBool::new(false));

    let (c2, s2, st2) = (shared_cfg.clone(), state.clone(), stop.clone());
    let ev2 = events.clone();
    let h = thread::spawn(move || {
        Engine::new(c2, Box::new(Rec(ev2)), Box::new(NoPower), st2, s2).run().unwrap();
    });

    // Wait for 3 clicks (the engine disables auto-click itself afterwards).
    for _ in 0..100 {
        if state.lock().unwrap().click_count >= 3 {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    stop.store(true, Ordering::Relaxed);
    h.join().unwrap();

    let ev = events.lock().unwrap().clone();
    assert_eq!(state.lock().unwrap().click_count, 3, "should click exactly max_clicks times");
    assert!(!shared_cfg.read().unwrap().click.enabled, "auto-click disables itself at the limit");

    // Every press must be followed by a release, and the last pointer position
    // before each press must be exactly on the target.
    let presses: Vec<usize> = ev.iter().enumerate().filter(|(_, e)| **e == Ev::Press).map(|(i, _)| i).collect();
    assert_eq!(presses.len(), 3);
    for i in presses {
        assert_eq!(ev[i + 1], Ev::Release, "press must be followed by release");
        let last = ev[..i].iter().rev().find_map(|e| if let Ev::Move(x, y) = e { Some((*x, *y)) } else { None });
        let (x, y) = last.expect("pointer must have moved before the first press");
        assert!((x - 0.25).abs() < 1e-4 && (y - 0.75).abs() < 1e-4, "pointer must be on target before pressing, was {x},{y}");
    }
    // All positions stay on screen.
    for e in &ev {
        if let Ev::Move(x, y) = e {
            assert!((0.0..=1.0).contains(x) && (0.0..=1.0).contains(y));
        }
    }
}
