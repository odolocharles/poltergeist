use clap::{Parser, Subcommand};
use poltergeist_backends::{dry_run::{DryRunInput, DryRunPower}, NativeInput, NativePower};
use poltergeist_core::{Config, Engine, Mode, Paths};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

#[derive(Parser)]
#[command(name = "poltergeist", version, about = "Keep-awake and cursor automator. Fully offline.")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run the engine until Ctrl+C / SIGTERM (this is what boot services run).
    Run {
        /// Print actions instead of performing them (no permissions needed).
        #[arg(long)]
        dry_run: bool,
        /// Override mode: keep-awake, cursor or both.
        #[arg(long)]
        mode: Option<String>,
        /// Override minimum seconds between moves.
        #[arg(long)]
        min: Option<f64>,
        /// Override maximum seconds between moves.
        #[arg(long)]
        max: Option<f64>,
    },
    /// Print the active config and where it lives.
    Config,
    /// Create the Poltergeist/screenshots and screen-recordings folders and print them.
    InitPaths,
}

fn main() {
    if let Err(e) = real_main() {
        eprintln!("poltergeist: {e}");
        std::process::exit(1);
    }
}

fn real_main() -> poltergeist_core::Result<()> {
    match Cli::parse().cmd {
        Cmd::Config => {
            let cfg = Config::load_or_create()?;
            println!("# {}", Config::path().display());
            println!("{}", serde_json::to_string_pretty(&cfg)?);
        }
        Cmd::InitPaths => {
            let cfg = Config::load_or_create()?;
            let p = Paths::init(cfg.output_root)?;
            println!("root:        {}", p.root.display());
            println!("screenshots: {}", p.screenshots.display());
            println!("recordings:  {}", p.recordings.display());
        }
        Cmd::Run { dry_run, mode, min, max } => {
            let mut cfg = Config::load_or_create()?;
            if let Some(m) = mode {
                cfg.mode = match m.as_str() {
                    "keep-awake" => Mode::KeepAwake,
                    "cursor" => Mode::Cursor,
                    "both" => Mode::Both,
                    other => {
                        return Err(poltergeist_core::Error::Backend(format!(
                            "unknown mode '{other}' (use keep-awake, cursor or both)"
                        )))
                    }
                };
            }
            if let Some(v) = min { cfg.min_interval_s = v; }
            if let Some(v) = max { cfg.max_interval_s = v.max(cfg.min_interval_s); }

            let stop = Arc::new(AtomicBool::new(false));
            let s = stop.clone();
            ctrlc::set_handler(move || s.store(true, Ordering::Relaxed))
                .map_err(|e| poltergeist_core::Error::Backend(e.to_string()))?;

            eprintln!("poltergeist: running (mode {:?}). Ctrl+C to stop.", cfg.mode);
            if dry_run {
                Engine::new(cfg, DryRunInput, DryRunPower, stop).run()?;
            } else {
                let input = NativeInput::new()?;
                Engine::new(cfg, input, NativePower::default(), stop).run()?;
            }
        }
    }
    Ok(())
}
