//! Poltergeist for Linux desktops: a native GTK4 + libadwaita app.
//! Shares the engine, config and backends with every other build.

#[cfg(target_os = "linux")]
mod app;

#[cfg(target_os = "linux")]
fn main() -> gtk::glib::ExitCode {
    app::run()
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("poltergeist-gtk is for Linux desktops. Use poltergeist-app on this platform.");
}
