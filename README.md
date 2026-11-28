# 👻 Poltergeist

> A small, **fully offline** app that keeps your machine awake, moves the cursor and clicks where you tell it to. Written in Rust. Runs on Linux, Windows, macOS and Android.

**Status:** early development (v0.5.0) · **Repo:** private (`odolocharles/poltergeist`)

## Install

Every platform has **one installer file**. Find your platform below and follow only that section.

| Your device | Installer | Needs |
|---|---|---|
| Linux (Fedora 40+, Ubuntu 24.04+ and similar) | `poltergeist-VERSION-linux-x86_64.run` | 64-bit Intel/AMD, GTK4 + libadwaita (part of GNOME) |
| Windows | `Poltergeist-Setup-VERSION.exe` | Windows 10 or 11, 64-bit |
| macOS | `Poltergeist-VERSION.dmg` | macOS 11 or newer, Apple silicon or Intel |
| Android | `poltergeist-VERSION.apk` | Android 8 or newer |

**Downloading.** The repo is private, so downloads need the GitHub CLI signed in as the owner (`gh auth login`, or `gh auth switch --user odolocharles`). Each platform below shows two ways to download:

- **From a release**, which exists once a version tag such as `v0.5.0` has been pushed and its build passed.
- **From a build**, which works before any release exists. First run `gh run list -R odolocharles/poltergeist --workflow build --limit 5`, pick a run with a green check, and replace `RUN_ID` in the command with its number (digits only, no brackets).

In a browser, the same files are under the repo's **Releases** page (Assets) or a build's page under **Actions** (Artifacts, at the bottom).

### Linux

```bash
# download (use one of the two)
gh release download -R odolocharles/poltergeist -D ~/Downloads --pattern '*.run'
gh run download RUN_ID -R odolocharles/poltergeist -n poltergeist-linux -D ~/Downloads

# install
chmod +x ~/Downloads/poltergeist-*-linux-x86_64.run
~/Downloads/poltergeist-*-linux-x86_64.run
```

If you have downloaded several versions, give the exact file name instead of the `*`.

The installer needs no compiler. It puts the app in `~/.local/bin`, adds **Poltergeist** to your app menu with its icon, and asks once (with your password) whether to allow real mouse and click control. Say yes, then **log out and back in once**. If you say no, the app runs in simulation only. You can run the installer again later to change that.

If GTK4 or libadwaita is missing it tells you the exact command for your distro. Open the app from your menu, or from a terminal with `poltergeist-gtk`. The command-line tool is `poltergeist`.

To remove it: `~/Downloads/poltergeist-*-linux-x86_64.run --uninstall`

### Windows

```powershell
# download (use one of the two)
gh release download -R odolocharles/poltergeist -D $HOME\Downloads --pattern '*.exe'
gh run download RUN_ID -R odolocharles/poltergeist -n poltergeist-windows -D $HOME\Downloads
```

1. Double-click `Poltergeist-Setup-VERSION.exe`.
2. Windows shows a blue **Windows protected your PC** box, because the installer is not code-signed yet. Click **More info**, then **Run anyway**.
3. Follow the wizard. It installs for your user only, so it does not ask for an administrator password. You can tick **Start Poltergeist when I sign in** and **Create a desktop shortcut**.
4. Finish, and Poltergeist opens. It is also in the Start menu.

Remove it from **Settings, Apps, Installed apps**. Windows will not let Poltergeist send clicks to windows running **as administrator**, unless Poltergeist is run as administrator too. The command-line tool `poltergeist.exe` is in the install folder (`%LOCALAPPDATA%\Programs\Poltergeist`).

### macOS

```bash
# download (use one of the two)
gh release download -R odolocharles/poltergeist -D ~/Downloads --pattern '*.dmg'
gh run download RUN_ID -R odolocharles/poltergeist -n poltergeist-macos -D ~/Downloads
```

1. Double-click `Poltergeist-VERSION.dmg`. A window opens with the app next to an **Applications** shortcut. Drag **Poltergeist** onto **Applications**.
2. The app is not notarized by Apple yet, so the first launch is blocked. In **Applications**, **right-click** Poltergeist, choose **Open**, then **Open** again. If macOS still refuses, run this once in Terminal and try again:
   ```bash
   xattr -dr com.apple.quarantine /Applications/Poltergeist.app
   ```
