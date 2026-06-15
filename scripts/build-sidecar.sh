#!/usr/bin/env bash
# Build the Apple SpeechAnalyzer ASR sidecar and place it in src-tauri/binaries/
# with the Rust target-triple suffix that tauri-plugin-shell expects.
#
# Run from the repo root:  bash scripts/build-sidecar.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SIDECAR_DIR="$ROOT/src-tauri/sidecar-asr"
OUT_DIR="$ROOT/src-tauri/binaries"
TRIPLE="$(rustc --print host-tuple)"   # e.g. aarch64-apple-darwin

echo "Building ASR sidecar (release)…"
( cd "$SIDECAR_DIR" && swift build -c release )

mkdir -p "$OUT_DIR"
cp -f "$SIDECAR_DIR/.build/release/asr" "$OUT_DIR/asr-$TRIPLE"
chmod +x "$OUT_DIR/asr-$TRIPLE"
echo "Placed: src-tauri/binaries/asr-$TRIPLE"
