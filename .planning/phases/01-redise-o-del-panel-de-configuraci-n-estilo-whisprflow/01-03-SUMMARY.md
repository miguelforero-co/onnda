---
phase: 01-redise-o-del-panel-de-configuraci-n-estilo-whisprflow
plan: 03
subsystem: frontend-shell
tags: [svelte5, design-system, sidebar, components, refactor]
requires:
  - "Rust structs (settings.rs/history.rs) extended with dictionary/source/sound/pause fields (01-01/01-02)"
provides:
  - "src/lib/styles/tokens.css — single source for the LOCKED palette + body reset"
  - "src/lib/types.ts — Settings/HistoryEntry/ModelInfo/DownloadProgress/View"
  - "Reusable components: Toggle, Row, Rows, PermissionRow, ModelCard, HotkeyRecorder, Sidebar"
  - "Section stubs with prop contracts: Home, Transcripciones, Diccionario, Ajustes"
  - "Two-column shell orchestrator (+page.svelte) routing view → sections"
affects:
  - "01-07 (fills Home + Ajustes against prop contracts)"
  - "01-08 (fills Transcripciones + Diccionario against prop contracts)"
tech-stack:
  added: []
  patterns:
    - "Svelte 5 runes ($props/$bindable/$derived/$state) for all new components"
    - "Design tokens in a plain imported .css (import \"$lib/styles/tokens.css\")"
    - "Section components receive orchestrator state + callbacks as props (no direct invoke in stubs)"
key-files:
  created:
    - src/lib/styles/tokens.css
    - src/lib/types.ts
    - src/lib/components/Toggle.svelte
    - src/lib/components/Row.svelte
    - src/lib/components/Rows.svelte
    - src/lib/components/PermissionRow.svelte
    - src/lib/components/ModelCard.svelte
    - src/lib/components/HotkeyRecorder.svelte
    - src/lib/components/Sidebar.svelte
    - src/lib/sections/Home.svelte
    - src/lib/sections/Transcripciones.svelte
    - src/lib/sections/Diccionario.svelte
    - src/lib/sections/Ajustes.svelte
  modified:
    - src/routes/+page.svelte
decisions:
  - "Added Rows.svelte (panel container with global .sep) so downstream plans get the .rows/.sep surface as a component, not just documentation"
  - "Normalized typography weights to the UI-SPEC 450/600 contract (was 550/650/700 in the inline CSS)"
  - "Removed dead history/settings logic from +page.svelte (playAudio/deleteEntry/fmtTime/LANGUAGES/etc.) — those move into Transcripciones/Ajustes in 01-07/01-08 against the prop contract"
metrics:
  duration_min: 4
  tasks: 2
  files: 14
  completed: 2026-06-14
---

# Phase 01 Plan 03: Shell Redesign + Component Extraction Summary

WhisprFlow-style two-column shell (200px sidebar + content router) replacing the header+tabs monolith, with the inline design system hoisted into reusable Svelte 5 components, a shared tokens stylesheet, and four section stubs whose prop contracts let 01-07/01-08 fill sections without touching `+page.svelte`. Palette preserved verbatim (D-04 LOCKED).

## What Was Built

- **`tokens.css`** — the `:root` palette (`--bg/--panel/--text/--muted/--faint/--line/--coral/--amber/--blue/--r`) plus the `*` reset and `body` font block. Single source; imported by the orchestrator.
- **`types.ts`** — `Settings` (extended with `sound_on_*`, `pause_media`, `dictionary: string[]`), `HistoryEntry` (with `source` + `original_filename`), `ModelInfo`, `DownloadProgress`, and the `View` union — all matching the Rust structs from 01-01/01-02.
- **Components** (Svelte 5 runes, scoped styles, verbatim CSS values):
  - `Toggle` (36×20, knob 16) — `{ checked=$bindable, label, id, onchange }`
  - `Row` (`label` + children slot) / `Rows` (panel container, global `.sep`)
  - `PermissionRow` — `{ label, description, granted, onOpen }`; granted → `--blue` dot + "Concedido"
  - `ModelCard` — `{ model, selected, progress, error, comingSoon, onDownload, onSelect }`; selected → coral 1px ring; comingSoon → muted "Próximamente" (no coral)
  - `HotkeyRecorder` — `{ shortcut=$bindable, onCommitted }`; idle/capturing/validating/saved keydown capture; rejects modifier-less combos; Escape cancels; never registers mid-capture
  - `Sidebar` — 200px `--panel` rail, wordmark/version, 4 nav items (hand-written inline SVG icons), active = `--text` on `--bg` pill, no coral on inactive
