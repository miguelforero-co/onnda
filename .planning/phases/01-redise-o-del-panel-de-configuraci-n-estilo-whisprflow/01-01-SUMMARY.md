---
phase: 01-redise-o-del-panel-de-configuraci-n-estilo-whisprflow
plan: 01
subsystem: persisted-data
tags: [settings, history, serde-migration, tdd]
dependency_graph:
  requires: []
  provides:
    - "AppSettings.sound_on_listen / sound_on_stop / sound_on_cancel (bool, default false)"
    - "AppSettings.pause_media (bool, default false)"
    - "AppSettings.dictionary (Vec<String>) + idempotent migrate_dictionary()"
    - "HistoryEntry.source (String, default \"dictation\") + original_filename (Option<String>)"
    - "history::save_entry(source, original_filename) signature"
  affects:
    - "src-tauri/src/commands.rs (existing dictation call site)"
tech_stack:
  added: []
  patterns:
    - "serde #[serde(default)] non-breaking field migration"
    - "idempotent free-function migration (derive-once when target empty)"
    - "verbatim reuse of transcription.rs CSV split for dictionary derivation"
key_files:
  created: []
  modified:
    - "src-tauri/src/settings.rs"
    - "src-tauri/src/history.rs"
    - "src-tauri/src/commands.rs"
decisions:
  - "Kept custom_words as the live source of truth for correct_words; dictionary derived from it (D-19/D-20)"
  - "Migration runs only on the disk-read path in load(); cache path already holds migrated value"
metrics:
  duration: "~2.5 min"
  tasks: 2
  files: 3
  completed: "2026-06-14"
---

# Phase 01 Plan 01: Persisted-Data Foundation Summary

Extended `AppSettings` (3 sound flags, pause-media, dictionary Vec + idempotent CSV migration) and `HistoryEntry` (source + original_filename) using serde defaults so existing `settings.json` / `history.json` files load unchanged. TDD: 6 new tests prove old-file deserialization, migration idempotency, and the dictation default.

## What Was Built

**Task 1 — AppSettings:** Added `sound_on_listen`, `sound_on_stop`, `sound_on_cancel`, `pause_media` (all `bool`, `#[serde(default)]` → false) and `dictionary: Vec<String>` (`#[serde(default)]` → empty). Added `migrate_dictionary(dictionary, custom_words)`, a free function that derives discrete vocabulary items from the legacy comma/newline CSV ONLY when `dictionary` is still empty (idempotent), reusing the exact split from `transcription.rs::correct_words`. Wired into `load()` on the disk-read path. `custom_words` is preserved (still consumed by the Whisper backend / `correct_words`).

**Task 2 — HistoryEntry:** Added `source: String` (`#[serde(default = "default_source")]` → `"dictation"`) and `original_filename: Option<String>` (`#[serde(default)]`). `save_entry` now takes `source` + `original_filename` params and writes them into the entry literal. The single existing call site in `commands.rs::stop_and_transcribe_internal` passes `"dictation".to_string(), None`.

## Tasks

| Task | Name | RED Commit | GREEN Commit | Files |
| ---- | ---- | ---------- | ------------ | ----- |
| 1 | Extend AppSettings + dictionary migration | abcfab8 | 1f468c6 | src-tauri/src/settings.rs |
| 2 | Add source/original_filename to HistoryEntry | aba14ef | 483074d | src-tauri/src/history.rs, src-tauri/src/commands.rs |

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml` → 12 passed, 0 failed (6 new + 6 pre-existing, no regressions).
- `cargo build --manifest-path src-tauri/Cargo.toml` → succeeds (updated call site compiles).
- New tests: `settings_new_fields_default_false`, `dictionary_migration_from_csv`, `dictionary_migration_idempotent`, `dictionary_join_for_prompt`, `old_history_entry_defaults_to_dictation`, `default_source_constant`.

## TDD Gate Compliance

Both tasks followed RED → GREEN. Each has a `test(...)` commit with a failing stub (migrate_dictionary returns empty; default_source returns "STUB") followed by a `feat(...)` commit with the real implementation. No REFACTOR commit needed — implementations are minimal and mirror existing verbatim patterns.

## Threat Model Compliance

- T-01-01 (tampering, missing/extra fields): mitigated — every new field uses `#[serde(default)]`; `load()` retains `unwrap_or_default()`.
- T-01-02 (migration clobbering user edits): mitigated — `migrate_dictionary` only derives when `dictionary` is empty; proven by `dictionary_migration_idempotent`.
- T-01-03 (large legacy CSV): accepted — local single-user file, O(n) split.

## Deviations from Plan

None - plan executed exactly as written.

## Notes

Two pre-existing untracked planning docs (`01-PATTERNS.md`, `01-RESEARCH.md`) are present in the phase directory; they were not created by this plan and were left untouched.

## Self-Check: PASSED

All modified files exist; all 4 task commits (abcfab8, 1f468c6, aba14ef, 483074d) present in git history.
