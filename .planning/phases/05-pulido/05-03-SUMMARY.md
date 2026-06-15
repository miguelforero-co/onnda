---
phase: 05-pulido
plan: "03"
subsystem: rust-backend
tags: [refactor, modules, polish]
dependency_graph:
  requires: [05-02]
  provides: [paste.rs, models.rs, recording.rs]
  affects: [commands.rs, lib.rs, escape.rs, shortcut.rs]
tech_stack:
  added: []
  patterns: [pub(crate) module visibility, resolve_model_path deduplication]
key_files:
  created:
    - src-tauri/src/paste.rs
    - src-tauri/src/models.rs
    - src-tauri/src/recording.rs
  modified:
    - src-tauri/src/commands.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/escape.rs
    - src-tauri/src/shortcut.rs
decisions:
  - transcribe_file moved to recording.rs (owns all audio-pipeline logic) and registered as recording::transcribe_file in invoke_handler
  - ax_is_trusted moved to paste.rs as pub(crate); commands.rs calls crate::paste::ax_is_trusted
  - Recording internals (statics + state machine) in recording.rs with pub(crate) visibility; thin wrappers remain in commands.rs
  - IS_RECORDING made pub(crate) so escape.rs/shortcut.rs can call crate::recording::is_recording()
metrics:
  duration_seconds: 595
  completed_date: "2026-06-15"
  tasks_completed: 3
  tasks_total: 3
  files_created: 3
  files_modified: 4
---

# Phase 05 Plan 03: commands.rs Split — Summary

**One-liner:** Behavior-preserving split of the 1017-LOC `commands.rs` god-file into `paste.rs` (clipboard+CGEvent), `models.rs` (catalogue+download+`resolve_model_path`), and `recording.rs` (state machine+streaming+tail), with `commands.rs` slimmed to 198 LOC of thin wrappers.

## What Was Built

### Task 1: paste.rs + models.rs (commit f3e6c60)

- **`src-tauri/src/paste.rs`** — `paste_text` (pub(crate)), `write_clipboard_utf8`, `post_cmd_v`, `ax_is_trusted` (pub(crate)). All `#[cfg(target_os = "macos")]` guards preserved verbatim.
- **`src-tauri/src/models.rs`** — `ModelInfo`, `ModelStatus`, `models_dir` (pub(crate)), `get_models`, `download_model`, `check_model_status` (all `#[tauri::command]`), and the new `resolve_model_path` (pub(crate)).
- **`resolve_model_path(dir, model_name)`** — single canonical implementation of primary/fallback/exists logic previously duplicated 4× in commands.rs. Used by `check_model_status` inside models.rs, and called 3× from recording.rs.
- **4 unit tests** for `resolve_model_path`: neither-exists→None, only-fallback→Some(fallback), only-primary→Some(primary), both-exist→primary wins.
- `lib.rs` updated: `mod paste`, `mod models` added; `get_models/download_model/check_model_status` routed to `models::` in invoke_handler.
- `commands.rs`: `paste_text` call replaced with `crate::paste::paste_text`; all 3 inline primary/fallback blocks replaced with `crate::models::resolve_model_path`.

### Task 2: recording.rs + slim commands.rs (commit ed0dadd)

