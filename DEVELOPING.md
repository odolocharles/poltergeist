# Developing Poltergeist

For people who build, test and release the app. Users should read [README.md](README.md).

## Layout

```
core/        config, storage paths, backend traits, shared state, action engine (no OS code)
backends/    one module per OS: linux (uinput), windows (SendInput), macos (CoreGraphics)
cli/         `poltergeist`, the headless command
app-gtk/     `poltergeist-gtk`, the native Linux app (GTK4 + libadwaita)
app/         `poltergeist-app`, the egui app for Windows, macOS and Android
assets/      icons (PNG, SVG, ICO, ICNS)
packaging/   linux (.run installer, launcher, udev rule), windows (Inno Setup), macos (.app + .dmg)
```

Everything is Rust. The engine, config and backends are shared; only the windows differ. Each OS backend is behind `cfg(target_os)`, so a broken backend on one OS never affects another.

## Build and test

```bash
cargo test -p poltergeist-core                 # engine tests (click ordering, limits)
cargo build --release -p poltergeist-cli       # CLI, any platform
cargo build --release -p poltergeist-app       # egui app (Windows, macOS; also runs on Linux)
cargo build --release -p poltergeist-gtk       # GTK app (Linux only)
```

The GTK app needs development files: Fedora `sudo dnf install gcc pkgconf-pkg-config gtk4-devel libadwaita-devel`; Ubuntu 24.04+ `sudo apt install build-essential pkg-config libgtk-4-dev libadwaita-1-dev`. Plain `cargo build` skips the GTK app on purpose, so it never needs those libraries.

Install the GTK app from source: `./packaging/linux/install-app.sh` (add `./packaging/linux/setup-uinput.sh` once for real mouse control).

Android needs the Android SDK and NDK: `rustup target add aarch64-linux-android`, `cargo install cargo-apk2 --locked`, then `cargo apk2 run -p poltergeist-app --lib --target aarch64-linux-android`.

## The installers (built by CI)

| Platform | File | Built by |
|---|---|---|
| Linux | `poltergeist-linux-x86_64.run` | `packaging/linux/make-run.sh` (self-extracting installer; needs only GTK4 + libadwaita at run time) |
| Windows | `Poltergeist-Setup.exe` | Inno Setup, `packaging/windows/poltergeist.iss` |
| macOS | `Poltergeist.dmg` | `packaging/macos/make-app.sh` (universal app, ad-hoc signed) |
| Android | `poltergeist.apk` | `cargo apk2 build --release` (throwaway signing key per build) |

The file names are fixed on purpose, so `releases/latest/download/NAME` links in the README never change.

## Releasing

Every push to `main` builds all four installers on GitHub Actions. Pushing a version tag publishes them as a release once all four jobs pass:

```bash
# bump `version` in Cargo.toml first, commit and push, wait for green, then:
git tag -a v0.5.2 -m "v0.5.2" && git push origin v0.5.2
```

To test a build before releasing, open the run under **Actions** and download from **Artifacts**, or:

```bash
gh run list --workflow build --limit 5                 # note a green run's number
gh run download RUN_ID -n poltergeist-linux            # also: poltergeist-windows, poltergeist-macos, poltergeist-android
```

Replace `RUN_ID` with the number, digits only.

## Public downloads from a private repo

Release downloads are only public if the repo that holds the release is public. Options:

1. **Make this repo public.** Simplest; the source becomes public too.
2. **Publish releases to a separate public repo** and keep the source private. The release job supports this:
   ```bash
   gh repo create OWNER/poltergeist-releases --public --add-readme
   gh variable set RELEASE_REPO --body OWNER/poltergeist-releases -R odolocharles/poltergeist
   gh secret set RELEASE_TOKEN -R odolocharles/poltergeist    # paste a token with Contents read/write on the releases repo
   sed -i 's#odolocharles/poltergeist/releases#OWNER/poltergeist-releases/releases#g' README.md
   ```
   Replace `OWNER` with the account that will own the public repo. After this, tags pushed here publish their installers to the public repo, and the README download links point there.

Until one of these is done, the README download links only work for people who can see this repo.

## Settings and output

| OS | Settings file |
|---|---|
| Linux | `~/.config/poltergeist/config.json` |
| macOS | `~/Library/Application Support/Poltergeist/config.json` |
| Windows | `%APPDATA%\Poltergeist\config\config.json` |
| Android | the app's private storage |

On first run the app creates `Poltergeist/screenshots` and `Poltergeist/screen-recordings` in the Pictures folder.

## Platform notes

| Platform | Keep-awake | Cursor and clicks |
|---|---|---|
| Linux | `systemd-inhibit` | `/dev/uinput` virtual pointer; the installer adds a udev rule for access |
| Windows | `SetThreadExecutionState` | `SendInput` (Win32 declarations written by hand, no binding crate) |
| macOS | `caffeinate` | CoreGraphics events (declarations by hand); needs Accessibility permission |
| Android | window keep-screen-on flag | not possible without an Accessibility Service |

## Known gaps

- The Windows and macOS installers and backends have only been compiled in CI; real-device testing is still needed.
- On GNOME/Wayland an app cannot learn its own window position, so the GTK app has no "aim at test pad" button.
- Releases are not code-signed (Windows) or notarized (macOS), so first launch shows a warning.

## Roadmap

- [ ] Screenshots and screen recording (the folders already exist)
- [ ] Tray icon, so it keeps running with the window closed
- [ ] Android Accessibility Service for taps (Kotlin glue)
- [ ] Pause when the real user is active, panic hotkey
- [ ] Signed and notarized releases, stable Android signing key
- [ ] Macro recorder, image-triggered actions
