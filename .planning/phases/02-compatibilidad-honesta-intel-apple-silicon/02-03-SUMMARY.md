---
phase: 02-compatibilidad-honesta-intel-apple-silicon
plan: 03
subsystem: frontend-svelte
tags: [compat, ui, model-card, widget, notch, hardware-detection]
requires:
  - commands::ModelInfo.disabled_reason  # from plan 02
provides:
  - ui::ModelCard.hardwareDisabled
  - ui::ModelInfo.disabled_reason
  - ui::widget.hasNotch-init-false
affects:
  - src/lib/types.ts (ModelInfo interface)
  - src/lib/components/ModelCard.svelte (disabled state rendering)
  - src/routes/widget/+page.svelte (notch init)
tech-stack:
  added: []
  patterns:
    - derived boolean (hardwareDisabled) from nullable model field, reusing existing .coming-soon CSS
    - $state(false) init + event-driven transition to eliminate first-frame flash
key-files:
  created: []
  modified:
    - src/lib/types.ts
    - src/lib/components/ModelCard.svelte
    - src/routes/widget/+page.svelte
decisions:
  - "Reused .coming-soon CSS class for hardwareDisabled (same muted/no-pointer appearance) rather than adding a new class — distinct states share visual treatment"
  - "disabled_reason rendered as inline subtitle + badge inside ModelCard, no new prop needed at call-sites"
  - "hasNotch init changed from true to false; existing CSS transition handles pill→notch on real notch screens"
metrics:
  duration: "8 minutes"
  completed: "2026-06-15T20:30:00Z"
  tasks: 3
  files: 3
---

# Phase 2 Plan 03: COMPAT-02 UI + COMPAT-05 Summary

**One-liner:** TypeScript ModelInfo gains `disabled_reason: string | null`; ModelCard renders hardware-gated engines as muted/non-selectable with a Spanish subtitle and "No disponible" badge; widget hasNotch starts false to eliminate the notch-shape flash on non-notch screens.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | COMPAT-02 UI: types.ts + ModelCard disabled_reason | e60cd67 | src/lib/types.ts, src/lib/components/ModelCard.svelte |
| 2 | COMPAT-05: hasNotch init false | aa9c0ef | src/routes/widget/+page.svelte |
| 3 | Checkpoint: human visual verification | — | (awaiting) |

## Verification

- `npm run check`: 161 files, 0 errors, 0 warnings (run after each task)
- All 5 grep acceptance criteria for Task 1 pass
- Task 2 grep acceptance criterion passes (`let hasNotch = $state(false);` confirmed on line 14)

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. `disabled_reason` flows from the Rust backend (Plan 02) through the JSON response into `model.disabled_reason` in ModelCard. The subtitle and badge render when the field is non-null.

## Threat Flags

No new threat surface introduced. Changes are purely presentational (read-only model field rendering and a state init value).

## Self-Check

### Modified files exist:
- `src/lib/types.ts` — FOUND (disabled_reason: string | null on line 52)
- `src/lib/components/ModelCard.svelte` — FOUND (hardwareDisabled, No disponible, model.disabled_reason)
- `src/routes/widget/+page.svelte` — FOUND (hasNotch = $state(false) on line 14)

### Commits exist:
- e60cd67 — Task 1 (feat: COMPAT-02 UI)
- aa9c0ef — Task 2 (fix: COMPAT-05)

## Self-Check: PASSED
