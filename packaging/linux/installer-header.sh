#!/usr/bin/env bash
# Poltergeist installer for Linux. One file, no compiler needed.
#   chmod +x poltergeist-*-linux-x86_64.run && ./poltergeist-*-linux-x86_64.run
# Options: --uninstall   --no-input-setup   --help
set -euo pipefail

ID=org.sorabbyngo.Poltergeist
BIN="$HOME/.local/bin"
APPS="$HOME/.local/share/applications"
ICONS="$HOME/.local/share/icons/hicolor"
SETUP_INPUT=ask

for arg in "$@"; do
  case "$arg" in
    --no-input-setup) SETUP_INPUT=no ;;
    --uninstall)
      rm -f "$BIN/poltergeist-gtk" "$BIN/poltergeist" "$APPS/$ID.desktop" \
            "$ICONS/scalable/apps/$ID.svg" "$ICONS/256x256/apps/$ID.png"
      update-desktop-database "$APPS" 2>/dev/null || true
      echo "Poltergeist removed. Your settings stay in ~/.config/poltergeist."
      echo "To also remove the mouse-control permission: sudo rm /etc/udev/rules.d/99-poltergeist-uinput.rules /etc/modules-load.d/poltergeist-uinput.conf"
      exit 0 ;;
    -h|--help)
      sed -n '2,4p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "Unknown option: $arg (try --help)" >&2; exit 2 ;;
  esac
done

if [ "$(uname -m)" != "x86_64" ]; then
  echo "This installer is for 64-bit Intel/AMD Linux (x86_64). This machine is $(uname -m)." >&2
  exit 1
fi

# The app window needs GTK4 and libadwaita at run time (not the -devel packages).
# This is only a warning: the command-line tool works without them.
missing=""
if command -v ldconfig >/dev/null 2>&1; then
  libs="$(ldconfig -p 2>/dev/null || true)"   # capture first: `grep -q` would break the pipe under pipefail
  case "$libs" in *libgtk-4.so*) ;; *) missing="gtk4" ;; esac
  case "$libs" in *libadwaita-1.so*) ;; *) missing="$missing libadwaita" ;; esac
fi
WARN_LIBS=""
if [ -n "${missing// /}" ]; then
  WARN_LIBS="$missing"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
start="$(awk '/^__PAYLOAD_BELOW__$/ { print NR + 1; exit }' "$0")"
tail -n +"$start" "$0" | tar xz -C "$tmp"

install -Dm755 "$tmp/bin/poltergeist-gtk" "$BIN/poltergeist-gtk"
install -Dm755 "$tmp/bin/poltergeist"     "$BIN/poltergeist"
install -Dm644 "$tmp/share/$ID.svg" "$ICONS/scalable/apps/$ID.svg"
install -Dm644 "$tmp/share/$ID.png" "$ICONS/256x256/apps/$ID.png"
mkdir -p "$APPS"
sed "s|^Exec=.*|Exec=$BIN/poltergeist-gtk|" "$tmp/share/$ID.desktop" > "$APPS/$ID.desktop"
update-desktop-database "$APPS" 2>/dev/null || true
gtk-update-icon-cache -f "$ICONS" 2>/dev/null || true

echo "Installed Poltergeist."
if [ -n "$WARN_LIBS" ]; then
  echo "Warning: the app window needs these libraries, which were not found:$WARN_LIBS"
  echo "  Fedora:        sudo dnf install gtk4 libadwaita"
  echo "  Debian/Ubuntu: sudo apt install libgtk-4-1 libadwaita-1-0"
  echo "  Arch:          sudo pacman -S gtk4 libadwaita"
  echo "  openSUSE:      sudo zypper install libgtk-4-1 libadwaita-1-0"
fi
echo "  App:      open your app menu and search for 'Poltergeist' (or run: poltergeist-gtk)"
echo "  Command:  poltergeist run --help"
case ":$PATH:" in *":$BIN:"*) ;; *) echo "  Note: add $BIN to your PATH to use the 'poltergeist' command." ;; esac

# Real mouse and click control needs permission to create a virtual input device.
if [ -w /dev/uinput ] 2>/dev/null; then
  echo "Mouse and click control: already allowed."
elif [ "$SETUP_INPUT" = ask ] && [ -t 0 ] && [ -t 1 ]; then
  printf "Allow real mouse and click control? This needs your password once. [Y/n] "
  read -r ans || ans=n
  case "${ans:-y}" in [Nn]*) SETUP_INPUT=no ;; *) SETUP_INPUT=yes ;; esac
fi
if [ "$SETUP_INPUT" = yes ]; then
  sudo install -Dm644 "$tmp/share/99-poltergeist-uinput.rules" /etc/udev/rules.d/99-poltergeist-uinput.rules
  echo uinput | sudo tee /etc/modules-load.d/poltergeist-uinput.conf >/dev/null
  sudo modprobe uinput || true
  sudo udevadm control --reload-rules || true
  sudo udevadm trigger --subsystem-match=misc --sysname-match=uinput || true
  echo "Done. Log out and back in once so your session picks up the permission."
elif [ ! -w /dev/uinput ] 2>/dev/null; then
  echo "Without that permission the app runs in simulation. To allow it later, run this installer again."
fi
exit 0
__PAYLOAD_BELOW__
