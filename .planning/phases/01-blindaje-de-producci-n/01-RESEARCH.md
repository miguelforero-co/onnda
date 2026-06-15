# Phase 1: Blindaje de producción — Research

**Researched:** 2026-06-15
**Domain:** Rust / Tauri 2 error hardening: cpal audio, mutex poisoning, model integrity, UX feedback, structured logging
**Confidence:** HIGH (all code anchors read from actual source; external facts verified via installed crate source + live HF headers)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- HARDEN-01: `audio.rs:73,75` `.expect()` → `?` propagation toward `Result<(), String>`. Spanish user messages.
- HARDEN-02: Replace `std::sync::Mutex` of `MODEL_CACHE` with `parking_lot::Mutex`. Audit other `.lock().unwrap()` in transcription path.
- HARDEN-03: Use `sha2` dep (already declared) for streaming SHA256 verification. Pin URL to HF commit SHA. Delete `.tmp` on mismatch.
- HARDEN-04: Proactive first-run offline detection. Reuse existing download flow. Don't redesign the download UX.
- HARDEN-05: Emit error events on silent-drop paths in the streaming loop and tail assembly.
- HARDEN-06: `tauri-plugin-log` (preferred). Rotating file in app-data dir. Convert 8 `eprintln!` sites. Keep `[timing]` logs.

### Claude's Discretion
- HARDEN-04: Where exactly the "modelo ausente / sin conexión" banner lives (onboarding existing vs. Home vs. widget).
- HARDEN-05: UX exacta del aviso no-bloqueante de error parcial.
- HARDEN-06: Nivel por defecto, política de rotación, si exponer "abrir carpeta de logs" en Ajustes.

### Deferred Ideas (OUT OF SCOPE)
- Crash reporting remoto (Sentry/GlitchTip) → Phase 4
- Tests → Phase 5
- Refactor / partir commands.rs → Phase 5
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| HARDEN-01 | La app no crashea si el micrófono se desconecta, está bloqueado o es denegado durante la grabación | cpal 0.15.3 `BuildStreamError`/`PlayStreamError` ambos impl `Display`+`Error` → `map_err(|e| e.to_string())` basta |
| HARDEN-02 | Un panic durante una transcripción no inutiliza el resto de la sesión | `parking_lot 0.12.x` no tiene poisoning; `Mutex::lock()` devuelve guard directamente |
| HARDEN-03 | El modelo descargado se verifica por SHA256 y la URL está pinneada a un commit estable | SHA256 verificados via HF LFS ETag headers; patrón streaming con `sha2 0.11` documentado |
| HARDEN-04 | En primer arranque sin modelo / sin conexión, el usuario ve estado claro y accionable | Detección reactiva ya existe en `commands.rs:332–340`; extensión proactiva en `setup()` es mínima |
| HARDEN-05 | Fallos de transcripción (segmento o tail) se muestran al usuario en vez de descartarse en silencio | Spots exactos identificados: `commands.rs:238` y `commands.rs:399–412` |
| HARDEN-06 | La app escribe logs rotativos a disco | `tauri-plugin-log 2.8.0` + `log = "0.4"`; sin JS companion requerido para Rust-only |
</phase_requirements>

---

## Summary

Esta investigación cubre los 6 hardening fixes de la Phase 1. Todo el trabajo es in-place en los tres archivos de Rust (`audio.rs`, `whisper_backend.rs`, `commands.rs`) más `lib.rs` y `Cargo.toml`. No se introducen arquitecturas nuevas.

Los dos crashers bloqueantes (HARDEN-01 y HARDEN-02) son cambios de 2-5 líneas cada uno: convertir `.expect()` a `map_err` y cambiar el tipo de mutex. HARDEN-03 es el más elaborado (streaming SHA256 + URL pinning) pero la infraestructura ya existe (`sha2`, `reqwest` con `features=["stream"]`, el patrón `.tmp`→rename). HARDEN-04 y HARDEN-05 añaden eventos Tauri a rutas silenciosas existentes. HARDEN-06 añade un plugin con ~10 líneas de setup y convierte 8 `eprintln!` en macros `log::`.

**Primary recommendation:** Trabajar HARDEN-01 y HARDEN-02 primero (sin dependencias nuevas, sin riesgo de regresión), luego HARDEN-06 (requiere añadir el plugin pero no toca lógica), después HARDEN-03 (cambio más delicado), y HARDEN-04/05 en paralelo al final (tocan frontend).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Audio capture error handling | Rust/Backend | — | cpal stream vive en un thread nativo; los errores se propagan hacia `start_recording_internal` |
| Mutex poisoning prevention | Rust/Backend | — | `MODEL_CACHE` es un static Rust; no hay representación frontend |
| Model integrity check | Rust/Backend | — | La descarga y el SHA256 ocurren en el backend; el frontend ya escucha `download-complete` |
| First-run offline UX | Rust/Backend + Frontend | SvelteKit | Backend detecta; frontend muestra el banner/modal. Reutiliza canal `transcribe-error` o nuevo evento |
| Transcription failure visibility | Rust/Backend | Widget/Main SvelteKit | Backend emite el evento; widget y main ya escuchan `transcribe-error` |
| Disk logging | Rust/Backend | — | `tauri-plugin-log` es Rust-only para logging estructurado; el JS companion es opcional |

---

## Standard Stack

### Core (verified against Cargo.lock and crates.io 2026-06-15)

