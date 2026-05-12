#!/bin/bash
set -e

cd "$(dirname "$0")/.."

APP_NAME="Synaptic.app"
APP_PATH="/Applications/$APP_NAME"
BUILD_DIR="src-tauri/target/release/bundle/macos"

echo "==> building synaptic locally..."
bun run tauri build --bundles app

if [ ! -d "$BUILD_DIR/$APP_NAME" ]; then
	echo "error: $APP_NAME not found after build."
	exit 1
fi

echo "==> killing running instance (if any)..."
killall -9 synaptic 2>/dev/null || true
killall -9 Synaptic 2>/dev/null || true
sleep 0.5

echo "==> removing old app (if any)..."
rm -rf "$APP_PATH"

echo "==> installing to /applications..."
ditto "$BUILD_DIR/$APP_NAME" "$APP_PATH"

echo "==> stripping gatekeeper quarantine..."
xattr -rd com.apple.quarantine "$APP_PATH"

echo "==> ad-hoc signing to appease gatekeeper..."
codesign --force --deep --sign - "$APP_PATH"

echo "==> done. synaptic is installed at $APP_PATH"
