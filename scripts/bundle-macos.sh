#!/usr/bin/env bash
#
# Build a macOS .app bundle of the egui_glass demo with a generated .icns icon.
#
# Usage:
#   ./scripts/bundle-macos.sh                     # uses assets/branding/EguiGlass.icns (or app-icon.png)
#   ./scripts/bundle-macos.sh path/to/icon.png    # custom square PNG (1024x1024 recommended)
#   ./scripts/bundle-macos.sh --install           # also copy the .app into /Applications
#
# Signing: by default the bundle is ad-hoc signed, which is fine for local use but
# makes Finder show "Apple could not verify ... is free of malware" on first launch
# (System Settings > Privacy & Security > Open Anyway, once). For a proper build set
#   CODESIGN_IDENTITY="Developer ID Application: <name> (<team>)"   # from `security find-identity -v -p codesigning`
#   NOTARY_PROFILE="<keychain profile>"                              # optional, from `xcrun notarytool store-credentials`
#
# macOS only: relies on sips + iconutil + codesign, all shipped with the OS.

set -euo pipefail

APP_NAME="Egui Glass"
BIN_NAME="egui_glass_demo"
BUNDLE_ID="com.heonny.egui-glass"

# Repo root = parent of this script's directory, regardless of where it's called from.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

INSTALL=0
SRC_ICON="assets/branding/app-icon.png"
PREBUILT_ICNS="assets/branding/EguiGlass.icns"
for arg in "$@"; do
  case "$arg" in
    --install) INSTALL=1 ;;
    *) SRC_ICON="$arg"; PREBUILT_ICNS="" ;;
  esac
done
APP_DIR="target/release/bundle/$APP_NAME.app"

# --- preflight ------------------------------------------------------------
[[ "$(uname)" == "Darwin" ]] || { echo "error: macOS only (needs sips/iconutil)" >&2; exit 1; }
for tool in sips iconutil codesign; do
  command -v "$tool" >/dev/null || { echo "error: '$tool' not found" >&2; exit 1; }
done
[[ -f "$PREBUILT_ICNS" || -f "$SRC_ICON" ]] || { echo "error: icon not found: $SRC_ICON" >&2; exit 1; }

VERSION="$(grep '^version' examples/demo/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"

# --- 1. build the release binary -----------------------------------------
echo "==> cargo build --release -p $BIN_NAME"
cargo build --release -p "$BIN_NAME"

# --- 2. AppIcon.icns: use the prebuilt one, else generate from the PNG ---
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
if [[ -f "$PREBUILT_ICNS" ]]; then
  echo "==> using $PREBUILT_ICNS"
  cp "$PREBUILT_ICNS" "$TMP/AppIcon.icns"
else
  echo "==> generating icon from $SRC_ICON"
  ICONSET="$TMP/AppIcon.iconset"
  mkdir -p "$ICONSET"
  for size in 16 32 128 256 512; do
    sips -z "$size" "$size"                 "$SRC_ICON" --out "$ICONSET/icon_${size}x${size}.png"    >/dev/null
    sips -z "$((size * 2))" "$((size * 2))" "$SRC_ICON" --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
  done
  iconutil -c icns "$ICONSET" -o "$TMP/AppIcon.icns"
fi

# --- 3. assemble the .app bundle -----------------------------------------
echo "==> assembling $APP_DIR"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources/asset"
cp "target/release/$BIN_NAME" "$APP_DIR/Contents/MacOS/$BIN_NAME"
chmod +x "$APP_DIR/Contents/MacOS/$BIN_NAME"
cp "$TMP/AppIcon.icns" "$APP_DIR/Contents/Resources/AppIcon.icns"
# The demo looks for its photos next to the executable (Contents/Resources/asset).
cp examples/asset/* "$APP_DIR/Contents/Resources/asset/"

cat > "$APP_DIR/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>              <string>$APP_NAME</string>
    <key>CFBundleDisplayName</key>       <string>$APP_NAME</string>
    <key>CFBundleIdentifier</key>        <string>$BUNDLE_ID</string>
    <key>CFBundleVersion</key>           <string>$VERSION</string>
    <key>CFBundleShortVersionString</key><string>$VERSION</string>
    <key>CFBundlePackageType</key>       <string>APPL</string>
    <key>CFBundleExecutable</key>        <string>$BIN_NAME</string>
    <key>CFBundleIconFile</key>          <string>AppIcon</string>
    <key>LSMinimumSystemVersion</key>    <string>11.0</string>
    <key>NSHighResolutionCapable</key>   <true/>
    <key>LSApplicationCategoryType</key> <string>public.app-category.developer-tools</string>
</dict>
</plist>
PLIST

# --- 4. sign: Developer ID (+ optional notarization) or ad-hoc -----------
if [[ -n "${CODESIGN_IDENTITY:-}" ]]; then
  echo "==> signing with $CODESIGN_IDENTITY"
  codesign --force --deep --options runtime --timestamp --sign "$CODESIGN_IDENTITY" "$APP_DIR"
  if [[ -n "${NOTARY_PROFILE:-}" ]]; then
    echo "==> notarizing"
    ditto -c -k --keepParent "$APP_DIR" "$TMP/app.zip"
    xcrun notarytool submit "$TMP/app.zip" --keychain-profile "$NOTARY_PROFILE" --wait
    xcrun stapler staple "$APP_DIR"
  fi
else
  echo "==> ad-hoc signing (local use; Finder will ask once: Privacy & Security > Open Anyway)"
  codesign --force --deep --sign - "$APP_DIR" >/dev/null 2>&1 \
    || echo "warn: ad-hoc codesign failed (bundle still usable locally)" >&2
fi

# --- 5. optional install into /Applications ------------------------------
if [[ "$INSTALL" == 1 ]]; then
  DEST="/Applications/$APP_NAME.app"
  echo "==> installing to $DEST"
  rm -rf "$DEST"
  cp -R "$APP_DIR" "$DEST"
  # Drop any quarantine flag so a locally built app is not treated as a download.
  xattr -dr com.apple.quarantine "$DEST" 2>/dev/null || true
fi

echo "==> done: $APP_DIR"