| Library | Version in project | Purpose | Notes |
|---------|-------------------|---------|-------|
| cpal | 0.15.3 (locked) | Audio capture | Ya presente; solo cambiar `.expect()` |
| parking_lot | 0.12.5 (crates.io latest) | Non-poisoning Mutex | Nueva dep a añadir |
| sha2 | 0.11.0 (Cargo.lock) | SHA256 streaming | Ya en Cargo.toml pero sin uso |
| tauri-plugin-log | 2.8.0 (crates.io latest) | Rotating file log | Nueva dep a añadir |
| log | 0.4 | Macro facade `log::warn!` etc. | Nueva dep a añadir con plugin |

**Version verification:** [VERIFIED: cargo search + Cargo.lock inspection]
- `cpal = "0.15.3"` — resuelto en Cargo.lock
- `parking_lot = "0.12.5"` — última versión en crates.io 2026-06-15
- `sha2 = "0.11.0"` — ya en Cargo.lock (declarado pero sin imports en source)
- `tauri-plugin-log = "2.8.0"` — última versión en crates.io 2026-06-15

**Cargo.toml additions:**
```toml
parking_lot = "0.12"
tauri-plugin-log = "2"
log = "0.4"
```

---

## HARDEN-01: cpal Error Handling (audio.rs)

### Exact error types in cpal 0.15.3 [VERIFIED: ~/.cargo/registry/src/.../cpal-0.15.3/src/error.rs]

**`BuildStreamError`** — returned by `device.build_input_stream(...)`:
```rust
pub enum BuildStreamError {
    DeviceNotAvailable,       // mic physically disconnected or unavailable
    StreamConfigNotSupported, // config (sample rate, channels) not supported by device
    InvalidArgument,          // calling capture on output-only device
    StreamIdOverflow,         // integer overflow on stream ID counter
    BackendSpecific { err: BackendSpecificError }, // OS/driver error with description
}
```
All variants impl `Display` and `Error`. The `.to_string()` path works cleanly.

**`PlayStreamError`** — returned by `stream.play()`:
```rust
pub enum PlayStreamError {
    DeviceNotAvailable,       // device disappeared after stream was built
    BackendSpecific { err: BackendSpecificError },
}
```
Note from cpal docs: "Only macOS may immediately return an error while calling `play()`" — exactly our target platform.

### The structural problem

The `.expect()` calls at `audio.rs:73` and `audio.rs:75` are inside `std::thread::spawn(move || {...})`. A panic inside a spawned thread does NOT propagate to the caller — instead it silently terminates that thread and the main code tries to use the now-dead audio stream. The `AudioCapture::start()` function signature already returns `Result<Self>` (anyhow), but the spawn closure drops errors.

### Fix pattern

The thread must capture errors and signal them back. Two options:

**Option A (simplest): Channel-based**
```rust
// In audio.rs — replace the thread::spawn block
use std::sync::mpsc;

pub fn start(on_level: impl Fn(f32, Vec<f32>) + Send + 'static) -> Result<Self> {
    use cpal::traits::{DeviceTrait, HostTrait};
    // ... (device/config setup stays the same) ...

    let (tx, rx) = mpsc::channel::<Result<(), String>>();
    let samples_thread = Arc::clone(&samples);
    let stop_thread = Arc::clone(&stop);

    let thread = std::thread::spawn(move || {
        use cpal::traits::StreamTrait;
        let stream = device
            .build_input_stream(...)
            .map_err(|e| match e {
                cpal::BuildStreamError::DeviceNotAvailable =>
                    "El micrófono dejó de estar disponible. Revisa que no esté en uso por otra app.".to_string(),
                cpal::BuildStreamError::StreamConfigNotSupported =>
                    "El micrófono no admite la configuración solicitada.".to_string(),
                e => format!("Error al configurar el micrófono: {e}"),
            });
        
        let stream = match stream {
            Ok(s) => { tx.send(Ok(())).ok(); s }
            Err(e) => { tx.send(Err(e)).ok(); return; }
        };
        
        match stream.play() {
            Ok(()) => {}
            Err(e) => {
                // Stream started but play failed — log, don't panic
                log::error!("[audio] stream.play() error: {e}");
                return;
            }
        }
        
        while !stop_thread.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    // Block briefly to get the stream-build result; play errors are logged
    rx.recv()
        .map_err(|_| anyhow!("El thread de audio terminó inesperadamente"))?
        .map_err(|e| anyhow!("{e}"))?;

    Ok(Self { sample_rate, samples, stop, thread: Some(thread) })
}
```

**Key gotcha:** The `stream` object (of type `cpal::Stream`) holds the audio alive. It MUST be kept in the thread — it cannot be moved to the caller. That's why the `Result<(), String>` signal is needed rather than returning the stream.

**Option B (alternative): Arc<Mutex<Option<Error>>>** — same idea but with shared state instead of channel. Option A is cleaner.

### Error callback (audio.rs:70, currently `eprintln!`)
```rust
// Change from:
|err| eprintln!("cpal error: {err}"),
// To:
|err| log::error!("[audio] cpal stream error: {err}"),
```
This is the per-chunk error callback (not stream-build errors). It runs mid-stream; no panic, just log.

---

## HARDEN-02: parking_lot Mutex (whisper_backend.rs)

### Current state [VERIFIED: read whisper_backend.rs]

`whisper_backend.rs:1` imports `use std::sync::Mutex;`
`whisper_backend.rs:10` declares `static MODEL_CACHE: Mutex<Option<(String, WhisperContext)>> = Mutex::new(None);`
`whisper_backend.rs:25` — `MODEL_CACHE.lock().unwrap()` (in `ensure_loaded`)
`whisper_backend.rs:50` — `MODEL_CACHE.lock().unwrap()` (in `transcribe`)

