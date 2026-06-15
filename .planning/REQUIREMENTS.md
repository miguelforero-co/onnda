# Requirements — Milestone v2.0 Camino al lanzamiento público

Derivados del diagnóstico en `research/LAUNCH-DIAGNOSIS.md`. Cada requirement es atómico, testeable y trazable a una fase.

## v2.0 Requirements

### HARDEN — Blindaje de producción

- [x] **HARDEN-01**: La app no crashea si el micrófono se desconecta, está bloqueado por otra app o es denegado durante la grabación (`audio.rs:73,75` `.expect()` → `Result`)
- [x] **HARDEN-02**: Un panic durante una transcripción no inutiliza el resto de la sesión (`MODEL_CACHE` no envenenable — `parking_lot` o recuperación de lock)
- [x] **HARDEN-03**: El modelo descargado se verifica por integridad (SHA256 contra hash fijado) y la URL de descarga está pinneada a un commit estable
- [x] **HARDEN-04**: En el primer arranque sin conexión (modelo no descargado) el usuario ve un estado claro y accionable, no un fallo críptico
- [x] **HARDEN-05**: Los fallos de transcripción (segmento de streaming o tail) se muestran al usuario en vez de descartarse en silencio
- [x] **HARDEN-06**: La app escribe logs rotativos a disco para diagnosticar fallos en producción (reemplaza los `eprintln!` que se pierden al lanzar desde Finder)

### COMPAT — Compatibilidad honesta Intel + Apple Silicon

- [x] **COMPAT-01**: El DMG de Intel (x86_64) se construye correctamente en CI (sidecar Apple opcional o build x86_64 del sidecar)
- [x] **COMPAT-02**: El motor Apple solo aparece disponible en Apple Silicon + macOS 26 con sidecar presente; oculto o deshabilitado con explicación donde no aplica
- [x] **COMPAT-03**: El modelo por defecto se elige según el hardware al primer arranque (Intel / poca RAM → small/base; Apple Silicon → turbo o motor Apple)
- [x] **COMPAT-04**: El número de threads de inferencia se limita a los cores reales de la máquina
- [ ] **COMPAT-05**: El widget no muestra el flash de forma-notch en pantallas sin notch (`hasNotch` init correcto)

### DIST — Firma, notarización y repo público

- [ ] **DIST-01**: El binario se firma con Developer ID, hardened runtime y los entitlements correctos (micrófono + accesibilidad)
- [ ] **DIST-02**: El DMG se notariza y staplea en CI; el usuario instala sin instrucciones de `xattr`/right-click-open
- [ ] **DIST-03**: El updater funciona de verdad (latest.json firmado en GitHub Releases) o se retira la dependencia falsa dejando un check honesto
- [ ] **DIST-04**: El repositorio está listo para abrirse como OSS: `.planning/` y `dev/` fuera del repo público, LICENSE MIT, README de cara al público
- [ ] **DIST-05**: La versión de la app es consistente en todos los manifiestos (`package.json`, `Cargo.toml`, `tauri.conf.json`, user-agent)

### METRICS — Medir y aprender opt-in

- [ ] **METRICS-01**: Analítica opt-in (apagada por defecto) vía Aptabase que registra solo conteos/eventos, jamás contenido transcrito ni PII
- [ ] **METRICS-02**: Crash reporting opt-in (tauri-plugin-sentry / GlitchTip) con `send_default_pii:false` y contenido scrubeado
- [ ] **METRICS-03**: El onboarding pide consentimiento de telemetría con explicación honesta de qué se recoge y qué nunca

### POLISH — Pulido

- [ ] **POLISH-01**: CI corre build + tests + `svelte-check` en cada PR (no solo en release-on-tag)
- [ ] **POLISH-02**: Tests cubren las rutas críticas hoy sin probar (recording/transcription en `commands.rs`, `whisper_backend.rs`, `vad.rs`)
- [ ] **POLISH-03**: `commands.rs` (god-file 909 LOC) se parte en módulos enfocados (paste, models, recording)

## Future Requirements (diferidos)

- Investigación profunda de tácticas de lanzamiento (Product Hunt / HN / Homebrew cask / pricing)
- Monetización: Polar (merchant-of-record) + llaves Ed25519 offline si se decide cobrar
- Cuentas / sync / backend si aparece una feature que lo exija (multi-dispositivo, equipos)
- Motor más rápido (Parakeet/ANE vía FluidAudio) si la latencia lo justifica

## Out of Scope (v2.0, con razón)

- **Mac App Store** — incompatible con el App Sandbox por el auto-paste (Accessibility/CGEvent)
- **Cuentas / backend propio / sync** — la app es 100% local; sin valor en v1
- **Pagos / licencias** — v2.0 es gratis + OSS
- **Cambiar el motor de ASR por defecto** — elección del usuario

## Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| HARDEN-01..06 | Phase 1 | ✅ Done (2026-06-15) |
| COMPAT-01..05 | Phase 2 | Pending |
| DIST-01..05 | Phase 3 | Pending |
| METRICS-01..03 | Phase 4 | Pending |
| POLISH-01..03 | Phase 5 | Pending |
