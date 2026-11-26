use crate::{backend::Button, paths, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    KeepAwake,
    Cursor,
    Both,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MovementStyle {
    Random,
    Jitter,
    Circle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ClickConfig {
    pub enabled: bool,
    pub button: Button,
    /// Target position, normalized 0..1.
    pub x: f32,
    pub y: f32,
    pub min_interval_s: f64,
    pub max_interval_s: f64,
    /// How long the button is held down, like a real finger/mouse press.
    pub hold_ms_min: u64,
    pub hold_ms_max: u64,
    /// Random offset around the target, as a fraction of the screen (0 = exact).
    pub scatter: f32,
    /// Stop clicking after this many clicks (0 = unlimited).
    pub max_clicks: u32,
}

impl Default for ClickConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            button: Button::Left,
            x: 0.5,
            y: 0.5,
            min_interval_s: 4.0,
            max_interval_s: 8.0,
            hold_ms_min: 40,
            hold_ms_max: 110,
            scatter: 0.0,
            max_clicks: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub mode: Mode,
    pub style: MovementStyle,
    pub min_interval_s: f64,
    pub max_interval_s: f64,
    /// Movement range as a fraction of the screen (0.12 = about 12%).
    pub radius: f32,
    /// Curved paths with easing, like a human hand.
    pub humanize: bool,
    pub idle_key_chance: f64,
    pub click: ClickConfig,
    /// Show the see-through pointer/click overlay on screen (GUI only).
    pub show_overlay: bool,
    /// Overrides the default Poltergeist/ output folder.
    pub output_root: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Both,
            style: MovementStyle::Random,
            min_interval_s: 3.0,
            max_interval_s: 12.0,
            radius: 0.12,
            humanize: true,
            idle_key_chance: 0.3,
            click: ClickConfig::default(),
            show_overlay: false,
            output_root: None,
        }
    }
}

/// The config shared between the UI and the running engine, so edits apply live.
pub type SharedConfig = Arc<RwLock<Config>>;

impl Config {
    pub fn shared(self) -> SharedConfig {
        Arc::new(RwLock::new(self))
    }

    pub fn path() -> PathBuf {
        paths::config_dir().join("config.json")
    }

    /// Loads the config, writing defaults on first run. Works fully offline.
    pub fn load_or_create() -> Result<Self> {
        let path = Self::path();
        if path.exists() {
            let mut cfg: Config = serde_json::from_str(&fs::read_to_string(&path)?)?;
            cfg.sanitize();
            Ok(cfg)
        } else {
            let cfg = Config::default();
            cfg.save()?;
            Ok(cfg)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        // Atomic write: temp file then rename, so a crash never corrupts the config.
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        fs::rename(tmp, path)?;
        Ok(())
    }

    pub fn sanitize(&mut self) {
        self.min_interval_s = self.min_interval_s.max(0.05);
        self.max_interval_s = self.max_interval_s.max(self.min_interval_s);
        self.radius = self.radius.clamp(0.005, 0.5);
        self.idle_key_chance = self.idle_key_chance.clamp(0.0, 1.0);
        let c = &mut self.click;
        c.x = c.x.clamp(0.0, 1.0);
        c.y = c.y.clamp(0.0, 1.0);
        c.min_interval_s = c.min_interval_s.max(0.05);
        c.max_interval_s = c.max_interval_s.max(c.min_interval_s);
        c.hold_ms_max = c.hold_ms_max.max(c.hold_ms_min);
        c.scatter = c.scatter.clamp(0.0, 0.2);
    }
}
