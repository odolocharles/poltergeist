#!/usr/bin/env bash
# One-time setup so Poltergeist can send real input without running as root.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
sudo install -m644 "$here/99-poltergeist-uinput.rules" /etc/udev/rules.d/99-poltergeist-uinput.rules
echo uinput | sudo tee /etc/modules-load.d/poltergeist-uinput.conf >/dev/null
sudo modprobe uinput
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=misc --sysname-match=uinput
echo "Done. If the app still can't open /dev/uinput, log out and back in once."
