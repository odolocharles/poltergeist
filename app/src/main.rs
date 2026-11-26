// Release builds on Windows run as a GUI app with no console window.
#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]

#[cfg(not(target_os = "android"))]
fn main() -> eframe::Result<()> {
    poltergeist_app::run_desktop()
}

// Android enters through `android_main` in the library; this binary is unused there.
#[cfg(target_os = "android")]
fn main() {}
