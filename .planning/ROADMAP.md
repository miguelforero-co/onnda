# Roadmap: Voz Local — Milestone v2.0 Camino al lanzamiento público

## Overview

Llevar Voz Local de "app personal madura" a "producto público": descarga directa firmada+notarizada + open source, funcionando bien en Intel y Apple Silicon, gratis. El trabajo se ordena por riesgo de lanzamiento: primero que **no crashee en público** (F1), luego que **funcione honestamente en toda la flota Mac** (F2), luego **firma/notarización/repo público** que destraban la distribución (F3), luego **medir sin traicionar la privacidad** (F4), y por último **pulido de calidad** (F5). Diagnóstico completo en `research/LAUNCH-DIAGNOSIS.md`.

> Milestone previo (v1.0: rediseño + motor Apple + auto-learn) archivado en `.planning/archive/v1.0/`. Numeración de fases reiniciada a 1 para este milestone.

## Phases

- [x] **Phase 1: Blindaje de producción** — cero crashes en ruta crítica, integridad+offline del modelo, fallos visibles, logging a disco
- [ ] **Phase 2: Compatibilidad honesta (Intel + Apple Silicon)** — build x86_64, motor Apple gateado a Silicon, modelo por defecto según hardware
- [ ] **Phase 3: Firma, notarización y repo público** — Developer ID + notarytool en CI, updater real, LICENSE MIT, higiene OSS
- [ ] **Phase 4: Métricas y crash reporting opt-in** — Aptabase + Sentry/GlitchTip, solo conteos, consentimiento honesto
- [ ] **Phase 5: Pulido** — CI en PRs, tests de rutas críticas, partir god-file commands.rs

## Phase Details

### Phase 1: Blindaje de producción
**Goal**: La app no crashea ni pierde dictado en silencio en las rutas que un usuario público va a estrenar; los fallos son visibles y diagnosticables.
**Depends on**: Nothing (first phase)
**Requirements**: HARDEN-01, HARDEN-02, HARDEN-03, HARDEN-04, HARDEN-05, HARDEN-06
**Success Criteria** (what must be TRUE):
  1. Desconectar el micrófono (o que otra app lo bloquee) durante la grabación muestra un error claro, no crashea la app.
  2. Un fallo de transcripción no deja muerta la transcripción del resto de la sesión.
  3. El modelo se descarga verificando integridad (SHA256) desde una URL pinneada; un primer arranque sin conexión muestra un estado accionable.
  4. Un fallo de transcripción (segmento o tail) se le comunica al usuario en vez de descartarse callado.
  5. La app deja un log en disco que permite diagnosticar un fallo reportado por un usuario.
**Plans**: 4 plans (3 waves)

Plans:
- [x] 01-01-PLAN.md — Fundación: deps (parking_lot, tauri-plugin-log, log) + registro del logger a disco (HARDEN-06)
- [x] 01-02-PLAN.md — Hardening backend: mic no crashea (HARDEN-01) + MODEL_CACHE parking_lot (HARDEN-02) + eprintln→log (HARDEN-06)
- [x] 01-03-PLAN.md — commands.rs: integridad SHA256+pin (HARDEN-03) + check_model_status (HARDEN-04) + transcribe-warning (HARDEN-05) + eprintln→log (HARDEN-06)
- [x] 01-04-PLAN.md — Frontend: banner de modelo ausente (HARDEN-04) + aviso no bloqueante de fallo parcial (HARDEN-05) + checkpoint humano

