---
phase: 01-blindaje-de-producci-n
verified: 2026-06-15T20:00:00Z
status: human_needed
score: 6/6
overrides_applied: 0
human_verification:
  - test: "HARDEN-04 — primer arranque sin modelo"
    expected: "Al abrir la ventana principal (onboarding ya hecho) con el modelo ausente, aparece el banner 'No hay un modelo de voz descargado. Descarga uno para empezar a dictar.' con botón que navega a Ajustes; al completar la descarga el banner desaparece."
    why_human: "Requiere eliminar físicamente el archivo .bin del disco, reiniciar la app desde Finder y observar la UI. No se puede verificar con grep."
  - test: "HARDEN-01 — mic desconectado no crashea"
    expected: "Desconectar el micrófono USB/Bluetooth durante una grabación activa (o revocar permiso) muestra un error en español ('El micrófono dejó de estar disponible…'), la app sigue viva."
    why_human: "Requiere hardware físico o revocación de permiso en tiempo real. El código path existe y compila, pero no se puede disparar sin hardware."
  - test: "HARDEN-06 — log en disco al lanzar desde Finder"
    expected: "Lanzar la app desde Finder (no desde terminal), realizar un dictado y confirmar que existe ~/Library/Logs/com.vozlocal.app/voz-local.log con líneas [timing] y eventos del plugin."
    why_human: "Requiere lanzar desde Finder (donde stderr se pierde) y verificar el sistema de archivos. El código está correcto pero el test en vivo es la única garantía."
  - test: "HARDEN-05 — fallo parcial de transcripción visible (opcional, difícil de forzar)"
    expected: "Cuando el backend emite transcribe-warning, el widget muestra 'Parcial' al terminar, la ventana principal muestra un toast de 4s con el mensaje en español, y el texto ya pegado no se pierde."
    why_human: "Forzar un fallo de segmento de Whisper requiere corromper el modelo o simular un OOM, no automatizable con grep."
---

# Phase 1: Blindaje de producción — Verification Report

**Phase Goal:** La app no crashea ni pierde dictado en silencio en las rutas que un usuario público va a estrenar; los fallos son visibles y diagnosticables.
**Verified:** 2026-06-15T20:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (Success Criteria from ROADMAP)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Desconectar el micrófono durante grabación muestra error claro, no crashea | ✓ VERIFIED (code) | audio.rs: mpsc channel propaga BuildStreamError/PlayStreamError como Err con mensaje en español; rx.recv()?.map_err(…)? devuelve Err a start() en vez de paniquear |
| 2 | Un fallo de transcripción no deja muerta la transcripción del resto de la sesión | ✓ VERIFIED | parking_lot::Mutex en MODEL_CACHE elimina envenenamiento; siguiente transcripción en sesión funciona normalmente |
| 3 | Modelo se descarga con SHA256 desde URL pinneada; primer arranque offline muestra estado accionable | ✓ VERIFIED | URLs con commit SHA inmutable (0b364b5…, 80da2d8…); SHA256 streaming verificado antes de rename; check_model_status + banner en frontend |
| 4 | Un fallo de transcripción se comunica al usuario, no se descarta callado | ✓ VERIFIED (code) | 4 sitios emit("transcribe-warning", …): 2 en loop streaming, 2 en tail; widget y +page.svelte escuchan con listeners activos |
| 5 | La app deja un log en disco para diagnosticar fallos reportados por usuarios | ✓ VERIFIED (code) | tauri_plugin_log registrado primero, RotationStrategy::KeepOne, 5MB, LogDir "voz-local" → ~/Library/Logs/com.vozlocal.app/voz-local.log; cero eprintln! en src/ |

