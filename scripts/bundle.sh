#!/bin/sh
# Builds dist/Happy-Kitten.app (release build, icon from assets/logo.png, ad-hoc signed).
set -eu
cd "$(dirname "$0")/.."
cargo build --release -p happy-kitten

APP="dist/Happy-Kitten.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/happy-kitten "$APP/Contents/MacOS/Happy-Kitten"

ICONSET="$(mktemp -d)/HappyKitten.iconset"
mkdir -p "$ICONSET"
for size in 16 32 128 256 512; do
  sips -z "$size" "$size" assets/logo.png --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
  double=$((size * 2))
  sips -z "$double" "$double" assets/logo.png --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/HappyKitten.icns"

VERSION="$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')"
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Happy-Kitten</string>
  <key>CFBundleDisplayName</key><string>Happy-Kitten</string>
  <key>CFBundleIdentifier</key><string>local.happy-kitten.editor</string>
  <key>CFBundleExecutable</key><string>Happy-Kitten</string>
  <key>CFBundleIconFile</key><string>HappyKitten</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>${VERSION}</string>
  <key>CFBundleVersion</key><string>${VERSION}</string>
  <key>NSHumanReadableCopyright</key><string>© 2026 Gottpot</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.video</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
  <key>CFBundleDocumentTypes</key>
  <array>
    <dict>
      <key>CFBundleTypeName</key><string>Happy-Kitten-Projekt</string>
      <key>CFBundleTypeExtensions</key><array><string>kitten</string></array>
      <key>CFBundleTypeRole</key><string>Editor</string>
    </dict>
  </array>
</dict>
</plist>
PLIST

codesign --force --deep --sign - "$APP" >/dev/null 2>&1 || echo "Hinweis: Signieren übersprungen"
echo "Fertig: $APP"