### parking_lot::Mutex behavior [VERIFIED: docs.rs/parking_lot]

- `parking_lot::Mutex::lock()` returns the guard DIRECTLY (no `Result`) — no poisoning mechanism
- The API is otherwise identical to `std::sync::Mutex` (same guard, same methods)
- `parking_lot::Mutex::new(...)` as static works fine via `parking_lot::const_mutex` or simply `Mutex::new` in a `static`; parking_lot supports `const fn new()` since 0.12

### Minimal diff

```rust
// whisper_backend.rs — top of file
// Remove:
use std::sync::Mutex;
// Add:
use parking_lot::Mutex;

// Line 25 — before: MODEL_CACHE.lock().unwrap()
// After:
let mut cache = MODEL_CACHE.lock();  // returns MutexGuard<_> directly

// Line 50 — before: MODEL_CACHE.lock().unwrap()
// After:
let cache = MODEL_CACHE.lock();  // same
```

The static declaration doesn't change — `parking_lot::Mutex` has the same API for `Mutex::new(None)`.

### Other `.lock().unwrap()` sites in the transcription path [VERIFIED: commands.rs full read]

| File | Line | Mutex | Risk |
|------|------|-------|------|
| `commands.rs:12` | `CAPTURE: Mutex<Option<AudioCapture>>` | `std::sync` | `.lock().unwrap()` at lines 118, 253, 448, 470 |
| `commands.rs:20` | `COMMITTED_TEXT: Mutex<Vec<String>>` | `std::sync` | `.lock().unwrap()` at lines 113, 241, 355, 470 |
| `commands.rs:22` | `STREAM_HANDLE: Mutex<Option<JoinHandle>>` | `std::sync` | `.lock().unwrap()` at lines 247, 282, 463 |
| `audio.rs:9` | `samples: Arc<Mutex<Vec<f32>>>` | `std::sync` | `.lock().unwrap()` at lines 47, 62, 91 |

