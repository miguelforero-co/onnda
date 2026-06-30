#!/usr/bin/env bash
# Genera un DMG con diseño (create-dmg) a partir de un .app YA firmado+notarizado+stapled.
set -euo pipefail
APP="$1"   # ruta al onnda.app sellado
OUT="$2"   # ruta de salida del .dmg
BG="$(cd "$(dirname "$0")/.." && pwd)/src-tauri/icons/dmg-background.png"
rm -f "$OUT"
create-dmg \
  --volname "onnda" \
  --background "$BG" \
  --window-pos 200 120 \
  --window-size 660 400 \
  --icon-size 120 \
  --icon "onnda.app" 170 190 \
  --app-drop-link 490 190 \
  --hide-extension "onnda.app" \
  --no-internet-enable \
  "$OUT" "$APP"
