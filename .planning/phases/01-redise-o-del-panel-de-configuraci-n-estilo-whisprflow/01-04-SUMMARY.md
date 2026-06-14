---
phase: 01-redise-o-del-panel-de-configuraci-n-estilo-whisprflow
plan: 04
subsystem: native-feedback
tags: [macos, objc2, cgevent, nssound, media-key, whisper-models]
requires:
  - "01-01 (settings flags: sound_on_listen/stop/cancel, pause_media)"
  - "01-02 (empty sounds.rs / media_pause.rs stubs + module decls)"
provides:
  - "sounds::play_listen/play_stop/play_cancel (NSSound, gated)"
  - "media_pause::pause_if_playing/resume_if_paused (CGEvent media key, symmetric)"
  - "expanded Whisper model catalog (base/small/medium/large-v3-turbo + Parakeet coming_soon)"
  - "ModelInfo.coming_soon: bool field"
affects:
  - "src/lib/types.ts ModelInfo (frontend, handled in 01-03/01-07 — coming_soon must be added there)"
tech-stack:
  added: []
  patterns:
    - "objc2 0.6 AnyClass::get + msg_send! native calls with #[cfg(macos)] + no-op twin"
    - "NSSystemDefined CGEvent for synthetic media keys (NX_KEYTYPE_PLAY)"
key-files:
  created: []
  modified:
    - src-tauri/src/sounds.rs
    - src-tauri/src/media_pause.rs
    - src-tauri/src/commands.rs
decisions:
  - "Sounds via NSSound soundNamed: system cues (Tink/Pop/Funk) — no asset bundling, reliable while window hidden"
  - "Pause-media is a symmetric Play/Pause toggle (no public play-state query); I_PAUSED AtomicBool keeps one pause ↔ one resume"
  - "ModelInfo gained coming_soon: bool; Parakeet listed but non-functional (falls through download_model error arm)"
metrics:
  duration: ~15m
  completed: 2026-06-14
  tasks: 3
  files: 3
---

# Phase 01 Plan 04: Native feedback (sounds + pause-media) & expanded model catalog Summary

NSSound start/stop/cancel cues and a symmetric CGEvent Play/Pause media toggle wired into the recording state machine at its three existing emit points, each gated on its opt-in setting; the Whisper catalog now lists base/small/medium/large-v3-turbo plus a non-functional Parakeet "coming soon" card.

## What Was Built

- **`sounds.rs`** — `play_listen()` (Tink), `play_stop()` (Pop), `play_cancel()` (Funk) via `NSSound soundNamed:` through the objc2 runtime (`AnyClass::get(c"NSSound")` — no extra app-kit feature needed). Async, non-blocking (`NSSound play` returns immediately, satisfying threat T-01-10). No-op twin for non-macOS.
- **`media_pause.rs`** — `pause_if_playing()` / `resume_if_paused()`. Sends the hardware Play/Pause key as an `NSSystemDefined` CGEvent (event type 14, subtype 8, `NX_KEYTYPE_PLAY` = 16, down+up), built via `NSEvent otherEventWithType:...` and posted with `CGEventPost`. A module-level `static I_PAUSED: AtomicBool` ensures we only resume when WE paused (threat T-01-08). No-op twin for non-macOS.
- **`commands.rs` hooks** — at start (`start_recording_internal`, after the recording-state emit), stop (`stop_and_transcribe_internal`), and cancel (`cancel_recording_internal`): load settings once and fire `play_*` + `pause/resume_if_paused`, each guarded by its flag (`sound_on_listen/stop/cancel`, `pause_media`).
- **Model catalog** — `get_models` returns base, small (466 MB), medium (1536 MB), large-v3-turbo, plus a Parakeet entry with `coming_soon: true`. `ModelInfo` gained a `coming_soon: bool` field. `download_model` resolves `ggml-small.bin` / `ggml-medium.bin` URLs; `parakeet` intentionally falls through to the "Modelo desconocido" error arm (D-13).

## Decisions Made

- **NSSound system cues over bundled assets.** Dependency-free, reliable even when the main window/widget is hidden (D-07, PATTERNS authoritative correction that WebView audio is unreliable when hidden). Three distinct short cues: Tink / Pop / Funk.
- **Symmetric Play/Pause toggle for pause-media.** There is no public API to query "is something playing" (private MediaRemote rejected in RESEARCH). Behavior: toggle once on start, toggle back on stop/cancel. If nothing was playing, the user gets a harmless no-op pause/resume pair. Gated behind opt-in `pause_media`, so it only runs when chosen. `I_PAUSED` guards against desync.
- **`coming_soon` field on ModelInfo** — backend now emits it for every model. The frontend `ModelInfo` TS interface in `src/lib/types.ts` must mirror this field; that wiring belongs to 01-03/01-07 (frontend plans), not this backend plan.

## Deviations from Plan

None functional — plan executed as written. Minor: in `get_models` the catalog is ordered base → small → medium → large-v3-turbo → parakeet (ascending size, ending with coming-soon), a clearer ordering than the original base-after-turbo; no behavioral impact.

The plan's note offered an NSEvent-vs-CGEvent choice for the media key; I used the NSEvent `otherEventWithType:` builder + `CGEvent` extraction + `CGEventPost` path (the documented reliable approach), and confirmed the objc2 multi-argument selector compiles (RESEARCH A7 verification satisfied).

## Verification

- `cargo build --manifest-path src-tauri/Cargo.toml` — exits 0, no warnings.
- All acceptance-criteria greps pass (sounds hooks, media hooks, small/medium + ggml-medium.bin URL, coming_soon + parakeet).
- Audible/media-effect verification (sounds firing, media pausing/resuming) is non-headless-testable by design; covered by the 01-09 manual checklist.

## Self-Check: PASSED

- FOUND: src-tauri/src/sounds.rs (play_listen/stop/cancel + no-op twin)
- FOUND: src-tauri/src/media_pause.rs (AtomicBool + no-op twin)
- FOUND: src-tauri/src/commands.rs (3 hooks + expanded catalog + coming_soon)
- FOUND: bec41fc, caaf2ce, 58aef2c
