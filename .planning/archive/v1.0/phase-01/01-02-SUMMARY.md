---
phase: 01-redise-o-del-panel-de-configuraci-n-estilo-whisprflow
plan: 02
subsystem: build-system + window foundation
tags: [tauri-plugins, capabilities, window-config, deps, rust-modules]
requires: []
provides:
  - "880x640 resizable main window (config + fallback builder)"
  - "tauri-plugin-dialog/fs/updater registered (Rust + JS)"
  - "symphonia 0.6 + sha2 0.11 Rust crates"
  - "capabilities for dialog/fs/updater commands"
  - "empty module stubs: sounds, media_pause, audio_decode, data_mgmt, updater_check"
affects:
  - src-tauri/tauri.conf.json
  - src-tauri/Cargo.toml
  - src-tauri/capabilities/default.json
  - src-tauri/src/lib.rs
  - package.json
tech-stack:
  added:
    - "tauri-plugin-dialog 2.7.1"
    - "tauri-plugin-fs 2.5.1"
    - "tauri-plugin-updater 2.10.1"
    - "symphonia 0.6.0 (mp3/isomp4/aac/alac/wav/pcm)"
    - "sha2 0.11.0"
    - "@tauri-apps/plugin-dialog 2.7.1 (JS)"
    - "@tauri-apps/plugin-fs 2.5.1 (JS)"
    - "@tauri-apps/plugin-updater 2.10.1 (JS)"
  patterns:
    - "Tauri 2 plugin registration via .plugin(..._::init()) in builder chain"
    - "Empty-but-for-doc-comment Rust files compile as empty modules"
key-files:
  created:
    - src-tauri/src/sounds.rs
    - src-tauri/src/media_pause.rs
    - src-tauri/src/audio_decode.rs
    - src-tauri/src/data_mgmt.rs
    - src-tauri/src/updater_check.rs
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/tauri.conf.json
    - src-tauri/capabilities/default.json
    - src-tauri/src/lib.rs
    - package.json
decisions:
  - "Resolved real published versions via cargo add (RESEARCH versions confirmed accurate): dialog 2.7.1, fs 2.5.1, updater 2.10.1, symphonia 0.6.0, sha2 0.11.0"
metrics:
  duration: "~4 min"
  completed: "2026-06-14"
  tasks: 3
  files: 10
---

# Phase 01 Plan 02: Build-System + Window Foundation Summary

Grew the main window to 880×640 resizable, installed the dialog/fs/updater Tauri plugins (Rust + JS guest bindings) plus symphonia and sha2, granted their capabilities, registered the plugins in the Tauri builder chain, and declared five empty Rust module stubs so the tree compiles and Wave 2+ plans can drop in implementations.

## What Was Built

- **Task 1 (`af6ab03`)** — Added Rust deps via `cargo add` (tauri-plugin-dialog/fs/updater@2, sha2@0.11, symphonia@0.6 with mp3/isomp4/aac/alac/wav/pcm features) and JS guest bindings (`@tauri-apps/plugin-dialog/fs/updater`). Cleaned up Cargo.toml placement: plugins grouped with other tauri plugins, audio/integrity crates with explanatory comments.
- **Task 2 (`0e3eb76`)** — Main window grown from 480×600 fixed to 880×640 resizable in `tauri.conf.json`; `open_main_window` fallback builder in `lib.rs` synced to the same dimensions to avoid divergence on window recreate (PATTERNS GOTCHA). Widget window left untouched (340×120).
- **Task 3 (`9d466ec`)** — Registered the three plugins in the builder chain, declared `mod sounds/media_pause/audio_decode/data_mgmt/updater_check` with empty stub files (doc-comment only), and granted capabilities (`dialog:allow-open`, `fs:allow-read-file`, `fs:allow-write-file`, `fs:default`, `updater:default`). `cargo build` succeeds with all new plugins compiled.

## Required App Relaunch (project GOTCHA)

Window-size and new-plugin/capability changes take effect only on a **full app relaunch** of the rebuilt binary — NOT via `npm run dev` HMR. Downstream plans that call dialog/fs/updater commands must relaunch the built app to see the granted permissions and the 880×640 window. This was the explicit purpose of front-loading these changes in Wave 1.

## User Setup Required (updater signing — deferred)

`tauri-plugin-updater` performs minisign signature verification for signed updates. For real signed updates a keypair is needed:

```bash
cargo tauri signer generate -w ~/.tauri/voz-local.key
```

Store `TAURI_SIGNING_PRIVATE_KEY` as a CI secret and add the pubkey + update endpoints to `tauri.conf.json`. **Not required for Phase 1** — the plan ships a check-only fallback (RESEARCH Open Q3, D-14). The plugin is registered now so the command surface exists; signing infra can be wired when release distribution is set up.

## Deviations from Plan

### Verification adjustments (not code deviations)

**1. [Rule 3 - Blocking] Task 1 JS-import verify command was invalid**
- **Found during:** Task 1 verification
- **Issue:** Plan's verify used `node -e "require('@tauri-apps/plugin-dialog/package.json')"`, which fails with `ERR_PACKAGE_PATH_NOT_EXPORTED` — the package's `exports` field forbids resolving `./package.json`. This is a flaw in the verify command, not a problem with the install.
- **Fix:** Verified the same goal (package installed + importable) via `test -d node_modules/@tauri-apps/plugin-dialog` and an ESM `import('@tauri-apps/plugin-dialog')` probe, which succeeded ("dialog import ok").
- **Files modified:** none
- **Commit:** n/a (verification only)

**2. [Note] Task 2 acceptance grep is positionally inaccurate for this config**
- **Found during:** Task 2 verification
- **Issue:** Acceptance criterion `grep -A11 '"label": "main"' tauri.conf.json | grep '"resizable": true'` assumes `"label"` precedes the window properties, but in this config `label`/`url` come AFTER width/height/resizable. The grep returns the widget block instead.
- **Resolution:** Verified the substantive done criteria directly: the main window block reads width 880, height 640, resizable true; widget block unchanged at 340×120. The done criteria are met; only the positional grep is unreliable for this file ordering.
- **Files modified:** none

### Crate versions

RESEARCH-stated versions were confirmed accurate by `cargo add` resolution (no version-not-found errors): dialog 2.7.1, fs 2.5.1, updater 2.10.1, symphonia 0.6.0, sha2 0.11.0. No fallback resolution needed.

## Self-Check: PASSED

- FOUND: src-tauri/src/sounds.rs
- FOUND: src-tauri/src/media_pause.rs
- FOUND: src-tauri/src/audio_decode.rs
- FOUND: src-tauri/src/data_mgmt.rs
- FOUND: src-tauri/src/updater_check.rs
- FOUND commit: af6ab03 (Task 1)
- FOUND commit: 0e3eb76 (Task 2)
- FOUND commit: 9d466ec (Task 3)
- cargo build: exit 0