**Decision:** CONTEXT.md says to replace `MODEL_CACHE` specifically. The other mutexes in `commands.rs` and `audio.rs` are less critical (they don't sit on the hot path with Whisper inference happening while held), but should be audited. The conservative approach: replace `MODEL_CACHE` as required, and treat the others as `[ASSUMED]` OK for Phase 1 given no panic has been observed on those paths. Phase 5 (refactor) can sweep them.

**Verdict:** Only `whisper_backend.rs` needs `parking_lot` in this phase. No new `use` needed in `commands.rs` for Phase 1.

---

## HARDEN-03: Model Integrity + URL Pinning (commands.rs)

### Current download_model state [VERIFIED: read commands.rs:785–836]

- URLs: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{model}.bin` — floating `main`
- `sha2` dep declared in Cargo.toml:53 but zero uses in source (grep confirms)
- Temp file pattern `.tmp`→rename already in place (lines 798, 834)
- `total = resp.content_length().unwrap_or(0)` but not verified after download

### HuggingFace URL pinning strategy [VERIFIED: live HF API headers]

HuggingFace serves LFS files via redirect. The `X-Linked-ETag` response header IS the SHA256 of the binary content — verified by comparing against the SHA256 shown in the HF web UI.

To pin to an immutable revision, replace `/resolve/main/` with `/resolve/{commit_sha}/`. The SHA256 in `X-Linked-ETag` is the ground truth for that revision.

### Verified hashes and pinned URLs [VERIFIED: curl -sI against live HF API, 2026-06-15]

| Model ID | Filename | Pinned commit | SHA256 (from X-Linked-ETag) | Size (bytes) |
|----------|----------|---------------|-----------------------------|--------------|
| `base` | `ggml-base.bin` | `80da2d8bfee42b0e836fc3a9890373e5defc00a6` | `60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe` | 147,951,465 |
| `small` | `ggml-small.bin` | `80da2d8bfee42b0e836fc3a9890373e5defc00a6` | `1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b` | 487,601,967 |
| `medium` | `ggml-medium.bin` | `80da2d8bfee42b0e836fc3a9890373e5defc00a6` | `6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208` | 1,533,763,059 |
| `large-v3-turbo` | `ggml-large-v3-turbo-q8_0.bin` | `0b364b566045a405be7225ee1e415a073e04da77` | `317eb69c11673c9de1e1f0d459b253999804ec71ac4c23c17ecf5fbe24e259a1` | 874,188,075 |

**Pinned URL pattern:**
```
https://huggingface.co/ggerganov/whisper.cpp/resolve/{commit_sha}/{filename}
```

**Important:** The pinned URL returns a 302 redirect to a CDN URL. `reqwest` follows redirects by default, so no changes needed in the HTTP client setup.

### sha2 0.11 streaming pattern [VERIFIED: sha2-0.11.0 README + digest-0.11.3 source]

`sha2 = "0.11"` resolves to `sha2 0.11.0` in this project. The `Digest` trait (from `digest 0.11.3`) exposes:
- `Sha256::new()` — create hasher
- `hasher.update(&[u8])` — feed bytes (impl `AsRef<[u8]>`)
- `hasher.finalize()` — consume and return fixed-size output (`GenericArray<u8, U32>`)

No `hex` crate is needed — format hex with standard Rust:
```rust
let hash_bytes = hasher.finalize();
let computed = hash_bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
```

### Complete download_model replacement pattern

```rust
use sha2::{Sha256, Digest};

#[tauri::command]
pub async fn download_model<R: Runtime>(app: AppHandle<R>, model_id: String) -> Result<(), String> {
    // Pinned URLs and expected SHA256 hashes (verified 2026-06-15)
    let (url, expected_sha256) = match model_id.as_str() {
        "large-v3-turbo" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/0b364b566045a405be7225ee1e415a073e04da77/ggml-large-v3-turbo-q8_0.bin",
            "317eb69c11673c9de1e1f0d459b253999804ec71ac4c23c17ecf5fbe24e259a1",
        ),
        "base" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/80da2d8bfee42b0e836fc3a9890373e5defc00a6/ggml-base.bin",
            "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        ),
        "small" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/80da2d8bfee42b0e836fc3a9890373e5defc00a6/ggml-small.bin",
            "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
        ),
        "medium" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/80da2d8bfee42b0e836fc3a9890373e5defc00a6/ggml-medium.bin",
            "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
        ),
        other => return Err(format!("Modelo desconocido: {}", other)),
    };

    let dir = models_dir(&app).ok_or("No se pudo obtener el directorio de datos")?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let dest = dir.join(format!("ggml-{}.bin", model_id));
    let tmp  = dir.join(format!("ggml-{}.bin.tmp", model_id));

    // Clean up any leftover .tmp from a previous failed download
    let _ = tokio::fs::remove_file(&tmp).await;

    let client = reqwest::Client::builder()
        .user_agent("voz-local/0.1")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} al descargar el modelo", resp.status()));
    }

    let total = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut hasher = Sha256::new();  // incremental hasher

    let mut file = tokio::fs::File::create(&tmp).await.map_err(|e| e.to_string())?;
    let mut stream = resp.bytes_stream();

    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        hasher.update(&chunk);                                          // feed SHA256
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        let percent = if total > 0 { downloaded as f32 / total as f32 * 100.0 } else { 0.0 };
        app.emit("download-progress", serde_json::json!({
            "model_id": model_id,
            "downloaded_mb": downloaded as f32 / 1_048_576.0,
            "total_mb":      total      as f32 / 1_048_576.0,
            "percent":       percent,
        })).ok();
    }

    drop(file);  // flush and close before rename

    // Verify SHA256 before making the file visible
    let hash_bytes = hasher.finalize();
    let computed = hash_bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();

    if computed != expected_sha256 {
        // Delete the corrupted temp file — never rename it
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(format!(
            "La descarga de '{}' está corrupta (hash incorrecto). Inténtalo de nuevo.",
            model_id
        ));
    }

    tokio::fs::rename(&tmp, &dest).await.map_err(|e| e.to_string())?;
    app.emit("download-complete", &model_id).ok();
    Ok(())
}
```

**Gotcha — `drop(file)` before `hasher.finalize()`:** `tokio::fs::File` does NOT flush on drop in async contexts — call `file.flush().await` before `drop(file)` or use `file.shutdown().await` to ensure all bytes are on disk before the rename. Pattern above uses `drop(file)` which flushes the OS buffer; the hasher doesn't depend on the file flush anyway (SHA256 is computed from the bytes received from the network, not re-read from disk).

**Gotcha — network errors mid-download:** If the stream returns `Err` mid-chunk, we return early and leave `.tmp` on disk. Add cleanup on error path:
```rust
while let Some(chunk) = stream.next().await {
    let chunk = chunk.map_err(|e| {
        let _ = std::fs::remove_file(&tmp);  // sync OK here, we're in async fn
        e.to_string()
    })?;
    // ...
}
```

---

## HARDEN-04: Offline First-Run UX (commands.rs)

### Current model detection [VERIFIED: read commands.rs:323–347]

Model-absent is handled REACTIVELY at `stop_and_transcribe_internal` (line 332–340):
```rust
// commands.rs:332 — already exists, reactive path
app.emit("transcribe-error", "Modelo no encontrado. Descárgalo en Ajustes → Modelos.").ok();
```

Same message at `commands.rs:339` (fallback also absent) and `commands.rs:583` (file transcription path).

### What's missing for HARDEN-04

The reactive path fires only when the user releases the PTT key. A user with no model downloaded and no internet sees the widget flash open → transcribing → error message, which is confusing. The requirement is that at startup (or earliest opportunity) the app informs the user proactively.

### Minimal implementation pattern

**Option A — Startup check command (recommended):**
```rust
// Add to commands.rs
#[tauri::command]
pub fn check_model_status<R: Runtime>(app: AppHandle<R>) -> ModelStatus {
    let settings = settings::load(&app);
    let model_name = if settings.selected_model.is_empty() {
        "large-v3-turbo".to_string()
    } else {
        settings.selected_model.clone()
    };
    if model_name == crate::speech_backend::APPLE_MODEL_ID {
        return ModelStatus { ready: true, model_id: model_name };
    }
    let Some(dir) = models_dir(&app) else {
        return ModelStatus { ready: false, model_id: model_name };
    };
    let primary  = dir.join(format!("ggml-{}.bin", model_name));
    let fallback = dir.join("ggml-base.bin");
    let ready = primary.exists() || fallback.exists();
    ModelStatus { ready, model_id: model_name }
}

