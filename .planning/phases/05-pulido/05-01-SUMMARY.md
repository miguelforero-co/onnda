---
phase: "05-pulido"
plan: "01"
subsystem: "CI / DevOps"
tags: ["ci", "github-actions", "cargo", "svelte-check", "workflow"]
dependency_graph:
  requires: []
  provides: ["ci-workflow-pr"]
  affects: []
tech_stack:
  added: ["GitHub Actions CI workflow"]
  patterns: ["macos-15 runner", "cargo no-bundle build", "Swatinem/rust-cache@v2"]
key_files:
  created:
    - .github/workflows/ci.yml
  modified: []
decisions:
  - "clippy added as non-blocking step (continue-on-error: true) to avoid breaking CI on pre-existing warnings, per POLISH-01 constraint"
  - "No swift/xcode steps needed because sidecar asr-aarch64-apple-darwin is committed in src-tauri/binaries/"
  - "cargo build no-bundle (not tauri build) keeps CI fast and avoids signing requirement in PRs"
metrics:
  duration: "~5 minutes"
  completed: "2026-06-15T20:37:37Z"
  tasks_completed: 1
  tasks_total: 1
  files_created: 1
  files_modified: 0
---

# Phase 5 Plan 1: CI Workflow for PRs Summary

**One-liner:** GitHub Actions CI on `macos-15` running `cargo build` + `cargo test` + `npm run check` on every push to main and every pull request, without full Tauri bundle or Swift sidecar rebuild.

## What Was Built

Created `.github/workflows/ci.yml` — a lean CI workflow that gates PRs and main pushes with:

1. Rust: `cargo build` (no-bundle) + `cargo test` in `src-tauri/`
2. Frontend: `npm ci` + `npm run check` (svelte-kit sync + svelte-check + tsc)
3. Optional lint: `cargo clippy` with `continue-on-error: true` (non-blocking, pre-existing warnings excluded)

The workflow mirrors the patterns in `release.yml` (same runner, same action versions, same cache setup) but deliberately omits the release-only steps: no `swift build`, no `xcode-select`, no `tauri build`, no signing.

## Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Crear el workflow de CI para PRs y push a main | 90bd111 | .github/workflows/ci.yml |

## Deviations from Plan

None — plan executed exactly as written.

The plan's `grep -A1 clippy` acceptance criterion is technically too narrow (the line `continue-on-error: true` appears 3 lines below the word "clippy", not 1), but was verified correctly with `grep -A5` confirming the attribute is present. All 11 substantive acceptance criteria pass.

## Verification Results

All acceptance criteria checked:

- `test -f .github/workflows/ci.yml` → PASS
- `grep -q "pull_request"` → PASS
- `grep -q "push"` → PASS
- `grep -q "macos-15"` → PASS
- `grep -Eq "cargo build"` → PASS
- `grep -Eq "cargo test"` → PASS
- `grep -q "npm run check"` → PASS
- `grep -q "npm ci"` → PASS
- `! grep -q "tauri build"` → PASS (no full Tauri build)
- `! grep -qi "swift build|xcode-select"` → PASS (no sidecar rebuild)
- `grep -A5 "clippy" | grep -q "continue-on-error: true"` → PASS

## Known Stubs

None.

## Threat Flags

None — this plan only adds a CI configuration file. No new network endpoints, auth paths, or schema changes.

## Self-Check: PASSED

- `.github/workflows/ci.yml` exists: FOUND
- Commit `90bd111` exists: FOUND (verified via `git rev-parse --short HEAD`)
- No unexpected file deletions in commit