- **`src-tauri/src/recording.rs`** — All recording state machine code moved here:
  - Statics: `IS_RECORDING` (pub(crate)), `CAPTURE`, `COMMITTED_TEXT`, `COMMITTED_SAMPLES`, `STREAM_HANDLE`
  - `start_recording_internal` (pub(crate)) — audio capture + streaming loop + model warm
  - `stop_and_transcribe_internal` (pub(crate)) — tail assembly + commit stitching + paste
  - `cancel_recording_internal` (pub(crate)) — abort loop + discard audio
  - `is_recording` (pub(crate)) — atomic flag read
  - `transcribe_file` (#[tauri::command]) — file decode + resample + Whisper + history
  - 3 calls to `crate::models::resolve_model_path` replace 3 inline primary/fallback blocks (0 occurrences of `ggml-base.bin` in recording.rs)
- **`commands.rs`** slimmed from 1017 → 198 LOC: only thin wrappers for recording calls + permissions + settings + history + misc.
- `lib.rs`: `mod recording` added; `transcribe_file` routed to `recording::transcribe_file`.
- `escape.rs`: `crate::commands::is_recording` → `crate::recording::is_recording`; `crate::commands::cancel_recording_internal` → `crate::recording::cancel_recording_internal`.
- `shortcut.rs`: all 3 `crate::commands::*` recording calls → `crate::recording::*`.

### Task 3: End-to-end verification (this task — no code changes)

- `cargo build` → `Finished` (0 errors)
- `cargo test` → **53 passed, 0 failed** (was 49 pre-plan; +4 from resolve_model_path tests)
- `npm run check` → `0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS`
- Frontend (`src/`) not modified: `git diff --name-only HEAD | grep "^src/"` → UNCHANGED
- `ggml-base.bin` grep across src-tauri/src/ → found only inside `models.rs::resolve_model_path` (single definition)

## Command Registry — Final Module Locations

| Command | Module | Handler entry |
|---|---|---|
| get_settings | commands | commands::get_settings |
| save_settings | commands | commands::save_settings |
| start_recording | commands | commands::start_recording |
| stop_and_transcribe | commands | commands::stop_and_transcribe |
| transcribe_file | recording | recording::transcribe_file |
| is_recording_cmd | commands | commands::is_recording_cmd |
| get_models | models | models::get_models |
| download_model | models | models::download_model |
| check_model_status | models | models::check_model_status |
| check_mic_permission | commands | commands::check_mic_permission |
| check_accessibility_permission | commands | commands::check_accessibility_permission |
| open_accessibility_settings | commands | commands::open_accessibility_settings |
| open_microphone_settings | commands | commands::open_microphone_settings |
| get_history | commands | commands::get_history |
| delete_history_entry | commands | commands::delete_history_entry |
| correct_history_entry | commands | commands::correct_history_entry |
| get_recording_audio | commands | commands::get_recording_audio |
| hide_widget | commands | commands::hide_widget |
| test_paste | commands | commands::test_paste |
| get_build_hash | commands | commands::get_build_hash |
| get_app_version | commands | commands::get_app_version |
| reveal_data_dir | data_mgmt | data_mgmt::reveal_data_dir |
| clear_history | data_mgmt | data_mgmt::clear_history |
| clear_models | data_mgmt | data_mgmt::clear_models |
| get_storage_usage | data_mgmt | data_mgmt::get_storage_usage |
| check_for_updates | updater_check | updater_check::check_for_updates |

**Total: 26 commands** (25 original + 1 that was always data_mgmt/updater_check = all accounted for).

## Decisions Made

1. `transcribe_file` moved to `recording.rs` (not kept as wrapper in commands.rs) — it owns the full audio pipeline (decode→resample→Whisper→history), same as stop_and_transcribe_internal. Registered as `recording::transcribe_file` in invoke_handler.
2. `ax_is_trusted` placed in `paste.rs` as `pub(crate)` — it's a paste prerequisite (gating CGEvent calls), so it belongs with paste infrastructure. `commands.rs` (test_paste, check_accessibility_permission) calls `crate::paste::ax_is_trusted`.
3. `IS_RECORDING` marked `pub(crate)` so `escape.rs` and `shortcut.rs` don't need an extra function call layer — `crate::recording::is_recording()` is the public API, IS_RECORDING itself stays private to the module.

## Deviations from Plan

None — plan executed exactly as written. The three extraction targets (paste.rs, models.rs, recording.rs) were completed in sequence with build verification between each step.

## Known Stubs

None introduced. All moved code is the same behavior as before the refactor.

## Self-Check

- `src-tauri/src/paste.rs` exists: FOUND
- `src-tauri/src/models.rs` exists: FOUND
- `src-tauri/src/recording.rs` exists: FOUND
- Task 1 commit f3e6c60: FOUND
- Task 2 commit ed0dadd: FOUND
- `cargo build` → Finished: PASSED
- `cargo test` → 53 passed, 0 failed: PASSED
- `npm run check` → 0 errors, 0 warnings: PASSED
- `ggml-base.bin` only in models.rs: PASSED
- commands.rs 198 LOC (< 500): PASSED
- All 26 commands registered in invoke_handler: PASSED

## Self-Check: PASSED
