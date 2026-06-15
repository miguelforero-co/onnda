---
phase: 01-blindaje-de-producci-n
plan: "01"
subsystem: rust-backend
tags: [logging, deps, tauri-plugin-log, parking_lot, harden]
dependency_graph:
  requires: []
  provides: [tauri-plugin-log registered, parking_lot dep, log dep]
  affects: [01-02-PLAN, 01-03-PLAN]
tech_stack:
  added: [parking_lot 0.12, tauri-plugin-log 2, log 0.4]
  patterns: [rotating-log-LogDir, KeepOne-rotation]
key_files:
  created: []
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/lib.rs
    - src-tauri/capabilities/default.json
    - src-tauri/Cargo.lock
decisions:
  - "KeepOne rotation (not KeepAll) para evitar bug macOS tauri-apps/plugins-workspace#1397"
  - "Log plugin registrado como primer .plugin() del builder para capturar logs de init de los demas plugins"
  - "Limite 5 MB por archivo, LogDir target con file_name voz-local"
metrics:
  duration: "~20 min (dominado por cargo build con deps nuevas)"
  completed: "2026-06-15T18:42:34Z"
  tasks_completed: 3
  tasks_total: 3
  files_changed: 4
---

# Phase 01 Plan 01: Infraestructura de logging a disco (HARDEN-06 foundation)

**One-liner:** Plugin tauri-plugin-log 2 registrado primero en el builder con LogDir "voz-local", 5 MB, KeepOne; deps parking_lot 0.12 y log 0.4 declaradas y resueltas; cargo build verde.

## What Was Built

Fundacion de robustez para la Phase 1: las tres dependencias nuevas que necesitan los planes 01-02 y 01-03 estan ahora declaradas en Cargo.toml y resueltas en Cargo.lock. El logger rotativo a disco queda registrado como el PRIMER plugin del builder de Tauri, garantizando que los logs de inicializacion de todos los demas plugins (autostart, global-shortcut, etc.) se capturen en `~/Library/Logs/{bundle_id}/voz-local.log`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Anadir deps parking_lot, tauri-plugin-log, log | 636daa5 | src-tauri/Cargo.toml |
| 2 | Registrar tauri-plugin-log como primer plugin | 0c2ab56 | src-tauri/src/lib.rs |
| 3 | Anadir log:default y verificar compilacion | bc88967 | src-tauri/capabilities/default.json, Cargo.lock |

## Decisions Made

1. **KeepOne en lugar de KeepAll:** El issue #1397 de tauri-apps/plugins-workspace documenta un bug en macOS donde KeepAll solo retiene 2 archivos de log independientemente del valor configurado. KeepOne es el comportamiento correcto y documentado.

2. **Log plugin primero en el builder:** Si se registrara despues de otros plugins, los mensajes de inicializacion de `tauri_plugin_autostart`, `tauri_plugin_global_shortcut`, etc. solo llegarian a stderr (invisible desde Finder en produccion). Registrarlo primero los captura todos.

3. **5 MB por archivo:** Balanceo entre diagnosticabilidad (logs suficientes para un ciclo de uso) y espacio en disco (~10 MB total con KeepOne).

## Deviations from Plan

None — plan ejecutado exactamente como estaba escrito.

## Known Stubs

None.

## Threat Flags

None — ningun endpoint de red nuevo, ningun path de auth nuevo, ningun cambio de schema. El logger escribe solo a `~/Library/Logs/` (local, accesible solo por el usuario). Los planes 02/03 que usan `log::info!`/`log::warn!` usan format args (no concatenacion), previniendo inyeccion de control chars (T-01-02 del threat model del plan, mitigacion implementada por diseno del API de log).

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| src-tauri/Cargo.toml | FOUND |
| src-tauri/src/lib.rs | FOUND |
| src-tauri/capabilities/default.json | FOUND |
| 01-01-SUMMARY.md | FOUND |
| commit 636daa5 (Cargo.toml deps) | FOUND |
| commit 0c2ab56 (log plugin registration) | FOUND |
| commit bc88967 (capabilities + Cargo.lock) | FOUND |
