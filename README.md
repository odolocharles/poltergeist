# 👻 Poltergeist

> A tiny, cross-platform ghost that keeps your machine awake and moves the cursor on its own, configured exactly how you want.

**Status:** early development · **Visibility:** private

---

## What it does

Poltergeist runs quietly in the background and gives you control over two things:

1. **Keep-awake:** stops the screen and system from sleeping or locking.
2. **Cursor movement:** nudges the pointer so idle timers, screensavers and activity monitors see a "live" session.

It was born as a test tool for virtual machines (where idle timeouts and screen blanking get in the way) and is growing into a small installable app for every desktop and phone.

## Features

- [ ] Modes: keep-awake only, cursor only, or both
- [ ] Movement styles: random, jitter, circle, scripted path
- [ ] Configurable interval range and max distance
- [ ] Pause automatically when the real user is active
- [ ] Global hotkey to toggle on/off
- [ ] System tray icon
- [ ] Start on boot
- [ ] Config file (JSON) shared across all platforms

## Platform support

| Platform | Keep-awake | Cursor movement | Notes |
|---|---|---|---|
| Linux (X11) | planned | planned | `enigo` backend |
| Linux (Wayland / console / VM) | planned | planned | `uinput` backend, needs `/dev/uinput` access |
| Windows | planned | planned | |
| macOS | planned | planned | needs Accessibility permission |
| Android | planned | n/a | wake lock only, there is no cursor |
| Huawei (EMUI / AppGallery) | planned | n/a | Android build; HarmonyOS NEXT needs a separate port |

## Architecture

- **Core:** Rust, with a platform-agnostic action engine and settings model
- **App shell:** [Tauri 2](https://tauri.app) with a small settings UI
- **Backends:** one per platform behind a common trait (`enigo`, `uinput`, `keepawake`)
- **Config:** a single JSON file; the same schema on every platform

```json
{
  "mode": "both",
  "movement": { "style": "random", "min_interval_s": 5, "max_interval_s": 30, "max_distance_px": 200 },
  "pause_when_user_active": true,
  "hotkey": "Ctrl+Alt+G",
  "start_on_boot": false
}
```

## Getting started (dev)

```bash
# prerequisites: Rust toolchain, Node.js
git clone <this-repo>
cd poltergeist
# scaffold coming soon
```

### Linux VM testing (uinput)

```bash
sudo dnf install python3-evdev
sudo modprobe uinput
sudo python3 prototypes/cursor_automator.py --min 3 --max 15
```

For custom kernels or Buildroot images, enable `CONFIG_INPUT_UINPUT`.

## Roadmap

1. Linux `uinput` prototype (VM testing)
2. Desktop MVP: Linux, Windows, macOS
3. Settings UI, tray icon, hotkeys
4. Android keep-awake build
5. Huawei / AppGallery packaging

## Privacy

No network access, no telemetry, no data collection. Poltergeist only talks to your operating system's input and power APIs.

## License

Private, all rights reserved (for now).