#[derive(serde::Serialize)]
pub struct ModelStatus {
    pub ready: bool,
    pub model_id: String,
}
```

**Frontend usage:** Call `invoke("check_model_status")` in `onMount` of `+page.svelte`. If `!ready`, navigate to the models section of onboarding or show a non-blocking banner. The existing download flow (`download_model` command + `download-progress` / `download-complete` events) handles the rest without changes.

**Option B — Startup event:** Emit `model-not-ready` from `setup()` in `lib.rs`. This is trickier because the frontend may not have listeners registered yet when setup runs. A command pulled by the frontend in `onMount` is more reliable.

### Where to show the banner (Claude's Discretion)

The cleanest place given the existing code is the `onboarding` → `models` step that already exists. If `onboarding_done` is true but no model is ready, show a dismissible banner on the Home section (not a modal), linking to the Models view (the `view = "models"` state already exists). This avoids adding a new screen.

### Existing download event chain [VERIFIED: read +page.svelte:81–108]

The frontend already listens to:
- `download-progress` → updates `downloadProgress` state (line 93)
- `download-complete` → refreshes models list and auto-selects (line 96–106)
- `download-error` — NOT listened to currently (gap: HARDEN-03 adds this implicitly via the `Err` return from `download_model` command, which surfaces as a rejected promise in the frontend invoke call)

---

## HARDEN-05: Surface Transcription Failures (commands.rs)

### Silent drop sites [VERIFIED: read commands.rs full, lines 192–413]

**Site 1 — Streaming loop segment failure (commands.rs:238)**
```rust
// Current (line 238):
if let Ok(Ok(t)) = text {
    let t = t.trim().to_string();
    if !t.is_empty() {
        COMMITTED_TEXT.lock().unwrap().push(t);
    }
}
// If text is Ok(Err(e)) or Err(e), it is silently discarded.
```

The `COMMITTED_SAMPLES` marker is advanced regardless (line 237), so the audio is consumed but the transcription result is lost without notification.

**Site 2 — Tail failure with committed segments (commands.rs:399–412)**
```rust
// Current (lines 399–412):
let tail_text = match tail_result {
    Ok(Ok(t)) => t.trim().to_string(),
    Ok(Err(e)) => {
        if committed_text.is_empty() {
            app.emit("transcribe-error", e.to_string()).ok();
            return;           // notified only if nothing was committed
        }
        String::new()         // SILENT drop when there are committed segments
    }
    Err(e) => {
        if committed_text.is_empty() {
            app.emit("transcribe-error", e.to_string()).ok();
            return;
        }
        String::new()         // SILENT drop when there are committed segments
    }
};
```

When `committed_text` is non-empty, a tail transcription failure produces `String::new()` and continues as if nothing happened.

### Frontend event already in use [VERIFIED: widget/+page.svelte:319]

```
await listen<string>("transcribe-error", () => { phase = "error"; scheduleClose(1200); }),
```

The widget listens to `transcribe-error`. The main page does NOT currently listen to it (only `transcription-done`).

### Minimal fix pattern

**Site 1 — streaming loop:**
```rust
// commands.rs around line 238 — replace the if let block:
COMMITTED_SAMPLES.fetch_add(cut, Ordering::SeqCst);
match text {
    Ok(Ok(t)) => {
        let t = t.trim().to_string();
        if !t.is_empty() {
            COMMITTED_TEXT.lock().unwrap().push(t);
        }
    }
    Ok(Err(e)) => {
        log::warn!("[voz-local] streaming segment failed: {e}");
        // Emit a non-fatal warning — don't abort, continue accumulating
        app_for_stream.emit("transcribe-warning", "Parte del dictado no pudo procesarse").ok();
    }
    Err(e) => {
        log::warn!("[voz-local] streaming segment JoinError: {e}");
        app_for_stream.emit("transcribe-warning", "Parte del dictado no pudo procesarse").ok();
    }
}
```

**Site 2 — tail with committed segments:**
```rust
// commands.rs around line 399:
Ok(Err(e)) => {
    log::warn!("[voz-local] tail transcription failed: {e}");
    if committed_text.is_empty() {
        app.emit("transcribe-error", e.to_string()).ok();
        return;
    }
    // We have committed segments — continue but warn user
    app.emit("transcribe-warning", "El cierre del dictado tuvo un error parcial").ok();
    String::new()
}
```

**New event: `transcribe-warning`**
A non-blocking toast in the widget (and optionally the main page). The widget already handles `transcribe-error` with a close; `transcribe-warning` should show a brief non-closing indicator (e.g., yellow dot or message) that auto-fades without closing the transcription result.

**Claude's Discretion (UX):** The widget phase transitions can stay as `"done"` for successful transcription even with a warning. A small visual indicator (color change, icon) is enough — the user already sees the pasted text.

---

## HARDEN-06: Disk Logging via tauri-plugin-log (lib.rs, Cargo.toml)

### Plugin facts [VERIFIED: docs.rs/tauri-plugin-log/2.8.0, v2.tauri.app/plugin/logging/]

- **Crate:** `tauri-plugin-log = "2"` (resolves to 2.8.0)
- **JS companion:** `@tauri-apps/plugin-log` — optional, only needed if the frontend also wants to send log messages. For Phase 1 (Rust-only conversion of `eprintln!`), the JS companion is NOT required.
- **Log macro facade:** Requires `log = "0.4"` in Cargo.toml for `log::warn!`, `log::error!`, `log::info!` to compile.
- **macOS log dir:** `~/Library/Logs/{bundle_identifier}/` — accessible when launched from Finder (unlike stderr which is swallowed)
- **Capabilities:** `log:default` must be added to `src-tauri/capabilities/default.json`

### Registration in lib.rs

```rust
// lib.rs — add import at top
use tauri_plugin_log::{Target, TargetKind, RotationStrategy};

