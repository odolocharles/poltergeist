# 👻 Poltergeist

> A small, **fully offline** app that keeps your machine awake, moves the cursor and clicks where you tell it to. Written in Rust. Runs on Linux, Windows, macOS and Android.

**Status:** early development (v0.5.1) · **Repo:** private (`odolocharles/poltergeist`)

## Install

Each platform has **one installer**. Find your platform and copy **only that one block**. Every block is complete on its own and has nothing to edit.

| Your device | Installer | Needs |
|---|---|---|
| Linux | `poltergeist-VERSION-linux-x86_64.run` | 64-bit Intel/AMD, Fedora 40+ / Ubuntu 24.04+ or similar, GTK4 + libadwaita (part of GNOME) |
| Windows | `Poltergeist-Setup-VERSION.exe` | Windows 10 or 11, 64-bit |
| macOS | `Poltergeist-VERSION.dmg` | macOS 11 or newer, Apple silicon or Intel |
| Android | `poltergeist-VERSION.apk` | Android 8 or newer |

### Before you start (once)

The repo is private, so downloads use the GitHub CLI. Check that the **owner** account is the active one:

```bash
gh auth status
```

If it says `odolocharles` is not active, run `gh auth switch --user odolocharles`. If you are not logged in at all, run `gh auth login`.

The commands below download the **latest release**. A release exists once a version tag such as `v0.5.1` has been pushed and its build passed. No release yet? See [No release yet](#no-release-yet-use-the-latest-build).

### Linux

```bash
(
  d="$(mktemp -d)" && cd "$d" &&
  gh release download -R odolocharles/poltergeist --pattern '*.run' &&
  chmod +x ./*.run &&
  ./*.run
)
```

That downloads into a fresh temporary folder, so older downloads can never get in the way, and starts the installer. It needs no compiler. It puts the app in `~/.local/bin`, adds **Poltergeist** to your app menu, and asks once (with your password) whether to allow real mouse and click control. Answer **Y**, then **log out and back in once**. If you answer no, the app runs in simulation only; run the installer again later to change that.

If GTK4 or libadwaita is missing, it prints the exact command for your distro. Open the app from your menu, or run `poltergeist-gtk`. The command-line tool is `poltergeist`.

To remove it, run the same installer with `--uninstall` (download it again with the block above, but end with `./*.run --uninstall`), or delete these by hand: `~/.local/bin/poltergeist*` and `~/.local/share/applications/org.sorabbyngo.Poltergeist.desktop`.

### Windows

Open **PowerShell** and paste:

```powershell
$d = New-Item -ItemType Directory -Path (Join-Path $env:TEMP "poltergeist-install") -Force
gh release download -R odolocharles/poltergeist --pattern '*.exe' -D $d.FullName --clobber
Start-Process (Get-ChildItem $d.FullName -Filter 'Poltergeist-Setup-*.exe' | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
```

The setup wizard opens.

1. Windows may show a blue **Windows protected your PC** box, because the installer is not code-signed yet. Click **More info**, then **Run anyway**.
2. Follow the wizard. It installs for your user only, so it does not ask for an administrator password. You can tick **Start Poltergeist when I sign in** and **Create a desktop shortcut**.
3. Finish, and Poltergeist opens. It is also in the Start menu.

Remove it from **Settings, Apps, Installed apps**. Windows will not let Poltergeist send clicks to windows running **as administrator**, unless Poltergeist is run as administrator too. The command-line tool `poltergeist.exe` is in `%LOCALAPPDATA%\Programs\Poltergeist`.

### macOS

Open **Terminal** and paste:

```bash
(
  d="$(mktemp -d)" && cd "$d" &&
  gh release download -R odolocharles/poltergeist --pattern '*.dmg' &&
  open ./*.dmg
)
```

1. A window opens with the app next to an **Applications** shortcut. Drag **Poltergeist** onto **Applications**.
2. The app is not notarized by Apple yet, so the first launch is blocked. In **Applications**, **right-click** Poltergeist, choose **Open**, then **Open** again. If macOS still refuses, run this once and try again:
   ```bash
   xattr -dr com.apple.quarantine /Applications/Poltergeist.app
   ```
3. Press **Start**. macOS asks for the **Accessibility** permission and opens the settings page. Switch Poltergeist on there (or add it with **+**), then press **Start** again. Without that permission macOS silently ignores cursor and click events, so the app runs in simulation instead.

To start it at login: **System Settings, General, Login Items, +**. To remove it, drag it to the Bin. The app is only ad-hoc signed, so macOS may ask for the Accessibility permission again after an update. The command-line tool is inside the app at `/Applications/Poltergeist.app/Contents/MacOS/poltergeist`.

### Android

On the computer, download the file:

```bash
gh release download -R odolocharles/poltergeist --pattern '*.apk' -D ~/Downloads --clobber
```

Then copy `~/Downloads/poltergeist-VERSION.apk` to your phone (USB, Bluetooth, or a file-sharing app) and open it. When the phone asks, allow installing from that source. Open **Poltergeist** and tap **Keep screen awake**.

With a phone connected over `adb` you can install in one step. Builds are signed with a throwaway key made for each build, so remove the old version first:

```bash
adb uninstall org.sorabbyngo.poltergeist ; adb install ~/Downloads/poltergeist-*.apk
```

Android keeps the screen on **only while Poltergeist is open and in front**. Moving a cursor or tapping for you is not possible on Android without a special Accessibility Service, so the phone app is keep-awake only. Huawei phones with EMUI install the same APK. HarmonyOS NEXT phones cannot run Android apps.

### No release yet? Use the latest build

Every push builds the installers, so you can fetch the newest successful build instead. Do the same as above but replace the download line as follows. First find the newest successful build (this works in Terminal and PowerShell; in PowerShell write `$RUN = ...`):

```bash
RUN="$(gh run list -R odolocharles/poltergeist --workflow build --status success --limit 1 --json databaseId -q '.[0].databaseId')"
```

Then, instead of `gh release download -R odolocharles/poltergeist --pattern '...'`, use the line for your platform (in PowerShell add `-D $d.FullName` and use `$RUN`):

| Platform | Download line |
|---|---|
| Linux | `gh run download "$RUN" -R odolocharles/poltergeist -n poltergeist-linux` |
| Windows | `gh run download $RUN -R odolocharles/poltergeist -n poltergeist-windows -D $d.FullName` |
| macOS | `gh run download "$RUN" -R odolocharles/poltergeist -n poltergeist-macos` |
| Android | `gh run download "$RUN" -R odolocharles/poltergeist -n poltergeist-android -D ~/Downloads` |

For example, the complete Linux block becomes:

```bash
RUN="$(gh run list -R odolocharles/poltergeist --workflow build --status success --limit 1 --json databaseId -q '.[0].databaseId')"
(
  d="$(mktemp -d)" && cd "$d" &&
  gh run download "$RUN" -R odolocharles/poltergeist -n poltergeist-linux &&
  chmod +x ./*.run &&
  ./*.run
)
```

In a browser, the same files are on a build's page under **Actions** (Artifacts, at the bottom) or on the repo's **Releases** page (Assets).

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
git tag -a v0.5.1 -m "v0.5.1" && git push origin v0.5.1
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
