#!/usr/bin/env bash
set -euo pipefail
id=org.sorabbyngo.Poltergeist
rm -f "$HOME/.local/bin/poltergeist-gtk" "$HOME/.local/bin/poltergeist" \
      "$HOME/.local/share/applications/$id.desktop" \
      "$HOME/.local/share/icons/hicolor/scalable/apps/$id.svg" \
      "$HOME/.local/share/icons/hicolor/256x256/apps/$id.png"
update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
echo "Poltergeist removed. Your settings stay in ~/.config/poltergeist."