// Inside pub fn run(), BEFORE other plugins (log should be first):
.plugin(
    tauri_plugin_log::Builder::new()
        .level(log::LevelFilter::Info)
        .target(Target::new(TargetKind::LogDir {
            file_name: Some("voz-local".to_string()),
        }))
        .max_file_size(5_000_000)  // 5 MB per file
        .rotation_strategy(RotationStrategy::KeepOne)
        .build()
)
```

**`RotationStrategy` gotcha:** `KeepAll` has a known bug on macOS (GitHub issue #1397 in tauri-apps/plugins-workspace) where only 2 log files survive rotation regardless of the setting. Use `KeepOne` (default) for reliability — one current + one rotated file. At 5MB limit that's ~10MB max log storage.

**Init order:** Register the log plugin FIRST in the builder chain so that logs from other plugin inits are captured.

### Capabilities file update

```json
// src-tauri/capabilities/default.json — add to permissions array:
"log:default"
```

### The 8 eprintln! sites [VERIFIED: grep output]

| # | File | Line | Current message | Replacement macro | Level |
|---|------|------|-----------------|------------------|-------|
| 1 | `audio.rs` | 70 | `"cpal error: {err}"` | `log::error!("[audio] cpal stream error: {err}")` | error |
| 2 | `commands.rs` | 288 | `"[voz-local] samples: {}, rate: {}, rms: {:.6}"` | `log::debug!("[voz-local] samples: {}, rate: {}, rms: {:.6}", ...)` | debug |
| 3 | `commands.rs` | 359 | `"[voz-local] streaming: {} committed..."` | `log::info!("[voz-local] streaming: {} committed...", ...)` | info |
| 4 | `escape.rs` | 21 | `"[escape] not on main thread..."` | `log::warn!("[escape] not on main thread; skipping monitor install")` | warn |
| 5 | `shortcut.rs` | 22 | `"[shortcut] start_recording error: {e}"` | `log::error!("[shortcut] start_recording error: {e}")` | error |
| 6 | `shortcut.rs` | 35 | `"[shortcut] start_recording error: {e}"` | `log::error!("[shortcut] start_recording error: {e}")` | error |
| 7 | `whisper_backend.rs` | 33 | `"[voz-local] loading model: {}"` | `log::info!("[voz-local] loading model: {}", self.model_path)` | info |
| 8 | `whisper_backend.rs` | 85 | `"[voz-local][timing] resample=..."` | `log::info!("[voz-local][timing] resample={:?} vad={:?} ...", ...)` | info |

**Note:** The timing log at whisper_backend.rs:85 contains multiple format args. Preserve the exact format string; just change `eprintln!` to `log::info!`.

**Required `use` statement:** Each file that uses `log::` macros does NOT need an explicit `use` — the `log` crate's macros work via the global logger registered by tauri-plugin-log. However, the file must have `log` in scope if using `log::LevelFilter` etc. For `log::warn!`/`log::error!` macros, no import needed beyond `log` being a dependency.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Non-poisoning mutex | Custom `try_lock` recovery loops | `parking_lot::Mutex` | Poisoning semantics eliminated; zero runtime cost |
| SHA256 streaming | Read file after download and hash | `sha2::Sha256::update()` per chunk | Single-pass; no temp file re-read; exact same performance |
| Rotating log files | Custom log file appender with size check | `tauri-plugin-log` with `TargetKind::LogDir` | Handles rotation, file naming, platform log dir, thread safety |
| cpal error messages | Matching on numeric OS error codes | cpal's `Display` impl | All `BuildStreamError` / `PlayStreamError` variants have human-readable messages already |

---

## Common Pitfalls

### Pitfall 1: Thread-spawned `.expect()` — panic silently kills audio, not the app
**What goes wrong:** The thread panics, `JoinHandle::join()` returns `Err` later, but `start_recording_internal` already returned `Ok` — so the UI says "recording" while the thread is dead.
**Why it happens:** `thread::spawn` does not propagate panics; they unwind only within the thread.
**How to avoid:** Use a channel (`mpsc::channel`) to transmit the stream-build result back to `start()` before returning `Ok`.
**Warning signs:** Recording starts (widget shows), but `audio-level` events stop arriving almost immediately.

### Pitfall 2: sha2 0.11 uses `digest` 0.11 trait — breaking if wrong import
**What goes wrong:** `use sha2::Digest` works; `use digest::Digest` also works (re-exported). If you accidentally mix sha2 0.10 and sha2 0.11 (which are already both in the Cargo.lock from other deps), the `Digest` trait impls are from different trait versions and won't unify.
**How to avoid:** Use `use sha2::{Sha256, Digest};` — both come from the same `sha2 0.11` crate. Don't add `digest` as a direct dep.

### Pitfall 3: tauri-plugin-log registered AFTER other plugins — misses early log output
**What goes wrong:** Plugin init logs from `tauri_plugin_autostart`, `tauri_plugin_global_shortcut`, etc. appear only on stderr (invisible from Finder) if log plugin hasn't been registered yet.
**How to avoid:** Register `tauri_plugin_log::Builder::new()...build()` as the FIRST `.plugin()` call in `tauri::Builder::default()`.

### Pitfall 4: KeepAll rotation bug on macOS (GitHub issue #1397)
**What goes wrong:** With `RotationStrategy::KeepAll`, only the 2 most recent log files survive on macOS — older files are deleted despite the name suggesting otherwise.
**How to avoid:** Use `RotationStrategy::KeepOne` which is the documented behavior that actually works.

### Pitfall 5: HuggingFace LFS redirect + SHA256 confusion
**What goes wrong:** The initial HTTP response (302 redirect) has `Content-Length: ~1KB` (the LFS pointer file size), not the actual model size. reqwest follows redirects automatically; the final body is the actual binary. The SHA256 from `X-Linked-ETag` is the hash of the FINAL binary content — the hash we verify.
**How to avoid:** Always follow redirects (reqwest default). The `content_length()` check on the response object will return the FINAL file size (HF CDN sets it). The `x-linked-size` in the 302 headers also gives the expected size. Our pattern hashes the stream bytes received after redirection.

### Pitfall 6: `tokio::fs::File` not fully flushed on `drop` in async contexts
**What goes wrong:** Calling `drop(file)` in an async function doesn't guarantee all OS buffers are flushed before `rename`. In practice on macOS/APFS this tends to be fine, but `rename` after an incomplete write produces a corrupt model.
**How to avoid:** Add `file.flush().await.map_err(|e| e.to_string())?;` before `drop(file)`.

### Pitfall 7: parking_lot Mutex in static requires same init as std
**What goes wrong:** `static MODEL_CACHE: parking_lot::Mutex<Option<...>> = parking_lot::Mutex::new(None);` — `parking_lot::Mutex::new` IS `const fn` since 0.12, so this works at compile time with no changes.
**How to avoid:** Just import `use parking_lot::Mutex;` and the static declaration compiles identically.

---

## Code Examples

### HARDEN-01: Channel-based error propagation from spawned thread
```rust
// Source: pattern derived from cpal 0.15.3 docs + std::sync::mpsc
use std::sync::mpsc;