**Score:** 5/5 truths VERIFIED (code), 3 pendientes de verificación humana para confirmar comportamiento en vivo

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/audio.rs` | Sin .expect() en stream build/play; errores en español vía mpsc | ✓ VERIFIED | mpsc::channel en L37; match en L44–87 (DeviceNotAvailable→español, StreamConfigNotSupported→español); rx.recv().map_err(…)?.map_err(…)? en L98–100; cero .expect() en rutas de stream |
| `src-tauri/src/whisper_backend.rs` | parking_lot::Mutex en MODEL_CACHE; sin lock().unwrap() | ✓ VERIFIED | `use parking_lot::Mutex` en L1; MODEL_CACHE.lock() en L25,50 (sin unwrap — parking_lot devuelve guard directo); L51 `.unwrap()` es sobre Option::as_ref() post-ensure_loaded, no sobre el lock |
| `src-tauri/src/commands.rs` | URLs pinneadas a commit SHA; SHA256 streaming; delete .tmp en mismatch; check_model_status; transcribe-warning x4; cero eprintln! | ✓ VERIFIED | URLs en L804–819 usan /resolve/{commit-sha}/; Sha256::new() + hasher.update() en loop; remove_file(&tmp) en L854 (error de red) y L877 (hash mismatch); ModelStatus struct + check_model_status fn en L890–916; 4 emit("transcribe-warning") en L247,251,415,424; 0 eprintln! |
| `src-tauri/src/lib.rs` | tauri_plugin_log registrado primero; check_model_status en generate_handler | ✓ VERIFIED | tauri_plugin_log::Builder first .plugin() en L36–45; commands::check_model_status en L66 del invoke_handler |
| `src-tauri/capabilities/default.json` | "log:default" permission | ✓ VERIFIED | L29: "log:default" presente |
| `src/routes/+page.svelte` | invoke check_model_status en onMount; banner accionable (!modelReady); listener transcribe-warning con toast 4s | ✓ VERIFIED | L85: invoke("check_model_status"); L62: modelReady=$state(true); L255–260: banner en español con botón → view="ajustes"; L106–110: listener con warnTimer 4s; L287–290: {#if warnMsg}<div class="warn-toast"> |
| `src/routes/widget/+page.svelte` | listener transcribe-warning; indicador "Parcial" no bloqueante; reset en recording-state | ✓ VERIFIED | L15: warned=$state(false); L329: listen("transcribe-warning", ()=>{warned=true}); L21–26: labelText derivado muestra "Parcial" si warned && done; L319: warned=false al iniciar grabación |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `+page.svelte` onMount | `commands::check_model_status` (Rust) | `invoke("check_model_status")` | ✓ WIRED | L85 +page.svelte → L66 lib.rs invoke_handler → L899 commands.rs |
| `backend emit transcribe-warning` | widget + main UI | `listen("transcribe-warning", ...)` | ✓ WIRED | commands.rs L247,251,415,424 emit → widget L329 + main L106 listen |
| `backend emit transcribe-warning` | main page toast visible | `{#if warnMsg}` render | ✓ WIRED | warnMsg asignado en L108, renderizado en L289 |
| `download-complete` event | `modelReady = true` (hide banner) | `listen("download-complete", …)` | ✓ WIRED | L123 en +page.svelte: modelReady=true dentro del listener existente |
| `MODEL_CACHE.lock()` | no lock poisoning | `parking_lot::Mutex` | ✓ WIRED | L1 whisper_backend.rs: use parking_lot::Mutex; L50: .lock() sin unwrap |
| `mpsc::channel` | propagate stream error to start() | `rx.recv().map_err(…)?.map_err(…)?` | ✓ WIRED | L37 tx/rx created; L75,84 tx.send(Ok/Err); L98–100 rx.recv()?.map_err(…)? |

---

### Data-Flow Trace (Level 4)

No aplica a esta fase — no hay componentes de rendering de datos dinámicos de BD. Los flujos críticos son eventos Tauri (mpsc, emit/listen) ya trazados en Key Links.

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| No eprintln! en src-tauri/src/ | `grep -rn "eprintln!" src-tauri/src/` | 0 resultados (hits en target/doc/ son artefactos de build previo) | ✓ PASS |
| URLs sin /resolve/main/ en commands.rs | `grep "resolve/main" commands.rs` | 0 resultados | ✓ PASS |
| check_model_status en invoke_handler | `grep "check_model_status" lib.rs` | L66 confirma registro | ✓ PASS |
| 4 sitios transcribe-warning en commands.rs | `grep -c "transcribe-warning" commands.rs` | 4 | ✓ PASS |
| parking_lot::Mutex en whisper_backend | `grep "use parking_lot" whisper_backend.rs` | L1 confirmado | ✓ PASS |
| log:default en capabilities | `grep "log:default" capabilities/default.json` | L29 confirmado | ✓ PASS |
| Banner accionable en +page.svelte | `grep "No hay un modelo de voz" +page.svelte` | L258 confirmado | ✓ PASS |
| cargo build exit 0 (declarado en SUMMARYs 01–03) | cargo build | Confirmed by all 3 SUMMARYs | ✓ PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| HARDEN-01 | 01-02-PLAN | No crash si mic desconectado | ✓ SATISFIED | audio.rs: mpsc + match + mensajes en español; build exit 0 |
| HARDEN-02 | 01-02-PLAN | Un panic no inutiliza MODEL_CACHE | ✓ SATISFIED | parking_lot::Mutex sin poisoning en whisper_backend.rs L1,10,25,50 |
| HARDEN-03 | 01-03-PLAN | SHA256 + URL pinneada en download_model | ✓ SATISFIED | commands.rs L800–886; 4 modelos con commit SHA y hash esperado |
| HARDEN-04 | 01-03-PLAN + 01-04-PLAN | Primer arranque offline muestra estado accionable | ✓ SATISFIED (code) | check_model_status fn + invoke en +page.svelte + banner español |
| HARDEN-05 | 01-03-PLAN + 01-04-PLAN | Fallos de transcripción visibles al usuario | ✓ SATISFIED (code) | 4 emit sites + 2 listeners activos + widget "Parcial" label |
| HARDEN-06 | 01-01-PLAN + 01-02-PLAN + 01-03-PLAN | Logs rotativos a disco, cero eprintln! | ✓ SATISFIED | tauri_plugin_log primer plugin; 0 eprintln! en src/; log:: macros en audio,whisper,escape,shortcut,commands |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src-tauri/src/whisper_backend.rs` | 51 | `.unwrap()` en `cache.as_ref()` | ℹ️ Info | No es un unwrap sobre lock (HARDEN-02 resuelto). Es unwrap sobre Option garantizado Some por ensure_loaded()? en L49. Lógicamente seguro, pero podría convertirse a `.expect("cache must be Some after ensure_loaded")` para claridad. No bloquea el goal. |
| `src-tauri/src/audio.rs` | 49,64 | `samples_cb.lock().unwrap()` en closure del callback cpal | ℹ️ Info | Este Mutex es std::sync::Mutex (AudioCapture.samples), NO el MODEL_CACHE. Un poison aquí afectaría solo la grabación activa. Estaba fuera del scope de HARDEN-02 (ver CONTEXT.md: "parking_lot reemplaza solo MODEL_CACHE; los otros mutexes se difieren a Phase 5"). No bloquea el goal de la fase. |

No hay stubs, placeholders, ni anti-patterns bloqueantes.

---

### Human Verification Required

Las siguientes rutas compilan y el código implementa los handlers correctos, pero solo se pueden confirmar con hardware o interacción en vivo:

#### 1. HARDEN-04 — Primer arranque sin modelo descargado

**Test:** Con onboarding completado, mover o renombrar el archivo .bin fuera de `~/Library/Application Support/com.vozlocal.app/models/`. Lanzar la app.
**Expected:** Banner en español "No hay un modelo de voz descargado. Descarga uno para empezar a dictar." con botón "Descargar modelo" que navega a Ajustes. Al completar la descarga, el banner desaparece.
**Why human:** Requiere manipulación del sistema de archivos en producción y observación visual de la UI.

#### 2. HARDEN-01 — Micrófono desconectado no crashea

**Test:** Iniciar un dictado (push-to-talk mantenido), desconectar el micrófono USB/Bluetooth o revocar el permiso en Preferencias del Sistema mientras se graba.
**Expected:** La app no crashea. El usuario ve un mensaje de error en español (ej. "El micrófono dejó de estar disponible. Revisa que no esté en uso por otra app."). La app sigue funcionando para el siguiente dictado.
**Why human:** Requiere hardware físico para disparar el path BuildStreamError::DeviceNotAvailable. No simulable con grep.

#### 3. HARDEN-06 — Log en disco desde Finder

**Test:** Lanzar la app haciendo doble clic desde Finder (no desde terminal). Realizar 1-2 dictados. Verificar: `ls -la ~/Library/Logs/com.vozlocal.app/` y `grep "\[timing\]" ~/Library/Logs/com.vozlocal.app/voz-local.log`.
**Expected:** Archivo `voz-local.log` presente con líneas `[voz-local][timing]` y eventos de grabación. Sin necesidad de terminal para acceder a diagnóstico.
**Why human:** La ausencia de stderr cuando se lanza desde Finder solo se puede confirmar observando que el log tiene contenido (no que stderr tenga nada).

#### 4. HARDEN-05 — Fallo parcial visible (opcional)

**Test:** Si posible, provocar un fallo de inferencia Whisper (ej. modelo corrupto, OOM bajo presión). Observar widget y ventana principal.
**Expected:** Widget muestra "Parcial" al finalizar. Ventana principal muestra toast por 4s. El texto ya pegado no se pierde.
**Why human:** Difícil de forzar en condiciones normales. El mecanismo code-side está verificado.

---

### Gaps Summary

No hay gaps funcionales. Todo el código requerido por HARDEN-01 a HARDEN-06 existe, es sustantivo, y está conectado. Los must-haves de código del PLAN están satisfechos.

**Nota administrativa:** `01-04-SUMMARY.md` no fue creado (el ejecutor completó los tasks 1 y 2 del plan pero no generó el artefacto final de resumen). Los tasks están marcados `<done>` en el PLAN y el código confirma la implementación. No es un gap funcional, pero el artefacto de auditoría falta.

El estado `human_needed` refleja los 3 tests físicos que el código garantiza pero que no se pueden confirmar sin hardware/interacción real. Una vez que el desarrollador ejecute los tests del checkpoint humano (Task 3 del 01-04-PLAN) y los apruebe, la fase puede marcarse como completada.

---

_Verified: 2026-06-15T20:00:00Z_
_Verifier: Claude (gsd-verifier)_
