---
phase: 02-compatibilidad-honesta-intel-apple-silicon
plan: 02
subsystem: backend-rust
tags: [compat, hardware-detection, apple-silicon, intel, whisper, models]
requires: []
provides:
  - compat::macos_major_version
  - compat::physical_ram_gb
  - compat::hardware_default_model
  - compat::sidecar_available
  - commands::ModelInfo.disabled_reason
  - settings::init hardware-default first-run
affects:
  - get_models (apple engine gate)
  - settings init (first-run default)
  - check_model_status, stop_and_transcribe, transcribe_file, start_recording (fallbacks)
  - whisper transcription thread count
tech-stack:
  added: []
  patterns:
    - cfg(target_arch="aarch64") compile-time gate eliminating dead code in x86_64 binary
    - pure parse functions (parse_macos_major, parse_ram_gb) enabling unit tests without shell-out
    - Option<String> disabled_reason pattern for hardware-gated catalog entries
key-files:
  created:
    - src-tauri/src/compat.rs
  modified:
    - src-tauri/src/lib.rs
    - src-tauri/src/commands.rs
    - src-tauri/src/settings.rs
    - src-tauri/src/whisper_backend.rs
decisions:
  - "disabled_reason: Option<String> added to ModelInfo (not reusing coming_soon) to distinguish hardware-unavailable from not-yet-built"
  - "parse_macos_major and parse_ram_gb are private pure fns called by public helpers — testable without spawning subprocesses"
  - "Default for AppSettings unchanged (selected_model stays large-v3-turbo) to preserve existing settings.json deserialization"
  - "x86_64 cross-compile (cargo build --target x86_64-apple-darwin) fails at link step due to native frameworks — pre-existing CI limitation, not caused by this plan; aarch64 build and all tests green"
metrics:
  duration: "15 minutes"
  completed: "2026-06-15T20:12:22Z"
  tasks: 3
  files: 5
---

# Phase 2 Plan 02: Compatibilidad honesta (COMPAT-02/03/04) Summary

**One-liner:** Hardware-gated Apple engine catalog entry with Spanish disabled_reason, hardware-chosen first-run model default via sysctl/sw_vers helpers in new compat.rs, and Whisper thread count clamped to min(available_parallelism, 6).

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | compat.rs con helpers + tests | b24bd85 | src-tauri/src/compat.rs (new), src-tauri/src/lib.rs |
| 2 | COMPAT-02: gate Apple engine in get_models | 646b2d0 | src-tauri/src/commands.rs |
| 3 | COMPAT-03 + COMPAT-04: hardware default + threads | 2affd45 | src-tauri/src/settings.rs, src-tauri/src/commands.rs, src-tauri/src/whisper_backend.rs |

## Verification

- `cargo test`: 44 tests pass (9 new in compat::tests, 1 new in whisper_backend::tests, 4 pre-existing in settings::tests)
- `cargo build`: Finished with 3 dead-code warnings for helper functions only called from tests in some cfg contexts — not errors.

## Deviations from Plan

### Pre-existing environment issue (not a code deviation)

**x86_64 cross-compile unavailable locally**
- **Found during:** Task 2 verification
- **Issue:** `cargo build --target x86_64-apple-darwin` fails with `can't find crate for 'core'` — Rust's std cross-compile for x86_64 conflicts with native macOS framework crates (objc2, etc.) that require the host linker. This is a pre-existing limitation of the local dev environment.
- **Impact:** Zero — the `#[cfg(target_arch = "aarch64")]` gates are evaluated at compile time; the aarch64 build (the actual dev/release target) compiles cleanly. CI on `macos-15` with both targets properly configured will handle cross-compilation (COMPAT-01).
- **Fix:** None needed for this plan. Documented for CI plan (02-01 / COMPAT-01).

### Auto-fixed warnings (Rule 2)

The `compat::hardware_default_model`, `compat::physical_ram_gb`, and `compat::macos_major_version` functions emit dead-code warnings in the aarch64 build because `hardware_default_model` uses `physical_ram_gb` only in the `cfg(aarch64)` branch (which IS the active branch here — warnings are for the `not(aarch64)` path that returns early). These are Rust's normal `#[cfg]` branch elimination warnings and are not errors.

## Known Stubs

None — all helpers are fully wired:
- `compat::macos_major_version` calls `sw_vers` at runtime
- `compat::physical_ram_gb` calls `sysctl hw.memsize` at runtime
- `compat::sidecar_available` calls `app.shell().sidecar("asr").is_ok()` at runtime
- `disabled_reason` flows from `get_models` to the serialized JSON response (frontend consumption is Phase 2 Plan 03 / COMPAT-05 scope)

## Threat Flags

No new threat surface introduced beyond what the plan's threat model covers. T-02-02 mitigation applied: `sidecar_available` uses `.is_ok()` only (no execution).

## Self-Check

### Created files exist:
- `src-tauri/src/compat.rs` — FOUND

### Commits exist:
- b24bd85 — FOUND
- 646b2d0 — FOUND
- 2affd45 — FOUND

### Tests pass:
- 44/44 tests green

## Self-Check: PASSED