let (tx, rx) = mpsc::channel::<Result<(), String>>();
let thread = std::thread::spawn(move || {
    use cpal::traits::StreamTrait;
    let stream = match device.build_input_stream(&config.into(), callback, err_fn, None) {
        Ok(s) => { tx.send(Ok(())).ok(); s }
        Err(e) => {
            let msg = match e {
                cpal::BuildStreamError::DeviceNotAvailable =>
                    "El micrófono dejó de estar disponible. Revisa que no esté en uso por otra app.".into(),
                e => format!("Error al configurar el micrófono: {e}"),
            };
            tx.send(Err(msg)).ok();
            return;
        }
    };
    if let Err(e) = stream.play() {
        log::error!("[audio] stream.play() error: {e}");
        return;
    }
    while !stop_thread.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(10));
    }
});
rx.recv().map_err(|_| anyhow!("Thread de audio terminó inesperadamente"))?
    .map_err(|e| anyhow!("{e}"))?;
```

### HARDEN-02: Minimal parking_lot swap
```rust
// whisper_backend.rs — change import only
use parking_lot::Mutex;  // replaces: use std::sync::Mutex;

// These two lines change:
let mut cache = MODEL_CACHE.lock();   // was: MODEL_CACHE.lock().unwrap()
let cache = MODEL_CACHE.lock();       // was: MODEL_CACHE.lock().unwrap()
```

### HARDEN-03: Streaming SHA256 while downloading
```rust
// Source: sha2-0.11.0 README (incremental API)
use sha2::{Sha256, Digest};

let mut hasher = Sha256::new();
// ... in chunk loop:
hasher.update(&chunk);
// ... after loop:
let computed = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect::<String>();
if computed != expected_sha256 {
    let _ = tokio::fs::remove_file(&tmp).await;
    return Err("La descarga está corrupta. Inténtalo de nuevo.".into());
}
```

### HARDEN-06: Plugin registration in lib.rs
```rust
// Source: v2.tauri.app/plugin/logging (verified 2026-06-15)
use tauri_plugin_log::{Target, TargetKind, RotationStrategy};

tauri::Builder::default()
    .plugin(
        tauri_plugin_log::Builder::new()
            .level(log::LevelFilter::Info)
            .target(Target::new(TargetKind::LogDir {
                file_name: Some("voz-local".to_string()),
            }))
            .max_file_size(5_000_000)
            .rotation_strategy(RotationStrategy::KeepOne)
            .build()
    )
    .plugin(tauri_plugin_opener::init())
    // ... rest of plugins
