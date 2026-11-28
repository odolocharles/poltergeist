#!/usr/bin/env bash
# Builds poltergeist-VERSION-linux-x86_64.run: the installer header plus a tar.gz payload.
# Run after: cargo build --release -p poltergeist-gtk -p poltergeist-cli
# Optional: TARGET_DIR=/path/to/target  (defaults to ./target)
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
target="${TARGET_DIR:-target}/release"
id=org.sorabbyngo.Poltergeist
version="$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')"
out="poltergeist-${version}-linux-x86_64.run"

stage="$(mktemp -d)"; trap 'rm -rf "$stage"' EXIT
mkdir -p "$stage/bin" "$stage/share"
cp "$target/poltergeist-gtk" "$target/poltergeist" "$stage/bin/"
cp assets/poltergeist.svg "$stage/share/$id.svg"
cp assets/poltergeist.png "$stage/share/$id.png"
cp packaging/linux/$id.desktop packaging/linux/99-poltergeist-uinput.rules "$stage/share/"
tar czf "$stage/payload.tgz" -C "$stage" bin share

cat packaging/linux/installer-header.sh "$stage/payload.tgz" > "$out"
chmod +x "$out"
echo "Built $out ($(du -h "$out" | cut -f1))"
