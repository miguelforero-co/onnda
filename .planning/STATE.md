---
gsd_state_version: 1.0
milestone: v2.0
milestone_name: Camino al lanzamiento público
status: defining
stopped_at: Milestone v2.0 iniciado — roadmap de 5 fases creado, listo para planear Phase 1.
last_updated: "2026-06-15T12:00:00.000Z"
last_activity: 2026-06-15
progress:
  total_phases: 5
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-06-15)

**Core value:** Dictado por voz rápido, privado y siempre disponible vía atajo global.
**Current focus:** Milestone v2.0 — preparar el lanzamiento público (descarga directa firmada + OSS, Intel+Silicon, gratis). Empezar por Phase 1 (Blindaje de producción).

## Current Position

Phase: Not started (defining requirements)
Plan: —
Status: Roadmap creado (5 fases), listo para `/gsd-plan-phase 1`
Last activity: 2026-06-15 — Milestone v2.0 iniciado

Progress: [          ] 0 of 5 complete

## Accumulated Context

### Decisions

Decisions logged in PROJECT.md Key Decisions table. Clave para v2.0:
- Distribución por descarga directa, NO App Store (auto-paste incompatible con App Sandbox).
- Soportar Intel + Apple Silicon; Neural Engine solo en Silicon (no existe en Intel).
- Gratis + OSS en v1, sin cuentas ni backend ni pagos.
- Métricas opt-in, solo conteos jamás contenido (Aptabase + Sentry/GlitchTip).
- Diagnóstico técnico completo (con file:line) en `.planning/research/LAUNCH-DIAGNOSIS.md`.

### Pending Todos

- UAT pendiente de v1.0: probar dictado por voz con el motor Apple (Ajustes→Modelos→"Apple (Neural Engine)").
- ~19 commits locales sin pushear en `main` (NO pushear hasta que el usuario lo pida).

### Blockers/Concerns

- 3 crashes en ruta crítica (audio.expect, mutex envenenado, descarga sin integridad) → Phase 1.
- Build x86_64 de Intel hoy falla (sidecar solo aarch64) → Phase 2.

## Session Continuity

Last session: 2026-06-15
Stopped at: Milestone v2.0 roadmap creado
Resume file: None