```

---

## Validation Architecture

```json
// .planning/config.json — no explicit nyquist_validation key found; treat as enabled
```

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None installed — `cargo test` with `#[test]` only (Phase 5 adds proper testing) |
| Config file | none |
| Quick run command | `cd src-tauri && cargo check` (compile-time verification) |
| Full suite command | `cd src-tauri && cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Approach | Notes |
|--------|----------|-----------|----------|-------|
| HARDEN-01 | App doesn't crash on mic error | Manual smoke | Launch app, unplug mic mid-record in `tauri dev` | No automated test — mic simulation not feasible in unit tests without mock |
| HARDEN-02 | Poisoned mutex doesn't kill transcription | Manual smoke | `cargo test` compile check sufficient for API change | parking_lot eliminates poison by type — no runtime path to test |
| HARDEN-03 | SHA256 verified, corrupted download rejected | Manual: download a model, verify `.tmp` cleaned up | Unit test for hash mismatch logic is feasible in Phase 5 | |
| HARDEN-04 | First-run offline shows clear state | Manual: remove model dir, launch without internet | Svelte frontend check |  |
| HARDEN-05 | Transcription failures surface to user | Manual: simulate whisper error | `cargo check` for event emission |  |
| HARDEN-06 | Logs appear in `~/Library/Logs/` | Manual: launch from Finder, check log dir | `ls ~/Library/Logs/com.voz-local.*` or bundle ID path |  |

### Wave 0 Gaps
- `cargo check` passes after every edit before committing — use as the per-task gate
- No new test files needed for Phase 1 (Phase 5 owns testing infrastructure)

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | Yes — model ID param in download_model | Allowlist match (already in place: `match model_id.as_str()`) |
| V6 Cryptography | Yes — SHA256 integrity | `sha2` crate (RustCrypto); never hand-rolled |
| V9 Communications | Yes — HTTPS model download | reqwest with `rustls-tls` (already in Cargo.toml) |
| V2/V3 Auth/Session | No — no auth in scope | — |
| V4 Access Control | No — local app, no multi-user | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Corrupted/malicious model file (MITM or CDN compromise) | Tampering | SHA256 verification against pinned hashes; URL pinned to immutable commit |
| Floating URL (`/resolve/main/`) replaced with malicious content | Spoofing/Tampering | Pin to commit SHA in URL |
| Log file injection (control chars in transcription text leaking to log) | Tampering | Use `log::info!` with format args (not string concat) — formatting escapes control chars |

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All HARDEN fixes | ✓ | (existing project compiles) | — |
| `cargo` | Build | ✓ | (project builds) | — |
| Internet (HuggingFace) | HARDEN-03 testing | ✓ | — | Test hash mismatch path offline with corrupted .tmp |
| `~/Library/Logs/` | HARDEN-06 | ✓ | macOS platform | — |

---

## Open Questions

1. **Exact bundle identifier for log directory path**
   - What we know: tauri-plugin-log writes to `~/Library/Logs/{bundle_id}/`
   - What's unclear: The bundle ID is in `tauri.conf.json`; didn't read it — the plugin uses it automatically so no hardcoding needed, but the tester should know where to look
   - Recommendation: Read `src-tauri/tauri.conf.json` → `identifier` field to tell the tester the exact path

2. **Frontend handling of `transcribe-warning` event**
   - What we know: Widget listens to `transcribe-error`; `transcription-done` goes to main page
   - What's unclear: The Claude's Discretion says the planner decides the exact UX for the warning toast
   - Recommendation: A `transcribe-warning` event is emitted from Rust; the planner should decide if it's handled in widget only, or also in the main page (new listener in +page.svelte)

3. **Download error UX when network fails mid-download**
   - What we know: `download_model` returns `Err(String)` which becomes a rejected promise in JS `invoke()`
   - What's unclear: The frontend in +page.svelte:93–108 handles `download-progress` and `download-complete` but not a download error event
   - Recommendation: The `invoke("download_model")` promise rejection should be caught in the frontend; OR add a `download-error` event emission in the Rust before returning the Err so the existing listener pattern is consistent

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `x-linked-etag` from HuggingFace HEAD response is the SHA256 of the final binary content | HARDEN-03 hashes table | If HF uses a different hash (e.g., Xet hash, not SHA256), verification would always fail on valid downloads |
| A2 | The `CAPTURE`, `COMMITTED_TEXT`, `STREAM_HANDLE` mutexes in commands.rs are low-risk for poisoning in Phase 1 | HARDEN-02 | If a panic occurs while any of these are held, the app could enter a bad state — covered in Phase 5 |
| A3 | `RotationStrategy::KeepOne` is correctly implemented on macOS (vs. KeepAll which has issue #1397) | HARDEN-06 | If KeepOne also has bugs, logs may not persist correctly; workaround: check manually after setup |

**Note on A1:** The HF web UI shows the same SHA256 value as the `x-linked-etag` header, and the curl verification returned consistent values. HIGH confidence but flagged because it depends on HF's LFS behavior.

---

## Sources

### Primary (HIGH confidence)
- `~/.cargo/registry/src/*/cpal-0.15.3/src/error.rs` — BuildStreamError, PlayStreamError variant definitions [VERIFIED: read directly]
- `~/.cargo/registry/src/*/sha2-0.11.0/src/lib.rs` + README — Sha256::new(), update(), finalize() API [VERIFIED: read directly]
- `~/.cargo/registry/src/*/digest-0.11.3/src/digest.rs` — Digest trait fn signatures [VERIFIED: read directly]
- `src-tauri/src/*.rs` — all code anchors (lines, variable names, function signatures) [VERIFIED: read directly]
- Live HuggingFace API `curl -sI` — SHA256 hashes via `x-linked-etag` + `x-linked-size` [VERIFIED: verified 2026-06-15]
- [v2.tauri.app/plugin/logging](https://v2.tauri.app/plugin/logging/) — tauri-plugin-log setup, LogDir config [CITED]

### Secondary (MEDIUM confidence)
- [docs.rs/tauri-plugin-log/2.8.0](https://docs.rs/tauri-plugin-log/2.8.0/tauri_plugin_log/) — Builder methods [CITED]
- [docs.rs/cpal/0.15.3](https://docs.rs/cpal/0.15.3/cpal/enum.BuildStreamError.html) — error type documentation [CITED]
- [docs.rs/parking_lot/0.12](https://docs.rs/parking_lot/latest/parking_lot/type.Mutex.html) — lock() returns guard directly [CITED]

### Tertiary (LOW confidence)
- GitHub issue [tauri-apps/plugins-workspace#1397](https://github.com/tauri-apps/plugins-workspace/issues/1397) — KeepAll bug on macOS [LOW: single source, not in official docs]

---

## Metadata

**Confidence breakdown:**
- HARDEN-01 code pattern: HIGH — read actual cpal 0.15.3 source
- HARDEN-02 parking_lot swap: HIGH — trivial API change, documented
- HARDEN-03 SHA256 hashes: HIGH for hash values (live API verification); MEDIUM for the "x-linked-etag = SHA256" interpretation (consistent with HF UI but assumed from pattern)
- HARDEN-04 startup check: HIGH — code path read directly
- HARDEN-05 silent drop sites: HIGH — exact lines identified in source
- HARDEN-06 plugin setup: HIGH — official docs + live crates.io version

**Research date:** 2026-06-15
**Valid until:** 2026-09-15 (stable; tauri-plugin-log and parking_lot versions stable; HF model hashes valid as long as the pinned commits exist)
