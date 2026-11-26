// Embeds the app icon into the Windows .exe so it shows in Explorer and the taskbar.
fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../assets/poltergeist.ico");
        res.set("ProductName", "Poltergeist");
        res.set("FileDescription", "Poltergeist keep-awake and automation");
        if let Err(e) = res.compile() {
            println!("cargo:warning=could not embed the Windows icon: {e}");
        }
    }
}
