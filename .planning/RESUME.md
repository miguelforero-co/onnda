# Handoff — Voz Local · Milestone v2.0 (2026-06-15)

## Por qué conviene reiniciar el computador antes de seguir

Durante esta sesión lancé la app en modo dev varias veces en background para hacer smoke-tests. Esos lanzamientos dejaron **procesos de desarrollo vivos** que conviene limpiar:

| PID | Proceso | Qué está haciendo |
|-----|---------|-------------------|
| 10245 | `npm run tauri dev` | launcher que quedó corriendo |
| 10263 | `node tauri dev` | watcher que recompila al guardar |
| 10433 | `node vite dev` | **tiene tomado el puerto 1420** |
| 10496 | `target/debug/voz-local` | **instancia de la app corriendo** (agarra el atajo global Alt+Space) |
| 10550 | (vite worker) | en el puerto 1420 |

**El motivo real:** esa instancia de dev tiene tomado el **atajo global Alt+Space** y el **puerto 1420**. Si abres otra instancia, pelean por el atajo y por el puerto. Además se le añadieron plugins nuevos (`tauri-plugin-log`) y deps (`parking_lot`) que solo cargan limpio en un arranque fresco.

**Importante:** un reinicio NO es estrictamente obligatorio — basta con matar esos procesos (`pkill -f "target/debug/voz-local"; pkill -f "tauri dev"; pkill -f "vite"`). Pero reiniciar es la forma más simple de garantizar pizarra limpia: mata todos los procesos huérfanos, libera el puerto, y suelta el atajo global. Recomendado antes de retomar el dev.

> Nota macOS (de la memoria del proyecto): reinstalar la app en /Applications resetea permisos (firma adhoc cambia). Para dev NO reinstales — solo `npm run tauri dev`.

## Estado del proyecto al pausar

- **Milestone v2.0 "Camino al lanzamiento público": 3 de 5 fases completas.**
  - ✅ **F1 Blindaje de producción** (verifier 6/6): no-crash mic, parking_lot, SHA256+pin, banner modelo, fallos visibles, logging a disco.
  - ✅ **F2 Compatibilidad Intel + Apple Silicon** (verifier 5/5): DMG x86_64 construible, motor Apple gateado, modelo por hardware, threads clamp, fix notch.
  - ✅ **F5 Pulido**: CI en PRs, tests 44→53, `commands.rs` 1017→198 LOC partido en paste/models/recording.
  - ⏸ **F3 Firma+notarización+repo público** — BLOQUEADA: necesita tu cuenta **Apple Developer ($99/año)** + certificado Developer ID + secrets en GitHub.
  - ⏸ **F4 Métricas opt-in** — BLOQUEADA: necesita key de **Aptabase** (+ opcional Sentry/GlitchTip).
- **Código:** `cargo build` verde · 53 tests · `npm check` 0/0 · app arranca limpia.
- **Git:** rama `main`, working tree limpio, **152 commits locales SIN pushear** (incluye ~19 previos + todo el milestone de hoy). NO pushear hasta que lo pidas.
- **UAT físico opcional pendiente de F1/F2** (no testeado con hardware): desconectar mic en grabación, banner sin modelo, contenido del log al dictar, flash del notch en pantalla sin notch, card Apple deshabilitada en Intel.

## Artefactos clave
- Diagnóstico completo (con file:line): `.planning/research/LAUNCH-DIAGNOSIS.md`
- Roadmap + estado: `.planning/ROADMAP.md`, `.planning/STATE.md`, `.planning/REQUIREMENTS.md`
- Planes/summaries por fase: `.planning/phases/0{1,2,5}-*/`

## Cómo retomar
Reinicia (o mata los procesos dev), abre Claude en el repo, y pega el prompt de continuación (abajo en el chat / en `.planning/STATE.md`).
