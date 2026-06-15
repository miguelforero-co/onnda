---
gsd_state_version: 1.0
milestone: v2.0
milestone_name: milestone
status: executing
stopped_at: Completed 05-01-PLAN.md (CI workflow)
last_updated: "2026-06-15T20:38:28.173Z"
last_activity: 2026-06-15
progress:
  total_phases: 5
  completed_phases: 2
  total_plans: 10
  completed_plans: 8
  percent: 80
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-06-15)

**Core value:** Dictado por voz rápido, privado y siempre disponible vía atajo global.
**Current focus:** Phase 05 — pulido

## Current Position

Phase: 05 (pulido) — EXECUTING
Plan: 2 of 3
Status: Ready to execute
Last activity: 2026-06-15

Progress: [████      ] 2 of 5 complete

## Accumulated Context

### Decisions

Decisions logged in PROJECT.md Key Decisions table. Clave para v2.0:

- Distribución por descarga directa, NO App Store (auto-paste incompatible con App Sandbox).
- Soportar Intel + Apple Silicon; Neural Engine solo en Silicon (no existe en Intel).
- Gratis + OSS en v1, sin cuentas ni backend ni pagos.
- Métricas opt-in, solo conteos jamás contenido (Aptabase + Sentry/GlitchTip).
- Diagnóstico técnico completo (con file:line) en `.planning/research/LAUNCH-DIAGNOSIS.md`.

**Phase 1 (decisiones de implementación):**

- KeepOne rotation para tauri-plugin-log (evita bug macOS con KeepAll); plugin registrado primero en el builder.
- mpsc::channel para propagar Result del stream cpal desde thread spawneado (HARDEN-01).
- parking_lot::Mutex solo en MODEL_CACHE; CAPTURE/COMMITTED_TEXT/STREAM_HANDLE diferidos a Phase 5.
- URLs de modelos pinneadas a commit SHA de HF + SHA256 verificado en streaming antes de rename (HARDEN-03).
- transcribe-warning no aborta el loop (preserva segmentos acumulados); check_model_status reutiliza lógica primary/fallback.
- [Phase 02-compatibilidad-honesta-intel-apple-silicon]: swift build --triple x86_64-apple-macosx11.0 para cross-compilar sidecar Swift a x86_64 desde arm64; ambos triples antes del primer tauri build
- [Phase 02-compatibilidad-honesta-intel-apple-silicon]: CI usa macos-15 (Xcode 26.x, SDK macOS 26) requerido por Package.swift .macOS(26.0)
- [Phase 02-compatibilidad-honesta-intel-apple-silicon]: disabled_reason: Option<String> added to ModelInfo (not reusing coming_soon) to distinguish hardware-unavailable from not-yet-built
- [Phase 02-compatibilidad-honesta-intel-apple-silicon]: Default for AppSettings unchanged (selected_model stays large-v3-turbo) to preserve existing settings.json deserialization; first-run override only in init() not-exists branch
- [Phase 02-compatibilidad-honesta-intel-apple-silicon]: Reused .coming-soon CSS for hardwareDisabled; disabled_reason rendered as inline subtitle + badge inside ModelCard (no new call-site props)
- [Phase 02-compatibilidad-honesta-intel-apple-silicon]: hasNotch init changed from true to false; existing CSS transition handles pill→notch on real notch screens with no layout-shift
- [Phase 05-01]: CI workflow (ci.yml): cargo build no-bundle + cargo test in src-tauri + npm ci + npm run check on macos-15, clippy non-blocking (POLISH-01)

### Pending Todos

- **UAT físico opcional de Phase 1** (rutas verificadas en código + verifier, pero no ejercidas con hardware): (1) desconectar mic en grabación → no crashea; (2) abrir sin modelo → banner; (3) dictar → ver contenido en `~/Library/Logs/com.vozlocal.app/voz-local.log`.
- UAT pendiente de v1.0: probar dictado con el motor Apple.
- Commits locales sin pushear en `main` (NO pushear hasta que el usuario lo pida).

### Blockers/Concerns

- ✅ RESUELTO en Phase 1: los 3 crashes de ruta crítica (audio.expect, mutex envenenado, descarga sin integridad).
- Build x86_64 de Intel hoy falla (sidecar solo aarch64) → Phase 2.

## Session Continuity

Last session: 2026-06-15T20:38:28.171Z
Stopped at: Completed 05-01-PLAN.md (CI workflow)
Resume file: None
