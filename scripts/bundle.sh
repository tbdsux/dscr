#!/bin/sh
# Builds DScr in release mode and wraps it in an ad-hoc signed app bundle at target/DScr.app.
set -eu
cd "$(dirname "$0")/.."

cargo build --release

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
app=target/DScr.app
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp target/release/dscr "$app/Contents/MacOS/dscr"
cp assets/AppIcon.icns "$app/Contents/Resources/AppIcon.icns"

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
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF

plutil -lint -s "$app/Contents/Info.plist"
# Sign with the first real identity in the keychain so the Screen Recording grant survives
# rebuilds; an ad-hoc ("-") signature is pinned to the binary's hash and is re-prompted every build.
identity=${DSCR_SIGN_IDENTITY:-$(security find-identity -v -p codesigning | awk 'NR==1 && $2 ~ /^[0-9A-F]{40}$/ {print $2}')}
if [ -z "$identity" ]; then
  identity=-
  echo "warning: no signing identity found, using ad-hoc; Screen Recording will be re-requested after every rebuild" >&2
fi
codesign --force --sign "$identity" "$app"

echo "Built $app"
