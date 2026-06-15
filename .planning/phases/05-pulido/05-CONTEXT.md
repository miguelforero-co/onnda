# Phase 5: Pulido - Context

**Gathered:** 2026-06-15
**Status:** Ready for planning
**Source:** Derivado de `.planning/research/LAUNCH-DIAGNOSIS.md` (sección F5).

<domain>
## Phase Boundary

Dejar la calidad del código y del pipeline de CI a la altura de un proyecto open source que recibirá contribuciones y escrutinio: CI en cada PR, tests en las rutas críticas hoy sin probar, y partir el god-file `commands.rs`. Es calidad interna; NO toca features de usuario, ni firma (F3), ni métricas (F4).

**Dentro de scope:** workflow de CI para PRs, tests de rutas críticas, refactor de `commands.rs` en módulos enfocados (sin cambio de comportamiento).

**Fuera de scope:** firma/notarización (F3), métricas (F4), cualquier feature nueva. El refactor es behavior-preserving — NO cambiar la lógica, solo moverla.
</domain>

<decisions>
## Implementation Decisions

### POLISH-01 — CI en cada PR
- Hoy solo existe `.github/workflows/release.yml` (release-on-tag). Falta un workflow que corra en `push`/`pull_request`.
- **Decisión:** nuevo workflow (ej. `.github/workflows/ci.yml`) que en cada PR/push a main corra: `cargo build` + `cargo test` (en src-tauri) + `npm ci` + `npm run check` (svelte-check). Runner `macos-15` (consistente con release.yml; el sidecar `asr-aarch64-apple-darwin` está commiteado, así que `cargo build` no-bundle no necesita cross-compile). No hace falta `tauri build` completo en el check de PR (más rápido y no requiere firma).
- **Claude's Discretion:** cachear cargo/npm; nombre exacto del workflow; si añadir `cargo clippy`/`cargo fmt --check` (deseable pero no romper el build por warnings preexistentes — usar `clippy` no-bloqueante o limitarlo).

### POLISH-02 — Tests de rutas críticas (pragmático)
- Hoy `commands.rs`, `whisper_backend.rs`, `vad.rs` tienen 0 tests (la inferencia completa no es unit-testeable sin modelo). ~44 tests existentes tras F1/F2 (incluye compat.rs).
- **Decisión:** añadir tests a la **lógica pura testeable**, no forzar E2E:
  - `vad.rs` — `vad_trim()` con casos (silencio, ruido puro, clip muy corto). Es función pura, alta prioridad.
  - Helpers puros que se extraigan en el refactor de POLISH-03 (ej. `resolve_model_path`, construcción del catálogo de modelos, verificación de sha256, parse de respuestas) — testearlos al moverlos.
  - `whisper_backend`: lo testeable son los helpers de pre-proceso (resample/normalize ya en transcription.rs tienen tests); la carga de modelo y la inferencia quedan como UAT manual (necesitan un modelo de 1.5GB).
- **Claude's Discretion:** qué helpers concretos valen un test; el E2E de dictado real sigue siendo UAT humano (no inventar mocks frágiles del pipeline de audio).

### POLISH-03 — Partir el god-file `commands.rs` (~1017 LOC, behavior-preserving)
- **Decisión:** extraer en módulos enfocados, SIN cambiar comportamiento:
  - `paste.rs` — inyección de texto: clipboard (write_clipboard_utf8) + CGEvent (post_cmd_v) + `paste_text`.
  - `models.rs` — catálogo (`get_models`), descarga (`download_model` con sha256+pin), `check_model_status`, resolución de path de modelo (extraer `resolve_model_path` deduplicando los 4 sitios).
  - `recording.rs` — máquina de estados de grabación + loop de streaming + ensamblado del tail (start/stop internos).
  - `commands.rs` queda como wrappers delgados de `#[tauri::command]` + lo que no encaje.
- Actualizar `lib.rs` (módulos + invoke_handler sigue apuntando a los comandos en su nueva ubicación; los `#[tauri::command]` pueden re-exportarse o moverse con `pub use`).
- **CRÍTICO:** es refactor mecánico. Build verde + los ~44 tests + `npm run check` deben pasar idénticos. Hacerlo DESPUÉS de POLISH-02 para que los tests nuevos blinden el movimiento.
- **Claude's Discretion:** límites exactos de cada módulo; usar `pub(crate)` donde aplique; mantener los `static` (CAPTURE, COMMITTED_TEXT, etc.) donde tenga más sentido (probablemente recording.rs).
</decisions>

<canonical_refs>
## Canonical References

### Diagnóstico de origen
- `.planning/research/LAUNCH-DIAGNOSIS.md` — sección "F5 — Pulido" (god-file, tests faltantes, CI).

### Código a tocar (leer estado actual antes)
- `.github/workflows/release.yml` — patrón de CI existente a imitar para el de PRs (POLISH-01)
- `src-tauri/src/commands.rs` — 1017 LOC a partir (POLISH-03)
- `src-tauri/src/vad.rs` — sin tests (POLISH-02)
- `src-tauri/src/lib.rs` — registro de módulos + invoke_handler (POLISH-03)
- `src-tauri/src/transcription.rs`, `compat.rs` — patrón de `#[cfg(test)] mod tests` existente a imitar

### Convenciones
- `./CLAUDE.md`. Iterar en `npm run tauri dev`. El refactor NO cambia comportamiento — verificar con build+tests+app.
</canonical_refs>

<specifics>
## Specific Ideas

- Imitar el patrón de tests ya presente en `transcription.rs`/`compat.rs`/`replacements.rs` (`#[cfg(test)] mod tests`).
- Deduplicar la lógica `primary = ggml-{model}.bin / fallback = ggml-base.bin / exists()` repetida 4× en commands.rs → un `resolve_model_path()` en models.rs (mencionado en el diagnóstico como duplicación LOW).
- El refactor debe dejar `lib.rs invoke_handler` funcionando idéntico — confirmar que todos los comandos siguen registrados.
</specifics>

<deferred>
## Deferred Ideas

- E2E real del pipeline de dictado (audio→whisper→paste) → UAT humano, no automatizable de forma robusta aquí.
- clippy/fmt estrictos como gate bloqueante → opcional; no bloquear el build por warnings preexistentes en esta fase.
</deferred>

---

*Phase: 05-pulido*
*Context derivado del diagnóstico 2026-06-15*
