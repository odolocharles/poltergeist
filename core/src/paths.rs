use crate::Result;
use directories::{ProjectDirs, UserDirs};
use std::{fs, path::PathBuf, sync::OnceLock};

static CONFIG_DIR_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

/// Lets a platform (Android) point the config at its app-private folder. Call once at startup.
pub fn set_config_dir(dir: PathBuf) {
    let _ = CONFIG_DIR_OVERRIDE.set(dir);
}

pub struct Paths {
    pub root: PathBuf,
    pub screenshots: PathBuf,
    pub recordings: PathBuf,
}

pub fn config_dir() -> PathBuf {
    if let Some(d) = CONFIG_DIR_OVERRIDE.get() {
        return d.clone();
    }
    ProjectDirs::from("", "", "Poltergeist")
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".poltergeist"))
}

impl Paths {
    /// Creates `Poltergeist/screenshots` and `Poltergeist/screen-recordings`
    /// on first run. Falls back to home, then the working directory, so it
    /// also works on minimal VM images with no Pictures folder.
    pub fn init(custom_root: Option<PathBuf>) -> Result<Self> {
        let root = custom_root.unwrap_or_else(|| {
            let user = UserDirs::new();
            user.as_ref()
                .and_then(|d| d.picture_dir().map(|p| p.to_path_buf()))
                .or_else(|| user.as_ref().map(|d| d.home_dir().to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
                .join("Poltergeist")
        });
        let screenshots = root.join("screenshots");
        let recordings = root.join("screen-recordings");
        fs::create_dir_all(&screenshots)?;
        fs::create_dir_all(&recordings)?;
        Ok(Self { root, screenshots, recordings })
    }
}
