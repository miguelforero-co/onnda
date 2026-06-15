#!/usr/bin/env bash
# Build the Apple SpeechAnalyzer ASR sidecar for BOTH target triples and place
# the binaries in src-tauri/binaries/ with the Rust target-triple suffix that
# tauri-plugin-shell expects.
#
# Run from the repo root:  bash scripts/build-sidecar.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SIDECAR_DIR="$ROOT/src-tauri/sidecar-asr"
OUT_DIR="$ROOT/src-tauri/binaries"

# ── arm64 (native) ────────────────────────────────────────────────────────────
echo "Building ASR sidecar (arm64, release)…"
( cd "$SIDECAR_DIR" && swift build -c release )
mkdir -p "$OUT_DIR"
cp -f "$SIDECAR_DIR/.build/arm64-apple-macosx/release/asr" "$OUT_DIR/asr-aarch64-apple-darwin"
chmod +x "$OUT_DIR/asr-aarch64-apple-darwin"
echo "Placed: src-tauri/binaries/asr-aarch64-apple-darwin"

# ── x86_64 (cross-compiled from arm64) ────────────────────────────────────────
echo "Building ASR sidecar (x86_64 cross-compiled, release)…"
( cd "$SIDECAR_DIR" && swift build -c release --triple x86_64-apple-macosx11.0 )
cp -f "$SIDECAR_DIR/.build/x86_64-apple-macosx/release/asr" "$OUT_DIR/asr-x86_64-apple-darwin"
chmod +x "$OUT_DIR/asr-x86_64-apple-darwin"
echo "Placed: src-tauri/binaries/asr-x86_64-apple-darwin"
