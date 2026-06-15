---
phase: 01-redise-o-del-panel-de-configuraci-n-estilo-whisprflow
plan: 06
subsystem: infra
tags: [tauri, updater, data-management, rust, filesystem]

requires:
  - phase: 01-02
    provides: tauri-plugin-updater registered + data_mgmt/updater_check stub modules
  - phase: 01-01
    provides: app_data_dir layout (history.json, recordings/, models/)
  - phase: 01-05
    provides: file-transcription history/audio storage conventions
provides:
  - reveal_data_dir command (opens app_data_dir in Finder, macOS)
  - clear_history command (deletes history.json + recordings, recreates empty dir)
  - clear_models command (deletes only *.bin under app_data_dir/models)
  - check_for_updates command (UpdateStatus with check-only no-endpoint fallback)
affects: [01-07, 01-08, 01-09]

tech-stack:
  added: []
  patterns:
    - "Data-deletion commands take NO frontend path argument — all paths derived from app_data_dir (no traversal)"
    - "Model deletion scoped to *.bin files only"
    - "Updater check-only fallback: check() errors surface as benign up_to_date + captured error (no hard failure)"

key-files:
  created: []
  modified:
    - src-tauri/src/data_mgmt.rs
    - src-tauri/src/updater_check.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "Data-deletion commands never accept a path from the frontend; paths derived from app_data_dir only (no path traversal possible)"
  - "clear_models restricted to *.bin so it cannot delete unrelated files"
  - "check_for_updates is check-only this phase; missing updater endpoint/keypair surfaces as benign 'up to date' with error captured (D-14 fallback); real endpoint + signing keypair deferred to user_setup"

patterns-established:
  - "App-data-scoped destructive commands: derive path from app_data_dir(), never from args"
  - "UpdateStatus shape: { up_to_date, available_version, current_version, error }"

requirements-completed: [Gestion-de-datos, Check-for-updates]

duration: 2min
completed: 2026-06-14
---

# Phase 01 Plan 06: Data Management + Check for Updates Summary

**App-data-scoped reveal/clear-history/clear-models commands plus a check-only `check_for_updates` (UpdateStatus) with a graceful no-endpoint fallback, all registered in lib.rs.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-06-14T18:03:55Z
- **Completed:** 2026-06-14T18:05:29Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- `reveal_data_dir` opens the app data directory in Finder (macOS) via `open`
- `clear_history` deletes `history.json` and the entire `recordings/` dir, then recreates an empty `recordings/` so `history::init`'s invariant holds
- `clear_models` deletes only `*.bin` files under `app_data_dir/models` — unrelated files are never touched
- `check_for_updates` returns `UpdateStatus { up_to_date, available_version, current_version, error }`, gracefully treating a missing updater endpoint/keypair as "up to date" with the error captured
- All four commands registered in `lib.rs` `generate_handler!`

## Task Commits

1. **Task 1: Implement data_mgmt.rs (reveal, clear history+audio, clear models)** - `6eacf1b` (feat)
2. **Task 2: Implement updater_check.rs + register all 4 commands** - `0766189` (feat)

## Files Created/Modified
- `src-tauri/src/data_mgmt.rs` - reveal_data_dir, clear_history, clear_models commands (all app_data_dir-scoped)
- `src-tauri/src/updater_check.rs` - UpdateStatus struct + check_for_updates command with check-only fallback
- `src-tauri/src/lib.rs` - registered the four new commands in generate_handler!

## Decisions Made
- Data-deletion commands take no frontend path argument; paths derived from `app_data_dir()` only (T-01-14 / T-01-15 mitigations — no path traversal).
- `clear_models` restricted to `*.bin` to avoid nuking unrelated files.
- `check_for_updates` is check-only this phase (T-01-16: no auto-install, so an unverified manifest cannot install anything). A missing endpoint/keypair surfaces as a benign "up to date" with the error captured. Wiring a real `updater.endpoints` + minisign keypair is deferred user_setup (from 01-02) and can land without code changes here.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None. First build showed "unused function" warnings for the new commands; these cleared after Task 2 registered them in `generate_handler!`. Final build is clean (0 warnings).

## User Setup Required
None required for this plan to function. A real auto-update path remains deferred user_setup (configure `updater.endpoints` in `tauri.conf.json` + generate a minisign keypair, `TAURI_SIGNING_PRIVATE_KEY` as a CI secret, embed pubkey). Until then, `check_for_updates` reports "Estás al día".

## Next Phase Readiness
- 01-07 can wire the confirmation-gated UI for clear_history / clear_models and the reveal button.
- 01-08/01-09 can surface the UpdateStatus result ("Estás al día" / "Hay versión X").
- `cargo build --manifest-path src-tauri/Cargo.toml` succeeds (clean).

## Self-Check: PASSED

- All modified files present on disk.
- Both task commits (6eacf1b, 0766189) found in git log.

---
*Phase: 01-redise-o-del-panel-de-configuraci-n-estilo-whisprflow*
*Completed: 2026-06-14*
