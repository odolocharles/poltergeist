# 👻 Poltergeist

> A small, **fully offline** app that keeps your machine awake, moves the cursor and clicks where you tell it to. Written in Rust. Runs on Linux, Windows, macOS and Android.

**Status:** early development (v0.4.1) · **Repo:** private (`odolocharles/poltergeist`)

## Which file do I need?

| Your device | File | Section |
|---|---|---|
| Fedora / RHEL-family Linux | `poltergeist-VERSION-1.x86_64.rpm` | [Linux](#linux) |
| Other Linux | `poltergeist-linux-x86_64.tar.gz` | [Linux](#linux) |
| Windows 10 / 11 | `poltergeist-windows-x64.zip` | [Windows](#windows) |
| macOS 11+ (Apple silicon and Intel) | `poltergeist-macos.zip` | [macOS](#macos) |
| Android 8+ (Huawei EMUI too) | `poltergeist.apk` | [Android](#android) |

## Step 1: get the file

The repo is private, so downloads need the GitHub CLI logged in as the repo owner (`gh auth login`, or `gh auth switch --user odolocharles`). Pick one way.

**A. From a published release** (exists once a version tag such as `v0.4.1` has been pushed and its build passed):

```bash
gh release list -R odolocharles/poltergeist
gh release download -R odolocharles/poltergeist -D ~/Downloads --pattern '*.rpm'
```

Change `'*.rpm'` to `'*.zip'`, `'*.apk'` or `'*.tar.gz'` for the other files. In a browser: GitHub repo, then **Releases**, then the newest release, then **Assets**.

**B. Straight from a build** (works before any release exists):

```bash
gh run list -R odolocharles/poltergeist --workflow build --limit 5
```

Pick a run with a green check and copy its number. Then replace `RUN_ID` with that number (type the digits only, no brackets):

```bash
gh run download RUN_ID -R odolocharles/poltergeist -n poltergeist-linux   -D ~/Downloads/poltergeist
gh run download RUN_ID -R odolocharles/poltergeist -n poltergeist-windows-x64 -D ~/Downloads/poltergeist
gh run download RUN_ID -R odolocharles/poltergeist -n poltergeist-macos   -D ~/Downloads/poltergeist
gh run download RUN_ID -R odolocharles/poltergeist -n poltergeist-android -D ~/Downloads/poltergeist
```

Run only the line for your device. The files land in `~/Downloads/poltergeist`.

## Step 2: install

### Linux

The Linux app is native GTK4 + libadwaita (GNOME-style header bar, follows your light/dark theme, F11 for full screen).

**Fedora (RPM):**

```bash
sudo dnf install ~/Downloads/poltergeist/poltergeist-*.rpm
```

Then **log out and back in once**. The package installs the app, its menu entry, the icon and a rule that lets your desktop session create virtual input devices (needed for real mouse and clicks). Open **Poltergeist** from your app menu. Remove it with `sudo dnf remove poltergeist`.

**Any distro, from source** (build tools and GTK4 + libadwaita 1.4 or newer development files):

```bash
# Fedora
sudo dnf install gcc pkgconf-pkg-config gtk4-devel libadwaita-devel
# Debian / Ubuntu 24.04+
sudo apt install build-essential pkg-config libgtk-4-dev libadwaita-1-dev

tar xzf ~/Downloads/poltergeist/poltergeist-linux-x86_64.tar.gz -C ~/
cd ~/poltergeist-linux-x86_64         # needs a Rust toolchain: https://rustup.rs
./packaging/linux/install-app.sh      # builds, then adds a menu entry and icon (no root)
./packaging/linux/setup-uinput.sh     # one time: allows real mouse and click control
```

Log out and back in once after `setup-uinput.sh`. Remove with `./packaging/linux/uninstall-app.sh`. Do not install both the RPM and the source build; remove one first.

### Windows

1. Right-click `poltergeist-windows-x64.zip`, choose **Extract All**, and pick a folder such as `C:\Users\YOU\Poltergeist`.
2. Double-click `poltergeist-app.exe`.
3. The first launch shows a blue **Windows protected your PC** box, because the build is not code-signed. Click **More info**, then **Run anyway**.

To start it with Windows, press `Win + R`, type `shell:startup`, and put a shortcut to `poltergeist-app.exe` in that folder. To remove it, delete the folder.

Windows will not let it send clicks to windows running **as administrator** unless Poltergeist is run as administrator too.

The command-line version is `poltergeist.exe` in the same folder, for example `poltergeist.exe run --click-at 50,50 --click-every 5` in PowerShell.

### macOS

1. Double-click `poltergeist-macos.zip` to unzip it, then drag **Poltergeist** into your **Applications** folder.
2. The app is not notarized, so the first launch is blocked. **Right-click** Poltergeist, choose **Open**, then **Open** again. If macOS still refuses, run this once in Terminal and try again:
   ```bash
   xattr -dr com.apple.quarantine /Applications/Poltergeist.app
   ```
3. Press **Start**. macOS will ask for the **Accessibility** permission and open the settings page. In **System Settings, Privacy & Security, Accessibility**, switch Poltergeist on (or add it with the **+** button), then press **Start** again. Without this permission macOS silently ignores cursor and click events, so the app runs in simulation instead.

To start it at login: **System Settings, General, Login Items**, then **+**, then choose Poltergeist. To remove it, drag it to the Bin.

The app is only ad-hoc signed, so macOS may ask for the Accessibility permission again after an update. The command-line version is inside the app: `/Applications/Poltergeist.app/Contents/MacOS/poltergeist`.

### Android

1. Copy `poltergeist.apk` to your phone (USB, Bluetooth, a file-sharing app, or `adb install poltergeist.apk`).
2. Open the file. When the phone asks, allow installing from that source.
3. Open **Poltergeist** and tap **Keep screen awake**.

Android keeps the screen on **only while Poltergeist is open and in front**. Moving a cursor or tapping for you is not possible on Android without a special Accessibility Service, so the phone app is keep-awake only. Huawei phones with EMUI install the same APK. HarmonyOS NEXT phones cannot run Android apps.

Builds are signed with a throwaway key made for each build, so uninstall the old version before installing a newer one.

## Using it

- **Mode:** keep awake only, move the cursor only, or both.
- **Cursor movement:** random, jitter or circle, with wait times, range and human-like curved paths.
- **Auto-click:** turn it on, pick the button, then click or drag inside the **live preview** to choose the spot. Set the wait between clicks, how long each press is held, and an optional limit on the number of clicks.
- **Live preview:** shows the pointer, the click spot and a ripple on every click.
- **Test pad:** counts clicks the operating system really delivers. Put the click spot over the pad and watch the counter rise.
- **Simulation only:** never touches your real mouse. The app falls back to it by itself if real input is not allowed, and says why.
- Every change applies immediately and is saved automatically.

On first run the app creates `Poltergeist/screenshots` and `Poltergeist/screen-recordings` inside your Pictures folder.

### Where settings are stored

| OS | File |
|---|---|
| Linux | `~/.config/poltergeist/config.json` |
| macOS | `~/Library/Application Support/Poltergeist/config.json` |
| Windows | `%APPDATA%\Poltergeist\config\config.json` |
| Android | the app's private storage |

### Command line

```bash
poltergeist run --dry-run                                # print actions, touch nothing
poltergeist run --click-at 37,69 --click-every 5 --max-clicks 20
poltergeist run --mode keep-awake
poltergeist config                                       # show settings and where they live
```

`--click-at X,Y` is a position in percent of the screen. Stop with Ctrl+C.

### Run at boot (Linux)

The RPM installs a service that needs no window:

```bash
sudo systemctl enable --now poltergeist
```

It reads the same settings file as the app. For a source install, first copy the binary: `sudo install -m755 ~/.local/bin/poltergeist /usr/bin/poltergeist`, then `sudo install -m644 packaging/linux/poltergeist.service /etc/systemd/system/`.

## Offline by design

No network code, no telemetry, no web assets. The Android build does not request the `INTERNET` permission. Boot actions run from the headless service, never from the UI.

## Platform support

| Platform | Keep-awake | Cursor and clicks | App |
|---|---|---|---|
| Linux (X11 / Wayland / VM) | systemd-inhibit | uinput virtual pointer | GTK4 app, CLI, RPM |
| Windows 10/11 | SetThreadExecutionState | SendInput | egui app, CLI |
| macOS 11+ | caffeinate | CoreGraphics events | egui app in a universal `.app`, CLI |
| Android 8+ | window keep-screen-on | not possible | egui app (APK) |

Known limits: Windows marks synthetic input as "injected" and cannot reach administrator windows; macOS needs the Accessibility permission; on GNOME/Wayland an app cannot learn its own window position, so the click spot is chosen in the preview.

## Building from source

```bash
cargo build --release -p poltergeist-cli                 # CLI, any platform
cargo build --release -p poltergeist-app                 # egui app (Windows, macOS, Linux)
cargo build --release -p poltergeist-gtk                 # GTK app (Linux only)
cargo test -p poltergeist-core
```

Android needs the Android SDK and NDK: `rustup target add aarch64-linux-android`, `cargo install cargo-apk2 --locked`, then `cargo apk2 run -p poltergeist-app --lib --target aarch64-linux-android`.

## Releasing

Every push builds all five files on GitHub Actions. Pushing a version tag publishes them as a release once every job is green:

```bash
git tag -a v0.4.1 -m "v0.4.1" && git push origin v0.4.1
```

## Repo layout

```
core/        config, storage paths, backend traits, shared state, action engine (no OS code)
backends/    one module per OS: linux (uinput), windows (SendInput), macos (CoreGraphics)
cli/         `poltergeist` headless command
app-gtk/     `poltergeist-gtk`, the native Linux app (GTK4 + libadwaita)
app/         `poltergeist-app`, the egui app for Windows, macOS and Android
assets/      icons (PNG, SVG, ICO, ICNS)
packaging/   Linux (RPM metadata, systemd, udev, launcher) and macOS (.app builder)
```

## Roadmap

- [ ] Screenshots and screen recording (the folders already exist)
- [ ] Tray icon, so it keeps running with the window closed
- [ ] Android Accessibility Service for taps
- [ ] Pause automatically when the real user is active, panic hotkey
- [ ] Signed releases (Windows code signing, Apple notarization, stable Android key)
- [ ] Macro recorder, image-triggered actions

## License

Private, all rights reserved (for now).
