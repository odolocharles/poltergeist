use crate::{paths, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub mode: Mode,
    pub style: MovementStyle,
    pub min_interval_s: f64,
    pub max_interval_s: f64,
    pub max_distance_px: i32,
    pub idle_key_chance: f64,
    /// Overrides the default Poltergeist/ output folder.
    pub output_root: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Both,
            style: MovementStyle::Random,
            min_interval_s: 5.0,
            max_interval_s: 30.0,
            max_distance_px: 200,
            idle_key_chance: 0.3,
            output_root: None,
        }
    }
}

impl Config {
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

    fn sanitize(&mut self) {
        self.min_interval_s = self.min_interval_s.max(0.05);
        if self.max_interval_s < self.min_interval_s {
            self.max_interval_s = self.min_interval_s;
        }
        self.max_distance_px = self.max_distance_px.max(1);
        self.idle_key_chance = self.idle_key_chance.clamp(0.0, 1.0);
    }
}
