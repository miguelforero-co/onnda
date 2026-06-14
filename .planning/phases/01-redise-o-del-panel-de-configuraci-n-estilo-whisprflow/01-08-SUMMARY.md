---
phase: 01-redise-o-del-panel-de-configuraci-n-estilo-whisprflow
plan: 08
subsystem: ui
tags: [svelte5, tauri, plugin-dialog, transcripciones, diccionario, home]

# Dependency graph
requires:
  - phase: 01-03
    provides: shell router + section stubs with prop contracts (Home/Transcripciones/Diccionario/Ajustes)
  - phase: 01-05
    provides: transcribe_file command + file-transcribe-progress/done/error events
  - phase: 01-01
    provides: dictionary↔custom_words migration on disk-read; HistoryEntry source + original_filename
provides:
  - Home.svelte dictation hub (shortcut badge + Dictar hero + Recientes + navigate)
  - Transcripciones.svelte unified list with source filter + file upload + play/delete/copy
  - Diccionario.svelte item-list editor (add/edit/delete) syncing custom_words
affects: [01-09]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Section components own their own playback/upload state; parent passes history + onRefresh"
    - "Dictionary mutations always sync custom_words = dictionary.join(', ') then onSave() (D-20)"
    - "File upload = plugin-dialog open → invoke transcribe_file → listen file-transcribe-* → onRefresh on done"

key-files:
  created: []
  modified:
    - src/lib/sections/Home.svelte
    - src/lib/sections/Transcripciones.svelte
    - src/lib/sections/Diccionario.svelte

key-decisions:
  - "Home Dictar hero is informational (disabled button + shortcut hint) — dictation fires via global hotkey, not a click"
  - "Diccionario chips support inline edit (click word → input) plus add/delete; duplicates rejected case-insensitively"
  - "Transcripciones delete + upload-done both call onRefresh() (re-pull get_history) to keep parent state authoritative"
  - "File entries tagged with a blue filename chip (--blue is the success/identity accent, not coral)"

patterns-established:
  - "History render (hist-item/hist-meta/hist-time/hist-dur/icon-btn/hist-text) recovered from pre-refactor +page.svelte and lifted into Home + Transcripciones"
  - "playAudio helper (get_recording_audio → base64 data URL) lifted into Transcripciones with local audioEl/playingId state"

requirements-completed: [Vista-unificada-transcripciones, Transcripcion-por-archivos, Diccionario-items, Shell-sidebar]

# Metrics
duration: 3min
completed: 2026-06-14
---

# Phase 01 Plan 08: Content sections (Home / Transcripciones / Diccionario) Summary

**Filled the three 01-03 stubs: a Home dictation dashboard, a unified Transcripciones list with source filter + file upload wired to transcribe_file, and a chip-based Diccionario editor that keeps custom_words synced for Whisper.**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-06-14T18:13:45Z
- **Completed:** 2026-06-14T18:16:08Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Home (D-03): page title "Inicio", a coral "Dictar" hero with the current shortcut as a `kbd` badge, a "Recientes" block (last 5 entries) with a "Ver todas" link, and a "Subir audio" quick action — all navigating via `onNavigate`.
- Transcripciones (D-16/D-17/D-18): unified `HistoryEntry` list, a Todas/Dictado/Archivo segmented source filter, a working "Subir audio" that opens the Tauri dialog and invokes `transcribe_file`, subscribes to `file-transcribe-progress|done|error` (Spanish status/errors), refreshes the list on done, shows the original filename for file entries, and reuses play/delete plus a copy-to-clipboard action.
- Diccionario (D-19/D-20/D-21): replaced the textarea with an add/edit/delete word-chip editor over `settings.dictionary`; every mutation syncs `custom_words = dictionary.join(", ")` then calls `onSave()`, preserving the Whisper `initial_prompt`/`correct_words` backend contract. Words-only (no `a→b` replacements).

## Task Commits

1. **Task 1: Home (dictation hub) + Diccionario (item-list editor)** - `0b6ace9` (feat)
2. **Task 2: Transcripciones (unified list + source filter + file upload)** - `fc4efd6` (feat)

## Files Created/Modified
- `src/lib/sections/Home.svelte` - Dictation dashboard: shortcut badge, Dictar hero, Recientes, navigate.
- `src/lib/sections/Transcripciones.svelte` - Unified list, source filter, file upload via dialog + transcribe_file events, play/delete/copy.
- `src/lib/sections/Diccionario.svelte` - Word-chip add/edit/delete editor; custom_words sync + onSave.

## Decisions Made
- Home "Dictar" is a disabled, informational hero (dictation is hotkey-driven); the actionable affordances are the navigation links and "Subir audio".
- Filename of file-sourced entries rendered with the `--blue` identity accent (not coral, per the LOCKED palette role split).
- Delete and upload-done both re-pull history via `onRefresh()` so the parent `+page.svelte` stays the single source of truth (no divergent local copy).

## Deviations from Plan
None - plan executed exactly as written. (No +page.svelte edits; implemented against the existing prop contracts.)

## Issues Encountered
None. `npm run check` (161 files, 0 errors/0 warnings) and `npm run build` both passed clean on first run after each task.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All four sections (Home, Transcripciones, Diccionario, Ajustes) are now fully wired — ready for 01-09 final integration/manual QA.
- Manual checks deferred to 01-09: upload an mp3 → appears tagged "Archivo"; filter Dictado/Archivo; play/delete; dictionary add/delete persists and biases Whisper.

## Self-Check: PASSED

- FOUND: src/lib/sections/Home.svelte
- FOUND: src/lib/sections/Transcripciones.svelte
- FOUND: src/lib/sections/Diccionario.svelte
- FOUND: 01-08-SUMMARY.md
- FOUND commit: 0b6ace9 (Task 1)
- FOUND commit: fc4efd6 (Task 2)

---
*Phase: 01-redise-o-del-panel-de-configuraci-n-estilo-whisprflow*
*Completed: 2026-06-14*
