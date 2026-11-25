# 👻 Poltergeist

> A tiny, **fully offline**, cross-platform ghost that keeps your machine awake and moves the cursor on its own, configured exactly how you want. Written in Rust.

**Status:** early development (v0.1, Linux backend first) · **Visibility:** private

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
core/        config, storage paths, backend traits, action engine (no OS code)
backends/    one module per OS behind cfg: linux (uinput + systemd-inhibit) done; others stubbed
cli/         `poltergeist` headless binary (what boot services run)
packaging/   systemd unit (more per-OS packaging to come)
```

A separate build is released per OS. Each backend sits behind `cfg(target_os)`, so a broken backend on one OS never affects another. CI builds Linux, Windows and macOS on every push.

## Usage

```bash
cargo build --release
./target/release/poltergeist run --dry-run          # print actions, no permissions needed
./target/release/poltergeist config                 # show config and its location
./target/release/poltergeist init-paths             # create the output folders
sudo ./target/release/poltergeist run --mode both --min 3 --max 15
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
  "min_interval_s": 5.0,
  "max_interval_s": 30.0,
  "max_distance_px": 200,
  "idle_key_chance": 0.3,
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
- [ ] Auto-click at a fixed spot (+ "pick a spot" overlay)
- [ ] Screenshots and screen recording (segmented, retention limits)
- [ ] Pause when the real user is active, panic hotkey
- [ ] Windows and macOS backends
- [ ] Pure-Rust settings UI (Dioxus or Slint), tray icon
- [ ] Macro recorder/replayer, image-triggered actions, human-like motion
- [ ] Android keep-awake build

## License

Private, all rights reserved (for now).
