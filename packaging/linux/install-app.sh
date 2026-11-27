#!/usr/bin/env bash
# Installs the native GTK app for the current user (no root needed): binaries in
# ~/.local/bin, icons, and an entry in your application menu. Builds from source.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

if ! pkg-config --exists gtk4 libadwaita-1 2>/dev/null; then
  echo "GTK4 / libadwaita development files are missing. On Fedora run:"
  echo "  sudo dnf install gcc pkgconf-pkg-config gtk4-devel libadwaita-devel"
  exit 1
fi

cargo build --release -p poltergeist-gtk -p poltergeist-cli

id=org.sorabbyngo.Poltergeist
bin="$HOME/.local/bin"; apps="$HOME/.local/share/applications"; icons="$HOME/.local/share/icons/hicolor"
install -Dm755 target/release/poltergeist-gtk "$bin/poltergeist-gtk"
install -Dm755 target/release/poltergeist     "$bin/poltergeist"
install -Dm644 assets/poltergeist.svg "$icons/scalable/apps/$id.svg"
install -Dm644 assets/poltergeist.png "$icons/256x256/apps/$id.png"
mkdir -p "$apps"
sed "s|^Exec=.*|Exec=$bin/poltergeist-gtk|" packaging/linux/$id.desktop > "$apps/$id.desktop"
rm -f "$apps/poltergeist.desktop"   # old entry from the earlier egui build, if present
update-desktop-database "$apps" 2>/dev/null || true
gtk-update-icon-cache -f "$icons" 2>/dev/null || true

echo "Installed. Open your app menu and search for 'Poltergeist'."
echo "For real mouse and click control, run once: ./packaging/linux/setup-uinput.sh"
case ":$PATH:" in *":$bin:"*) ;; *) echo "Note: add $bin to your PATH to run 'poltergeist' from a terminal." ;; esac
