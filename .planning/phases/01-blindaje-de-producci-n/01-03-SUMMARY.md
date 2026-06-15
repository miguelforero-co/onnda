---
phase: 01-blindaje-de-producci-n
plan: "03"
subsystem: backend-rust
tags: [integrity, sha256, url-pinning, model-status, transcribe-warning, logging]
dependency_graph:
  requires: ["01-01", "01-02"]
  provides: ["download_model con SHA256 streaming + URLs pinneadas", "check_model_status command", "transcribe-warning events", "eprintln! convertidos"]
  affects: ["src-tauri/src/commands.rs", "src-tauri/src/lib.rs"]
tech_stack:
  added: ["sha2::{Sha256, Digest} (primer uso en source)", "log::warn!/info!/debug! macros en commands.rs"]
  patterns: ["streaming SHA256 via hasher.update() en chunk loop", "match exhaustivo en streaming loop para propagación de errores no fatales"]
key_files:
  modified:
    - src-tauri/src/commands.rs
    - src-tauri/src/lib.rs
decisions:
  - "URLs pinneadas a commit SHA inmutable de HF (no /resolve/main/) para HARDEN-03"
  - "flush().await antes de drop(file) antes de rename para garantizar integridad OS-buffer en macOS/APFS"
  - "transcribe-warning no aborta el loop de streaming — acumula segmentos exitosos"
  - "check_model_status reutiliza lógica primary/fallback existente de stop_and_transcribe_internal"
  - "eprintln! de samples/rms → log::debug! (muy verboso); summary de streaming → log::info!"
metrics:
  duration_minutes: 35
  completed_date: "2026-06-15T18:54:48Z"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 2
---

# Phase 01 Plan 03: HARDEN-03/04/05/06 Summary

**One-liner:** SHA256 streaming con URLs pinneadas a commit HF + `check_model_status` command + `transcribe-warning` en drops silenciosos + 2 `eprintln!` → `log::`.

## What Was Built

### Task 1 — HARDEN-03: URLs pinneadas + SHA256 en streaming (commit `5ba7ff8`)

`download_model` reescrito completamente:

- Las 4 URLs ahora apuntan a revisiones de commit inmutables de HuggingFace (no `/resolve/main/`)
- `Sha256::new()` creado antes del loop; `hasher.update(&chunk)` alimentado en cada chunk antes de `write_all`
- `file.flush().await` + `drop(file)` antes de `rename` (Pitfall 6: tokio File no flushea en drop)
- `hasher.finalize()` comparado contra `expected_sha256`; en mismatch: `tokio::fs::remove_file(&tmp)` y `Err` con mensaje en español — nunca se renombra el `.tmp` corrupto
- Limpieza de `.tmp` previo al inicio de cada descarga
- Errores de red mid-chunk también borran el `.tmp` via `std::fs::remove_file`

Hashes pinneados:
- `large-v3-turbo`: commit `0b364b5…` / SHA256 `317eb69c…`
- `base`: commit `80da2d8…` / SHA256 `60ed5bc3…`
- `small`: commit `80da2d8…` / SHA256 `1be3a9b2…`
- `medium`: commit `80da2d8…` / SHA256 `6c14d5ad…`

### Task 2 — HARDEN-04: comando `check_model_status` (commit `926a763`)

- `pub struct ModelStatus { pub ready: bool, pub model_id: String }` añadido
- `pub fn check_model_status<R: Runtime>(app: AppHandle<R>) -> ModelStatus` añadido: reutiliza la misma lógica `primary.exists() || fallback.exists()` que `stop_and_transcribe_internal`; Apple model siempre retorna `ready: true`
- Registrado en `lib.rs` dentro de `tauri::generate_handler![...]`

### Task 3 — HARDEN-05 + HARDEN-06: `transcribe-warning` + conversión `eprintln!` (commit `1b43b3e`)

**Sitio 1 — loop de streaming (~línea 238):**
- `if let Ok(Ok(t))` reemplazado por `match text { Ok(Ok(t)) => ..., Ok(Err(e)) => log::warn! + emit transcribe-warning, Err(e) => log::warn! + emit transcribe-warning }`
- `COMMITTED_SAMPLES.fetch_add` se mantiene en todos los caminos — nunca se reprocesa audio

**Sitio 2 — tail con segmentos comprometidos (~línea 399):**
- `Ok(Err(e))` y `Err(e)`: si `committed_text` está vacío → `transcribe-error` + `return` (comportamiento anterior preservado); si no está vacío → `log::warn!` + `emit("transcribe-warning", "El cierre del dictado tuvo un error parcial")` + `String::new()`

**HARDEN-06 — 2 `eprintln!` convertidos:**
- `eprintln!` de samples/rate/rms → `log::debug!`
- `eprintln!` del summary de streaming → `log::info!`

Total `transcribe-warning` emits: 4 (2 en streaming loop + 2 en tail).

## Deviations from Plan

None — plan executed exactly as written. El conteo de `transcribe-warning` es 4 en lugar del mínimo 3 del plan porque ambas ramas error del tail (Ok(Err) y Err) emiten cada una, tal como especificaba la acción del task.

## Threat Surface Scan

| Flag | File | Description |
|------|------|-------------|
| T-01-05 mitigated | src-tauri/src/commands.rs | SHA256 streaming verificado; .tmp nunca renombrado en mismatch |
| T-01-06 mitigated | src-tauri/src/commands.rs | URL flotante /resolve/main/ eliminada; pinneada a commit SHA inmutable |
| T-01-07 preserved | src-tauri/src/commands.rs | allowlist match model_id.as_str() con other => Err conservado |
| T-01-08 mitigated | src-tauri/src/commands.rs | log::warn!/info!/debug! con format args — no concatenación de texto transcrito |

## Self-Check: PASSED

- `src-tauri/src/commands.rs`: FOUND (modified)
- `src-tauri/src/lib.rs`: FOUND (modified)
- commit `5ba7ff8`: FOUND
- commit `926a763`: FOUND
- commit `1b43b3e`: FOUND
- `cargo build`: exit 0
- `grep -c 'transcribe-warning' src/commands.rs`: 4 (>= 3)
- `grep -c 'eprintln!' src/commands.rs`: 0
- `grep -q 'expected_sha256'`: OK
- `grep -q '317eb69c...'`: OK
- `! grep -q 'resolve/main/'`: OK
