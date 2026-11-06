#!/bin/sh
# Builds DScr in release mode and wraps it in an ad-hoc signed app bundle at target/DScr.app.
set -eu
cd "$(dirname "$0")/.."

cargo build --release

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
app=target/DScr.app
rm -rf "$app"
mkdir -p "$app/Contents/MacOS"
cp target/release/dscr "$app/Contents/MacOS/dscr"

# LSUIElement keeps it out of the Dock from the very first frame; the app is menu-bar only.
cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>DScr</string>
  <key>CFBundleDisplayName</key><string>DScr</string>
  <key>CFBundleIdentifier</key><string>dev.tbdsux.dscr</string>
  <key>CFBundleExecutable</key><string>dscr</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF

plutil -lint -s "$app/Contents/Info.plist"
# Ad-hoc signature ("-"): no Apple Developer account needed, and macOS can track the
# Screen Recording permission against the bundle.
codesign --force --sign - "$app"

echo "Built $app"
