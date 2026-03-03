#!/bin/bash
set -e

APP_NAME="Terminaline"
APP_DIR="${APP_NAME}.app"
TARGET="aarch64-apple-darwin"

echo "🔨 Building release binary for $TARGET..."
cargo build --release --target "$TARGET"

echo "📦 Generating AppleScript application bundle..."
rm -rf "$APP_DIR"

# Create an AppleScript that dynamically resolves its own path
# This allows the .app to be moved anywhere (like Applications) and still find its embedded binary
cat << 'EOF' > launch.applescript
set app_path to POSIX path of (path to me)
set bin_path to app_path & "Contents/Resources/terminaline"
do shell script "open -a Terminal " & quoted form of bin_path
EOF

# Compile the AppleScript into a macOS .app bundle
osacompile -o "$APP_DIR" launch.applescript
rm launch.applescript

echo "� Injecting AppleEvents permissions into Info.plist..."
plutil -replace NSAppleEventsUsageDescription -string "Terminaline needs to control Terminal to display its user interface." "$APP_DIR/Contents/Info.plist"

echo "�🚚 Embedding binary into application bundle..."
cp "target/${TARGET}/release/terminaline" "$APP_DIR/Contents/Resources/"

if [ -f "icon.png" ]; then
    echo "🎨 Applying custom application icon..."
    mkdir -p Terminaline.iconset
    sips -z 16 16     icon.png --out Terminaline.iconset/icon_16x16.png > /dev/null
    sips -z 32 32     icon.png --out Terminaline.iconset/icon_16x16@2x.png > /dev/null
    sips -z 32 32     icon.png --out Terminaline.iconset/icon_32x32.png > /dev/null
    sips -z 64 64     icon.png --out Terminaline.iconset/icon_32x32@2x.png > /dev/null
    sips -z 128 128   icon.png --out Terminaline.iconset/icon_128x128.png > /dev/null
    sips -z 256 256   icon.png --out Terminaline.iconset/icon_128x128@2x.png > /dev/null
    sips -z 256 256   icon.png --out Terminaline.iconset/icon_256x256.png > /dev/null
    sips -z 512 512   icon.png --out Terminaline.iconset/icon_256x256@2x.png > /dev/null
    sips -z 512 512   icon.png --out Terminaline.iconset/icon_512x512.png > /dev/null
    sips -z 1024 1024 icon.png --out Terminaline.iconset/icon_512x512@2x.png > /dev/null
    
    # Overwrite the default AppleScript applet's icon with our custom icon
    iconutil -c icns Terminaline.iconset -o "$APP_DIR/Contents/Resources/applet.icns"
    rm -rf Terminaline.iconset
fi

echo "🔐 Code signing application..."
codesign --force --deep --sign - "$APP_DIR"

# Force macOS icon cache refresh
touch "$APP_DIR"
touch "$APP_DIR/Contents/Info.plist"

echo "✅ Done! Application bundle created at ${APP_DIR}."
echo "You can now double click Terminaline.app or drag it to your Applications folder."
