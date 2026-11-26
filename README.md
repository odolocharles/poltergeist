# 👻 Poltergeist

> A tiny, **fully offline**, cross-platform ghost that keeps your machine awake and moves the cursor on its own, configured exactly how you want. Written in Rust.

**Status:** early development (v0.2: desktop app + Linux backend) · **Visibility:** private

## What it does

1. **Keep-awake:** stops the screen and system from sleeping or locking.
2. **Cursor automation:** random, jitter or circular movement and (soon) clicks at a chosen spot.
3. **Capture (planned):** screenshots and screen recordings, saved automatically.

Born as a test tool for virtual machines, where idle timeouts and screen blanking get in the way.

## Offline by design

- No network code, no telemetry, no CDN assets. Android builds will not request the `INTERNET` permission.
- Boot actions run from a **headless service** (`poltergeist run`), never from the UI, so wake-on-boot works with no UI and no connection.
- Config is a local JSON file, written atomically.
- On first run the app creates its own output folders on the device:

```
<Pictures or home>/Poltergeist/
├── screenshots/
└── screen-recordings/
```

## Workspace layout

```
core/        config, storage paths, backend traits, shared state, action engine (no OS code)
backends/    one module per OS behind cfg: linux (uinput + systemd-inhibit) done; others stubbed
cli/         `poltergeist` headless binary (what boot services run)
app/         `poltergeist-app` desktop UI (pure Rust, egui: no webview, no network)
assets/      app icon (PNG and SVG)
packaging/   Linux systemd unit, uinput udev rule, desktop launcher and installer scripts
```

A separate build is released per OS. Each backend sits behind `cfg(target_os)`, so a broken backend on one OS never affects another. CI builds Linux, Windows and macOS on every push.

## The app

Install it as a normal desktop app (menu entry and icon, no root needed):

```bash
./packaging/linux/install-app.sh
```

Or just run it: `cargo run --release -p poltergeist-app`

- **Window controls:** on Linux the app draws its own title bar with minimize, maximize/restore and close, plus edge resizing. Double-click the bar to maximize or restore. **F11** toggles full screen and **Esc** always leaves it.

- **Every setting applies live** while it runs, and is auto-saved to the shared config file.
- **Live preview** shows the pointer, a click target and click ripples. Click inside it to choose where auto-click lands.
- **Pointer overlay** (optional): a see-through, click-through ring that follows the pointer across the real screen. Best on X11, Windows and macOS; compositors may restrict it on Wayland.
- **Test pad** counts clicks the operating system actually delivers. Press "Aim auto-click at test pad", then Start, and watch the counter rise. That is the proof the clicks are real.
- **Simulation only** mode never touches your real mouse. The app falls back to it automatically if real input is unavailable.

### How clicks reach the OS

On Linux the app creates a virtual absolute pointer (the same kind of device virtual machines use) through `/dev/uinput`. Events enter the kernel input stack exactly like hardware events, so X11, Wayland and the console see a normal pointer with real press and release events. Each click glides to the spot, pauses briefly, holds the button for a randomized 40-110 ms, then releases. Note that the OS can see the device exists ("Poltergeist Virtual Pointer"), as it can for any virtual input device.

One-time permission setup (so no root is needed):

```bash
./packaging/linux/setup-uinput.sh
```

## Usage (CLI)

```bash
cargo build --release
./target/release/poltergeist run --dry-run          # print actions, no permissions needed
./target/release/poltergeist config                 # show config and its location
./target/release/poltergeist init-paths             # create the output folders
./target/release/poltergeist run --click-at 37,69 --click-every 5 --max-clicks 20
```

Linux real mode needs `/dev/uinput` access: `sudo modprobe uinput` (custom kernels: `CONFIG_INPUT_UINPUT`).

### Run at boot (Linux)

```bash
sudo install -m755 target/release/poltergeist /usr/local/bin/
sudo install -m644 packaging/linux/poltergeist.service /etc/systemd/system/
sudo systemctl enable --now poltergeist
```

### Config

`~/.config/poltergeist/config.json` (created on first run):

```json
{
  "mode": "both",
  "style": "random",
  "min_interval_s": 3.0,
  "max_interval_s": 12.0,
  "radius": 0.12,
  "humanize": true,
  "idle_key_chance": 0.3,
  "click": { "enabled": false, "button": "left", "x": 0.5, "y": 0.5,
             "min_interval_s": 4.0, "max_interval_s": 8.0,
             "hold_ms_min": 40, "hold_ms_max": 110, "scatter": 0.0, "max_clicks": 0 },
  "show_overlay": false,
  "output_root": null
}
```

`mode`: `keep_awake` | `cursor` | `both` · `style`: `random` | `jitter` | `circle`

## Platform support

| Platform | Keep-awake | Cursor | Status |
|---|---|---|---|
| Linux (X11 / Wayland / console / VM) | ✅ systemd-inhibit | ✅ uinput | working |
| Windows | planned | planned | stub |
| macOS | planned | planned | stub (needs Accessibility permission) |
| Android | planned | n/a | wake lock only |
| Huawei (EMUI / AppGallery) | planned | n/a | Android build; HarmonyOS NEXT needs a port |

## Roadmap

- [x] Workspace, config, offline paths, Linux backend, headless CLI, CI matrix
- [x] Desktop app, live preview, pointer overlay, auto-click at a chosen spot, test pad
- [ ] Screenshots and screen recording (segmented, retention limits)
- [ ] Pause when the real user is active, panic hotkey
- [ ] Windows and macOS backends
- [x] Pure-Rust settings UI (egui), installable Linux launcher
- [ ] Tray icon, per-OS installers (Windows MSI, macOS .app, Linux RPM/AppImage)
- [ ] Macro recorder/replayer, image-triggered actions, human-like motion
- [ ] Android keep-awake build

## License

Private, all rights reserved (for now).