### Phase 2: Compatibilidad honesta (Intel + Apple Silicon)
**Goal**: La app se instala y funciona bien tanto en Intel como en Apple Silicon, sin ofrecer features que no existen en cada hardware y eligiendo un modelo razonable por máquina.
**Depends on**: Phase 1
**Requirements**: COMPAT-01, COMPAT-02, COMPAT-03, COMPAT-04, COMPAT-05
**Success Criteria** (what must be TRUE):
  1. El DMG de Intel (x86_64) se construye en CI y arranca en un Mac Intel.
  2. El motor Apple solo se ofrece donde funciona (Apple Silicon + macOS 26); en Intel no aparece o aparece deshabilitado con explicación.
  3. Al primer arranque, el modelo por defecto se ajusta al hardware (Intel/poca RAM → liviano; Silicon → turbo o Apple) y los threads no exceden los cores reales.
  4. El widget no muestra el flash de forma-notch en pantallas sin notch.
**Plans**: 3 plans (2 waves)

Plans:
- [ ] 02-01-PLAN.md — COMPAT-01: build x86_64 del sidecar (cross-compile) + CI macos-15 + dual-triple
- [ ] 02-02-PLAN.md — COMPAT-02/03/04 backend: gate motor Apple, default por hardware, threads clamp
- [ ] 02-03-PLAN.md — COMPAT-02 UI (disabled_reason) + COMPAT-05 fix flash notch (checkpoint)

### Phase 3: Firma, notarización y repo público
**Goal**: Un usuario puede descargar, instalar y actualizar la app sin advertencias de Gatekeeper ni comandos de terminal, y el repositorio está listo para abrirse como open source.
**Depends on**: Phase 1
**Requirements**: DIST-01, DIST-02, DIST-03, DIST-04, DIST-05
**Success Criteria** (what must be TRUE):
  1. El DMG está firmado con Developer ID y notarizado/stapleado: se instala sin `xattr` ni right-click-open.
  2. El updater instala de verdad una versión nueva, o el flujo de updates es honesto (check-only sin dep falsa).
  3. El repo público no expone `.planning/`/`dev/`, tiene LICENSE MIT y un README de cara al público.
  4. La versión de la app es consistente en todos los manifiestos.
**Plans**: TBD

Plans:
- [ ] 03-01: TBD

### Phase 4: Métricas y crash reporting opt-in
**Goal**: El desarrollador puede entender uso y fallos en producción sin recoger jamás contenido transcrito, con consentimiento explícito del usuario.
**Depends on**: Phase 1
**Requirements**: METRICS-01, METRICS-02, METRICS-03
**Success Criteria** (what must be TRUE):
  1. Con telemetría activada (opt-in), llegan eventos de conteo (activaciones, modelo, errores) pero nunca texto/audio ni PII.
  2. Un crash en producción genera un reporte (opt-in) con PII desactivada y contenido scrubeado.
  3. El onboarding explica honestamente qué se recoge y qué nunca, y la telemetría está apagada por defecto.
**Plans**: TBD

Plans:
- [ ] 04-01: TBD

### Phase 5: Pulido
**Goal**: La calidad del código y del pipeline de CI está a la altura de un proyecto open source que recibirá contribuciones y escrutinio.
**Depends on**: Phase 1
**Requirements**: POLISH-01, POLISH-02, POLISH-03
**Success Criteria** (what must be TRUE):
  1. Cada PR corre build + tests + svelte-check automáticamente.
  2. Las rutas críticas hoy sin probar (recording/transcription, whisper_backend, vad) tienen tests.
  3. `commands.rs` está partido en módulos enfocados (paste, models, recording).
**Plans**: TBD

Plans:
- [ ] 05-01: TBD

## Progress

**Execution Order:** 1 → 2 → 3 → 4 → 5 (F2–F5 dependen solo de F1; F2/F3 son los que más mueven la aguja del lanzamiento)

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Blindaje de producción | 4/4 | ✅ Done | 2026-06-15 |
| 2. Compatibilidad honesta | 0/3 | ⬜ Not started | — |
| 3. Firma, notarización y repo público | 0/? | ⬜ Not started | — |
| 4. Métricas y crash reporting opt-in | 0/? | ⬜ Not started | — |
| 5. Pulido | 0/? | ⬜ Not started | — |
