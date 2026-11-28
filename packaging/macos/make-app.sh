#!/usr/bin/env bash
# Builds Poltergeist.app (universal: Apple silicon + Intel) and a Poltergeist-VERSION.dmg disk image.
# Run on a Mac after building both targets:
#   cargo build --release -p poltergeist-app -p poltergeist-cli --target aarch64-apple-darwin
#   cargo build --release -p poltergeist-app -p poltergeist-cli --target x86_64-apple-darwin
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
version="$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')"
app="dist/Poltergeist.app"

rm -rf dist && mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
for bin in poltergeist-app poltergeist; do
  lipo -create -output "$app/Contents/MacOS/$bin" \
    "target/aarch64-apple-darwin/release/$bin" "target/x86_64-apple-darwin/release/$bin"
done
cp assets/poltergeist.icns "$app/Contents/Resources/Poltergeist.icns"

cat > "$app/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Poltergeist</string>
  <key>CFBundleDisplayName</key><string>Poltergeist</string>
  <key>CFBundleIdentifier</key><string>org.sorabbyngo.Poltergeist</string>
  <key>CFBundleExecutable</key><string>poltergeist-app</string>
  <key>CFBundleIconFile</key><string>Poltergeist</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>${version}</string>
  <key>CFBundleVersion</key><string>${version}</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.utilities</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# Ad-hoc signature: required for the app to launch on Apple silicon. It is not an
# Apple Developer ID, so Gatekeeper still asks the user to confirm the first launch.
codesign --force --deep --sign - "$app"
# Disk image: the app next to an Applications shortcut, so install is "drag across".
stage="dist/dmg"; mkdir -p "$stage"
cp -R "$app" "$stage/"
ln -s /Applications "$stage/Applications"
hdiutil create -volname "Poltergeist" -srcfolder "$stage" -ov -format UDZO "Poltergeist-${version}.dmg"
echo "Built Poltergeist-${version}.dmg"
