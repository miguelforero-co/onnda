#!/usr/bin/env bash
# Re-sign the dev binary with microphone entitlements after each build.
# Run once after `npm run tauri dev` exits, or after `cargo build`:
#   bash scripts/sign-dev.sh
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MANIFEST="$SCRIPT_DIR/../src-tauri"
BINARY="$MANIFEST/target/debug/onnda"
ENTITLEMENTS="$MANIFEST/entitlements.plist"

if [ ! -f "$BINARY" ]; then
  echo "Binary not found at $BINARY — build first." >&2
  exit 1
fi

codesign --force --sign - --entitlements "$ENTITLEMENTS" "$BINARY"
echo "Signed: $BINARY"
