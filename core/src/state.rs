use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Instant,
};

#[derive(Debug, Clone, Copy)]
pub struct ClickEvent {
    pub x: f32,
    pub y: f32,
    pub at: Instant,
}

/// Live telemetry the engine publishes so a UI (or the CLI) can show what is happening.
#[derive(Default)]
pub struct Shared {
    pub running: bool,
    /// Normalized pointer position (0..1).
    pub pos: (f32, f32),
    pub pressed: bool,
    pub recent_clicks: Vec<ClickEvent>,
    pub click_count: u64,
    pub move_count: u64,
    pub started_at: Option<Instant>,
    pub log: VecDeque<String>,
    pub error: Option<String>,
}

impl Shared {
    pub fn push_log(&mut self, msg: impl Into<String>) {
        if self.log.len() >= 200 {
            self.log.pop_front();
        }
        self.log.push_back(msg.into());
    }
}

pub type SharedState = Arc<Mutex<Shared>>;

pub fn new_state() -> SharedState {
    Arc::new(Mutex::new(Shared { pos: (0.5, 0.5), ..Shared::default() }))
}
