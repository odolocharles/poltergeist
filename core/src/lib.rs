//! Poltergeist core: config, storage paths, backend traits and the action engine.
//! No OS-specific code lives here, and nothing here touches the network.

pub mod backend;
pub mod config;
pub mod engine;
pub mod paths;

pub use backend::{Button, InputBackend, PowerBackend};
pub use config::{Config, Mode, MovementStyle};
pub use engine::Engine;
pub use paths::Paths;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config error: {0}")]
    Config(#[from] serde_json::Error),
    #[error("backend error: {0}")]
    Backend(String),
}

pub type Result<T> = std::result::Result<T, Error>;
