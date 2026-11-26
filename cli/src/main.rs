use clap::{Parser, Subcommand};
use poltergeist_backends::{
    dry_run::{DryRunInput, DryRunPower},
    NativeInput, NativePower,
};
use poltergeist_core::{
    state::new_state, Config, Engine, Error, InputBackend, Mode, Paths, PowerBackend, Result,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Parser)]
#[command(name = "poltergeist", version, about = "Keep-awake, cursor and click automator. Fully offline.")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run the engine until Ctrl+C / SIGTERM (this is what boot services run).
    Run {
        /// Simulate: print actions instead of performing them (no permissions needed).
        #[arg(long)]
        dry_run: bool,
        /// Override mode: keep-awake, cursor or both.
        #[arg(long)]
        mode: Option<String>,
        /// Auto-click at this spot, as percentages of the screen, e.g. --click-at 50,50
        #[arg(long)]
        click_at: Option<String>,
        /// Seconds between auto-clicks (default from config).
        #[arg(long)]
        click_every: Option<f64>,
        /// Stop clicking after this many clicks.
        #[arg(long)]
        max_clicks: Option<u32>,
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

fn parse_spot(s: &str) -> Result<(f32, f32)> {
    let bad = || Error::Backend(format!("--click-at expects 'X,Y' percentages like 50,50 (got '{s}')"));
    let (a, b) = s.split_once(',').ok_or_else(bad)?;
    let x: f32 = a.trim().parse().map_err(|_| bad())?;
    let y: f32 = b.trim().parse().map_err(|_| bad())?;
    Ok(((x / 100.0).clamp(0.0, 1.0), (y / 100.0).clamp(0.0, 1.0)))
}

fn real_main() -> Result<()> {
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
        Cmd::Run { dry_run, mode, click_at, click_every, max_clicks } => {
            let mut cfg = Config::load_or_create()?;
            if let Some(m) = mode {
                cfg.mode = match m.as_str() {
                    "keep-awake" => Mode::KeepAwake,
                    "cursor" => Mode::Cursor,
                    "both" => Mode::Both,
                    other => {
                        return Err(Error::Backend(format!(
                            "unknown mode '{other}' (use keep-awake, cursor or both)"
                        )))
                    }
                };
            }
            if let Some(spot) = click_at {
                let (x, y) = parse_spot(&spot)?;
                cfg.click.enabled = true;
                cfg.click.x = x;
                cfg.click.y = y;
            }
            if let Some(s) = click_every {
                cfg.click.min_interval_s = s;
                cfg.click.max_interval_s = s;
            }
            if let Some(n) = max_clicks {
                cfg.click.max_clicks = n;
            }
            cfg.sanitize();

            let stop = Arc::new(AtomicBool::new(false));
            let s = stop.clone();
            ctrlc::set_handler(move || s.store(true, Ordering::Relaxed))
                .map_err(|e| Error::Backend(e.to_string()))?;

            let (input, power): (Box<dyn InputBackend + Send>, Box<dyn PowerBackend + Send>) = if dry_run {
                (Box::new(DryRunInput::default()), Box::new(DryRunPower::default()))
            } else {
                (Box::new(NativeInput::new()?), Box::new(NativePower::default()))
            };

            eprintln!("poltergeist: running (mode {:?}, click {}). Ctrl+C to stop.", cfg.mode, if cfg.click.enabled { "on" } else { "off" });
            Engine::new(cfg.shared(), input, power, stop, new_state()).verbose(true).run()?;
        }
    }
    Ok(())
}
