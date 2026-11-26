#!/usr/bin/env bash
# Installs Poltergeist as a normal desktop app for the current user (no root needed):
# binaries in ~/.local/bin, an icon, and an entry in your application menu.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

cargo build --release -p poltergeist-app -p poltergeist-cli

bin="$HOME/.local/bin"
apps="$HOME/.local/share/applications"
icons="$HOME/.local/share/icons/hicolor"
install -Dm755 target/release/poltergeist-app "$bin/poltergeist-app"
install -Dm755 target/release/poltergeist     "$bin/poltergeist"
install -Dm644 assets/poltergeist.svg "$icons/scalable/apps/poltergeist.svg"
install -Dm644 assets/poltergeist.png "$icons/256x256/apps/poltergeist.png"
mkdir -p "$apps"
sed "s|@EXEC@|$bin/poltergeist-app|" packaging/linux/poltergeist.desktop.in > "$apps/poltergeist.desktop"
update-desktop-database "$apps" 2>/dev/null || true
gtk-update-icon-cache -f "$icons" 2>/dev/null || true

echo "Installed. Open your app menu and search for 'Poltergeist'."
case ":$PATH:" in *":$bin:"*) ;; *) echo "Note: add $bin to your PATH to run 'poltergeist' from a terminal." ;; esac