3. Press **Start**. macOS asks for the **Accessibility** permission and opens the settings page. Switch Poltergeist on there (or add it with **+**), then press **Start** again. Without that permission macOS silently ignores cursor and click events, so the app runs in simulation instead.

To start it at login: **System Settings, General, Login Items, +**. To remove it, drag it to the Bin. The app is only ad-hoc signed, so macOS may ask for the Accessibility permission again after an update. The command-line tool is inside the app at `/Applications/Poltergeist.app/Contents/MacOS/poltergeist`.

### Android

```bash
# download (use one of the two)
gh release download -R odolocharles/poltergeist -D ~/Downloads --pattern '*.apk'
gh run download RUN_ID -R odolocharles/poltergeist -n poltergeist-android -D ~/Downloads
```

1. Copy `poltergeist-VERSION.apk` to your phone (USB, Bluetooth, a file-sharing app, or `adb install poltergeist-VERSION.apk`).
2. Open the file and, when the phone asks, allow installing from that source.
3. Open **Poltergeist** and tap **Keep screen awake**.

Android keeps the screen on **only while Poltergeist is open and in front**. Moving a cursor or tapping for you is not possible on Android without a special Accessibility Service, so the phone app is keep-awake only. Huawei phones with EMUI install the same APK. HarmonyOS NEXT phones cannot run Android apps. Builds are signed with a throwaway key made for each build, so uninstall the old version before installing a newer one.

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

### Run at boot or login

- **Windows:** tick the option in the installer, or put a shortcut in `shell:startup`.
- **macOS:** System Settings, General, Login Items, +.
- **Linux:** a background service that needs no window and starts when you sign in:

```bash
mkdir -p ~/.config/systemd/user
cat > ~/.config/systemd/user/poltergeist.service << 'EOF'
[Unit]
Description=Poltergeist keep-awake and automation (offline)

[Service]
ExecStart=%h/.local/bin/poltergeist run
Restart=on-failure

[Install]
WantedBy=default.target
EOF
systemctl --user enable --now poltergeist
```

It reads the same settings file as the app. Stop it with `systemctl --user disable --now poltergeist`.

## Offline by design

No network code, no telemetry, no web assets. The Android build does not request the `INTERNET` permission. Boot actions run from the headless service, never from the UI.

## Platform support

| Platform | Keep-awake | Cursor and clicks | App |
|---|---|---|---|
| Linux (X11 / Wayland / VM) | systemd-inhibit | uinput virtual pointer | GTK4 app + CLI, `.run` installer |
| Windows 10/11 | SetThreadExecutionState | SendInput | egui app + CLI, setup `.exe` |
| macOS 11+ | caffeinate | CoreGraphics events | egui app + CLI in a universal `.app`, `.dmg` |
| Android 8+ | window keep-screen-on | not possible | egui app (APK) |

Known limits: Windows marks synthetic input as "injected" and cannot reach administrator windows; macOS needs the Accessibility permission; on GNOME/Wayland an app cannot learn its own window position, so the click spot is chosen in the preview.

## Building from source

```bash
cargo build --release -p poltergeist-cli                 # CLI, any platform
cargo build --release -p poltergeist-app                 # egui app (Windows, macOS, Linux)
cargo build --release -p poltergeist-gtk                 # GTK app (Linux only)
./packaging/linux/install-app.sh                          # build and install the GTK app from source (Linux)
cargo test -p poltergeist-core
```

Android needs the Android SDK and NDK: `rustup target add aarch64-linux-android`, `cargo install cargo-apk2 --locked`, then `cargo apk2 run -p poltergeist-app --lib --target aarch64-linux-android`.

## Releasing

Every push builds the four installers on GitHub Actions. Pushing a version tag publishes them as a release once every job is green:

```bash
git tag -a v0.5.0 -m "v0.5.0" && git push origin v0.5.0
```

## Repo layout

```
core/        config, storage paths, backend traits, shared state, action engine (no OS code)
backends/    one module per OS: linux (uinput), windows (SendInput), macos (CoreGraphics)
cli/         `poltergeist` headless command
app-gtk/     `poltergeist-gtk`, the native Linux app (GTK4 + libadwaita)
app/         `poltergeist-app`, the egui app for Windows, macOS and Android
assets/      icons (PNG, SVG, ICO, ICNS)
packaging/   Linux (.run installer builder, launcher, udev rule), Windows (Inno Setup script), macOS (.app and .dmg builder)
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
