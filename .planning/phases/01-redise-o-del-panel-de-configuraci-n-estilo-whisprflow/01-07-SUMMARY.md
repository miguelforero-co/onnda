---
phase: 01-redise-o-del-panel-de-configuraci-n-estilo-whisprflow
plan: 07
subsystem: ui-settings
tags: [svelte, settings, ajustes, permissions, models, updates, data-management]
requires:
  - "01-03: extracted components (Toggle, PermissionRow, ModelCard, HotkeyRecorder) + Ajustes stub prop contract"
  - "01-04: expanded get_models (small/medium + Parakeet coming_soon) + ModelInfo.coming_soon Rust field"
  - "01-06: check_for_updates + reveal_data_dir/clear_history/clear_models commands"
  - "01-01: sound_on_listen/stop/cancel + pause_media settings fields"
provides:
  - "Ajustes.svelte: full settings surface wired to backend (D-07..D-15)"
  - "UpdateStatus TS type; ModelInfo.coming_soon TS field"
affects:
  - "01-09: manual verification of every Ajustes control"
tech-stack:
  added: []
  patterns:
    - "Section grouping: <section> + .section-label + .rows/.sep/.row (recovered from pre-refactor +page.svelte)"
    - "Local $state mirror of a prop with $effect re-sync, refreshable after destructive ops"
    - "window.confirm gate before destructive invoke (Phase 1 native confirm)"
key-files:
  created: []
  modified:
    - "src/lib/types.ts (ModelInfo.coming_soon + UpdateStatus interface)"
    - "src/lib/sections/Ajustes.svelte (stub → full settings section, ~280 lines)"
decisions:
  - "LANGUAGES list defined locally in Ajustes (the +page.svelte list cited by the plan was removed in the 01-03 shell refactor)"
  - "clear_models refreshes a local modelList copy (parent only re-fetches on download events), avoiding non-bindable prop mutation"
metrics:
  duration: ~2min
  tasks: 2
  files: 2
  completed: 2026-06-14
---

# Phase 01 Plan 07: Ajustes Settings Section Summary

Filled the `Ajustes.svelte` stub into the complete WhisprFlow-style settings surface — hotkey recorder, push-to-talk, language, 3 sound toggles + pause-media, launch-at-login, a live mic/a11y permissions panel, expanded model cards with the Parakeet "Próximamente" state, check-for-updates with Spanish result, and confirmation-gated data-management actions — all wired to the Wave 2-4 backend through the 01-03 prop contract, palette LOCKED, copy in Spanish.

## What Was Built

**Task 1 (commit `e076874`):**
- `ModelInfo.coming_soon: boolean` added to `types.ts` (mirrors the Rust struct from 01-04).
- Grabación: `HotkeyRecorder` (commits via `onSave(true)` → `shortcutChanged` → re_register, only after capture) + push-to-talk `Toggle` with flipping hint.
- Reconocimiento: language `<select>` (auto/es/en/pt/fr/de) + active-model `<select>` over downloaded models.
- Sonidos: `sound_on_listen` / `sound_on_stop` / `sound_on_cancel` + `pause_media` toggles.
- Sistema: `autostart` toggle. All `onchange={() => onSave()}`.

**Task 2 (commit `6f81178`):**
- `UpdateStatus` TS interface added to `types.ts`.
- Permisos: `PermissionRow` for Micrófono + Accesibilidad (live via parent polling; `onCheckPerms()` on mount); opens system settings.
- Modelos: `ModelCard` per model — real models selectable/downloadable, Parakeet (`coming_soon`) rendered muted with "Próximamente". Selection persists via `onSave()`.
- Actualizaciones: `check_for_updates` → "Estás al día" or "Hay una versión nueva disponible (vX.Y.Z)"; shows current version; disabled state while checking.
- Datos: `reveal_data_dir`; `clear_history` and `clear_models` each behind `window.confirm` with "no se puede deshacer" copy. `clear_models` refreshes the local model list.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] LANGUAGES list no longer in +page.svelte**
- **Found during:** Task 1
- **Issue:** Plan referenced `+page.svelte L50-57` for the LANGUAGES array, but the 01-03 shell refactor removed it (the shell is now a thin orchestrator).
- **Fix:** Defined the LANGUAGES array locally in `Ajustes.svelte` (recovered identical values from git history `b1dd6a5`).
- **Files modified:** `src/lib/sections/Ajustes.svelte`
- **Commit:** `e076874`

**2. [Rule 1 - Bug] Avoid non-bindable prop mutation on clear_models refresh**
- **Found during:** Task 2
- **Issue:** Plan suggested re-invoking get_models locally; assigning to the `models` prop directly would trigger Svelte 5 `ownership_invalid_mutation`.
- **Fix:** Introduced a local `$state` `modelList` mirror kept in sync with the prop via `$effect`; `clear_models` updates `modelList`. Added `svelte-ignore state_referenced_locally` (intentional — the effect handles syncing).
- **Files modified:** `src/lib/sections/Ajustes.svelte`
- **Commit:** `6f81178`

**3. [Rule 1 - Bug] Pre-existing `-webkit-appearance` lint on the select control**
- **Found during:** Task 2
- **Issue:** The `.sel` style (copied from the original markup) warned "Also define the standard property 'appearance'".
- **Fix:** Added standard `appearance: none;` alongside `-webkit-appearance: none;`. svelte-check is now fully clean (0 warnings).
- **Files modified:** `src/lib/sections/Ajustes.svelte`
- **Commit:** `6f81178`

No structural section reuse classes (`.rows`/`.sep`/`.section-label`) existed in shared CSS; they were recovered from the pre-refactor `+page.svelte` and inlined into the component's scoped `<style>` — consistent with the project's per-section style pattern.

## Threat Mitigations Applied

- **T-01-17 (accidental data wipe):** both `clear_history` and `clear_models` are behind `window.confirm` with explicit Spanish "no se puede deshacer" / "Tendrás que volver a descargarlos" copy before any `invoke`.
- **T-01-18 (update version string):** `available_version` / `current_version` interpolated as Svelte text (auto-escaped); no `{@html}`.

## Verification

- `npm run check`: 0 errors, 0 warnings.
- `npm run build`: succeeds (`✔ done`, built in ~0.9s).
- All Task 1 + Task 2 grep acceptance criteria pass.
- Backend commands confirmed registered in `src-tauri/src/lib.rs` (open_*_settings, reveal_data_dir, clear_history, clear_models).
- Manual functional verification deferred to 01-09 per plan.

## Self-Check: PASSED

- FOUND: src/lib/types.ts
- FOUND: src/lib/sections/Ajustes.svelte (321 lines, min 120)
- FOUND: .planning/phases/01-redise-o-del-panel-de-configuraci-n-estilo-whisprflow/01-07-SUMMARY.md
- FOUND commit e076874 (Task 1)
- FOUND commit 6f81178 (Task 2)