- **Section stubs** — `Home`, `Transcripciones`, `Diccionario`, `Ajustes`, each rendering a 16px/600 `page-title` and declaring its prop contract.
- **`+page.svelte`** — shell orchestrator: retains `$state`, `onMount` invoke/listen wiring, `schedSave`, `checkPerms`, `startDownload`, `finishOnboarding`, `goHistory`. Onboarding (using `PermissionRow` + `ModelCard`) precedes the shell; `finishOnboarding` lands on Home.

## Section Prop Contracts (for 01-07 / 01-08)

| Section | Props |
|---------|-------|
| `Home` | `{ settings: Settings, history: HistoryEntry[], onNavigate: (v: View) => void }` |
| `Transcripciones` | `{ history: HistoryEntry[], onRefresh: () => void }` |
| `Diccionario` | `{ settings: Settings, onSave: () => void }` |
| `Ajustes` | `{ settings, models, downloadProgress, downloadErrors, micGranted, a11yGranted, onSave: (shortcutChanged?: boolean) => void, onDownload: (modelId: string) => void, onCheckPerms: () => void }` |

Component prop contracts: see "What Was Built" above. `onSave` in Ajustes maps to `schedSave(sc)`; `onSave` in Diccionario maps to `schedSave()`; `HotkeyRecorder.onCommitted` should drive `schedSave(true)` in the parent (never re-register mid-capture).

## Deviations from Plan

### Auto-fixed / spec-mandated adjustments

**1. [Rule 2 - completeness] Added `Rows.svelte` container component**
- **Found during:** Task 1
- **Issue:** The plan said to either provide `Rows.svelte` OR document the `.rows`/`.sep` convention. Providing the component gives downstream plans a typed surface.
- **Files:** src/lib/components/Rows.svelte
- **Commit:** 86896bf

**2. [Spec normalization] Typography weights 550/650/700 → 450/600**
- **Found during:** Task 1 (PermissionRow, ModelCard, btn-primary)
- **Issue:** Inline CSS used non-contract weights; UI-SPEC L77 mandates normalizing any touched 550/650/700 to 450/600.
- **Fix:** wordmark 650→600, perm-info strong 550→600, perm-status/link-btn/badge 500→450, ob-intro h1 700→600.
- **Commit:** 86896bf, f6aeb32

**3. [Rule 1 - cleanup] Removed dead history/settings logic from `+page.svelte`**
- **Found during:** Task 2
- **Issue:** `playAudio`/`deleteEntry`/`fmtTime`/`fmtDur`/`playingId`/`audioEl`/`pasteStatus`/`LANGUAGES`/`POSITIONS` became unused once the history/settings markup moved out — would be dead code in the orchestrator.
- **Fix:** removed; this logic is reimplemented inside `Transcripciones`/`Ajustes` in 01-07/01-08 against the prop contracts.
- **Commit:** f6aeb32

**4. [Rule 1 - a11y] Suppressed false-positive a11y warning in ModelCard**
- **Found during:** Task 1
- **Issue:** svelte-check flagged the conditionally-interactive card div (`a11y_no_noninteractive_tabindex`) — static analysis can't resolve the dynamic role/tabindex pair.
- **Fix:** `<!-- svelte-ignore -->` on the line; role/tabindex are applied together only when `selectable`.
- **Commit:** 86896bf

## Known Stubs

The four section components are intentional stubs (render only a `page-title`). This is by design — the plan stands up the router + prop contracts so feature plans fill them:
- `Home.svelte`, `Ajustes.svelte` → filled in **01-07**
- `Transcripciones.svelte`, `Diccionario.svelte` → filled in **01-08**

The shell compiles, navigates between all four sections, and onboarding lands on Home. No data path is broken — the stubs receive live orchestrator state via props.

## Verification

- `npm run check` → 0 errors, 0 warnings (160 files).
- `npm run build` → succeeds (adapter-static, built in ~0.9s).
- Manual nav/visual verification deferred to the 01-09 checklist.

## Threat Surface

No new trust boundary (frontend structure only). T-01-07 mitigation upheld: history text will be rendered via plain `{text}` interpolation (Svelte auto-escapes) in 01-08 — no `{@html}` introduced anywhere in this plan.

## Self-Check: PASSED

All 13 created files present on disk; both task commits (86896bf, f6aeb32) in git log; check + build green.
