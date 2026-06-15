---
phase: 01-blindaje-de-producci-n
plan: 02
subsystem: backend-rust
tags: [harden, audio, whisper, logging, parking_lot, mpsc, cpal]
dependency_graph:
  requires: [01-01]
  provides: [HARDEN-01, HARDEN-02, HARDEN-06-partial]
  affects: [src-tauri/src/audio.rs, src-tauri/src/whisper_backend.rs, src-tauri/src/escape.rs, src-tauri/src/shortcut.rs]
tech_stack:
  added: [parking_lot::Mutex (ya en Cargo desde plan 01-01)]
  patterns: [mpsc::channel para propagar Result desde thread spawneado, parking_lot sin poisoning, log:: macros via tauri-plugin-log]
key_files:
  created: []
  modified:
    - src-tauri/src/audio.rs
    - src-tauri/src/whisper_backend.rs
    - src-tauri/src/escape.rs
    - src-tauri/src/shortcut.rs
decisions:
  - "mpsc::channel elegido sobre Arc<Mutex<Option<Error>>> para propagar el Result del stream — más limpio y explícito (Option A del RESEARCH)"
  - "parking_lot::Mutex reemplaza solo MODEL_CACHE; los otros mutexes (CAPTURE, COMMITTED_TEXT, STREAM_HANDLE) se difieren a Phase 5 per CONTEXT.md"
  - "stream.play() error se loguea con log::error! y retorna del thread en vez de paniquear; no se señala por canal porque la falla es posterior al Ok(())"
metrics:
  duration: "~15 min"
  completed: "2026-06-15T18:47:46Z"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 4
  commits: 3
---

# Phase 01 Plan 02: Audio Hardening + Mutex Poison Prevention Summary

**One-liner:** Canal mpsc propaga errores cpal del thread spawneado a start(); parking_lot::Mutex elimina el riesgo de envenenamiento de MODEL_CACHE; 5 eprintln! en 4 archivos backend convertidos a macros log::.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | HARDEN-01 — mpsc channel para errores de stream de audio | e83a91c | src-tauri/src/audio.rs |
| 2 | HARDEN-02 + HARDEN-06 — parking_lot en MODEL_CACHE y log:: en whisper_backend | 9f8f7a7 | src-tauri/src/whisper_backend.rs |
| 3 | HARDEN-06 — eprintln! a log:: en escape.rs y shortcut.rs | 29f3ed8 | src-tauri/src/escape.rs, src-tauri/src/shortcut.rs |

## What Was Built

### HARDEN-01: Propagación de errores de stream vía canal mpsc (audio.rs)

**Problema estructural resuelto:** Los `.expect()` en `audio.rs:73,75` vivían dentro de `std::thread::spawn`. Un panic ahí NO se propagaba a `start()` — mataba el thread en silencio y `start()` devolvía `Ok` con un stream muerto.

**Solución implementada:**
- Se añade `let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();` antes del spawn
- `build_input_stream` se convierte en `match`: `Ok(s)` envía `tx.send(Ok(())).ok()` y continúa; `Err(e)` mapea el error a un mensaje en español, envía `tx.send(Err(msg)).ok()` y retorna del thread
- `stream.play()` usa `if let Err(e)` con `log::error!` + `return` (no señala el canal pues el stream ya se construyó bien)
- Después del spawn: `rx.recv().map_err(...)?.map_err(...)?` bloquea brevemente hasta recibir el resultado — si falla, `start()` devuelve `Err` con mensaje claro en español al caller

**Mensajes en español implementados:**
- `DeviceNotAvailable` → "El micrófono dejó de estar disponible. Revisa que no esté en uso por otra app."
- `StreamConfigNotSupported` → "El micrófono no admite la configuración solicitada."
- otros → `format!("Error al configurar el micrófono: {e}")`

**Error callback per-chunk:** `eprintln!("cpal error: {err}")` → `log::error!("[audio] cpal stream error: {err}")` (no es panic, solo log)

### HARDEN-02: parking_lot::Mutex en MODEL_CACHE (whisper_backend.rs)

**Cambio mínimo y seguro:**
- `use std::sync::Mutex;` → `use parking_lot::Mutex;`
- La declaración `static MODEL_CACHE: Mutex<Option<...>> = Mutex::new(None);` no cambia — `parking_lot::Mutex::new` es `const fn` desde 0.12
- Los dos `.lock().unwrap()` → `.lock()` (parking_lot devuelve el guard directamente, sin `Result`)

**Efecto:** Un panic durante inferencia de Whisper ya no puede envenenar MODEL_CACHE. La siguiente transcripción de la sesión funciona normalmente.

### HARDEN-06: 5 eprintln! convertidos a log:: (4 archivos)

| Archivo | Línea | Antes | Después |
|---------|-------|-------|---------|
| audio.rs | 70 | `eprintln!("cpal error: {err}")` | `log::error!("[audio] cpal stream error: {err}")` |
| whisper_backend.rs | 33 | `eprintln!("[voz-local] loading model: ...")` | `log::info!("[voz-local] loading model: ...")` |
| whisper_backend.rs | 85 | `eprintln!("[voz-local][timing] ...")` | `log::info!("[voz-local][timing] ...")` |
| escape.rs | 21 | `eprintln!("[escape] not on main thread...")` | `log::warn!("[escape] not on main thread...")` |
| shortcut.rs | 22,35 | `eprintln!("[shortcut] start_recording error: {e}")` | `log::error!("[shortcut] start_recording error: {e}")` |

Los logs de timing de Whisper se conservan con el mismo format string — solo cambia la macro.

## Verification

```
cargo build (src-tauri) — exit 0 tras cada task y al final del plan
```

- audio.rs: sin `.expect("failed to build input stream")`, sin `.expect("failed to start stream")`, sin `eprintln!`; contiene `mpsc::channel`, `rx.recv`, "El micrófono dejó de estar disponible"
- whisper_backend.rs: `use parking_lot::Mutex;`, sin `use std::sync::Mutex;`, sin `MODEL_CACHE.lock().unwrap()`, sin `eprintln!`
- escape.rs: sin `eprintln!`, contiene `log::warn!("[escape] not on main thread`
- shortcut.rs: sin `eprintln!`, contiene exactamente 2 ocurrencias de `log::error!("[shortcut] start_recording error`

## Deviations from Plan

None — plan ejecutado exactamente como se especificó.

La única nota: el grep del verify script de Task 2 (`grep -q 'log::info!("\[voz-local\]\[timing\]'`) falló por escaping del shell en la cadena `&&` compuesta; la inspección directa del archivo y el build confirmaron que el contenido es correcto. No fue una desviación del plan.

## Known Stubs

None — todos los cambios son transformaciones directas de código existente sin stubs ni placeholders.

## Threat Flags

Ninguna nueva superficie de seguridad introducida. Este plan elimina vectores de DoS existentes (T-01-03, T-01-04) sin añadir endpoints, rutas de auth, ni acceso a archivos nuevos.

## Self-Check

### Checks

| Item | Result |
|------|--------|
| `src-tauri/src/audio.rs` existe y contiene `mpsc::channel` | FOUND |
| `src-tauri/src/whisper_backend.rs` contiene `use parking_lot::Mutex;` | FOUND |
| `src-tauri/src/escape.rs` contiene `log::warn!` | FOUND |
| `src-tauri/src/shortcut.rs` contiene `log::error!` x2 | FOUND |
| commit e83a91c existe | FOUND |
| commit 9f8f7a7 existe | FOUND |
| commit 29f3ed8 existe | FOUND |
| `cargo build` exit 0 | PASSED |

## Self-Check: PASSED
