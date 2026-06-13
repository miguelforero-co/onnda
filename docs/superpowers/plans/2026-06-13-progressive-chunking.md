# Progressive Chunking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show partial transcription text in the widget every 5 seconds during recording so users never stare at a blank screen, then run a final Whisper pass on key-release using the accumulated partials as `initial_prompt` for context-aware correction.

**Architecture:** A background tokio task runs during recording, snapshotting the last 30s of audio every 5s and transcribing it with `spawn_blocking`. Each result is stored in `PARTIAL_TRANSCRIPTS` and emitted as a `partial-transcript` event. On key-release, the chunk task is stopped, accumulated partials are joined and passed as `initial_prompt` to the final full-audio transcription, giving Whisper full context to correct boundaries and style. The widget displays partial text live and replaces it on `transcription-done`.

**Tech Stack:** Rust/Tauri 2, whisper-rs, tokio, SvelteKit

---

### Task 1: Expose audio buffer from AudioCapture + add chunk task globals

**Files:**
- Modify: `src-tauri/src/transcription.rs` (add `samples_arc()` method)
- Modify: `src-tauri/src/commands.rs` (add 3 new statics)

- [ ] **Step 1: Add `samples_arc()` to AudioCapture in transcription.rs**

In `transcription.rs`, add this method inside the `impl AudioCapture` block, after `start()` and before `stop()`:

```rust
pub fn samples_arc(&self) -> Arc<Mutex<Vec<f32>>> {
    Arc::clone(&self.samples)
}
```

- [ ] **Step 2: Add statics for chunk task in commands.rs**

Add these three statics below the existing `static CAPTURE` line in `commands.rs`:

```rust
static CHUNK_HANDLE: Mutex<Option<tokio::task::JoinHandle<()>>> = Mutex::new(None);
static PARTIAL_TRANSCRIPTS: Mutex<Vec<String>> = Mutex::new(Vec::new());
```

- [ ] **Step 3: Verify it compiles**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```

Expected: `Finished` with no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/transcription.rs src-tauri/src/commands.rs
git commit -m "feat: expose AudioCapture buffer + add chunk task statics"
```

---

### Task 2: Spawn chunk transcription task on recording start

**Files:**
- Modify: `src-tauri/src/commands.rs` — update `start_recording_internal`

- [ ] **Step 1: Replace `start_recording_internal` with the version that spawns chunk task**

Replace the entire `start_recording_internal` function (lines 88-102) with:

```rust
pub fn start_recording_internal<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if IS_RECORDING.load(Ordering::SeqCst) {
        return Ok(());
    }
    let app_clone = app.clone();
    let capture = AudioCapture::start(move |rms| {
        app_clone.emit("audio-level", rms).ok();
    })
    .map_err(|e| e.to_string())?;

    // Clear partials from previous recording
    PARTIAL_TRANSCRIPTS.lock().unwrap().clear();

    // Spawn background chunk transcription task
    let samples_arc = capture.samples_arc();
    let sample_rate = capture.sample_rate;
    let app_for_chunk = app.clone();

    let handle = tokio::spawn(async move {
        let settings = crate::settings::load(&app_for_chunk);
        let language = settings.selected_language.clone();
        let custom_words = settings.custom_words.clone();
        let threshold = settings.word_correction_threshold;

        let model_name = if settings.selected_model.is_empty() {
            "large-v3-turbo".to_string()
        } else {
            settings.selected_model.clone()
        };

        let model_path = match app_for_chunk.path().app_data_dir().ok() {
            Some(dir) => {
                let primary = dir.join("models").join(format!("ggml-{}.bin", model_name));
                let fallback = dir.join("models").join("ggml-base.bin");
                if primary.exists() { primary }
                else if fallback.exists() { fallback }
                else { return; }
            }
            None => return,
        };
        let model_path_str = match model_path.to_str().map(str::to_owned) {
            Some(s) => s,
            None => return,
        };

        // 30-second window, poll every 500ms and transcribe every 5s
        let window_samples = (30 * sample_rate) as usize;
        let min_new_samples = (4 * sample_rate) as usize; // require 4s of new audio before next chunk
        let mut last_processed_len: usize = 0;
        let mut elapsed_polls: u32 = 0;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            if !IS_RECORDING.load(Ordering::SeqCst) {
                break;
            }

            elapsed_polls += 1;
            if elapsed_polls < 10 {
                // Wait 5 seconds (10 × 500ms) between chunks
                continue;
            }

            let snapshot: Vec<f32> = {
                let buf = samples_arc.lock().unwrap();
                if buf.len() < last_processed_len + min_new_samples {
                    continue;
                }
                let start = buf.len().saturating_sub(window_samples);
                buf[start..].to_vec()
            };

            elapsed_polls = 0;
            last_processed_len = {
                let buf = samples_arc.lock().unwrap();
                buf.len()
            };

            // Use last partial as context for this chunk
            let initial_prompt = PARTIAL_TRANSCRIPTS
                .lock()
                .unwrap()
                .last()
                .cloned()
                .unwrap_or_default();

            let model_path_clone = model_path_str.clone();
            let lang = language.clone();
            let words = custom_words.clone();

            let result = tokio::task::spawn_blocking(move || {
                crate::transcription::transcribe(
                    &model_path_clone,
                    &snapshot,
                    sample_rate,
                    &lang,
                    &initial_prompt,
                    threshold,
                )
            })
            .await;

            if let Ok(Ok(text)) = result {
                if !text.is_empty() {
                    PARTIAL_TRANSCRIPTS.lock().unwrap().push(text.clone());
                    app_for_chunk.emit("partial-transcript", &text).ok();
                }
            }
        }
    });

    *CHUNK_HANDLE.lock().unwrap() = Some(handle);
    *CAPTURE.lock().unwrap() = Some(capture);
    IS_RECORDING.store(true, Ordering::SeqCst);
    app.emit("recording-state", true).ok();
    Ok(())
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```

Expected: `Finished` with no errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands.rs
git commit -m "feat: spawn 5s chunk transcription task on recording start"
```

---

### Task 3: Use accumulated partials as initial_prompt in final transcription

**Files:**
- Modify: `src-tauri/src/commands.rs` — update `stop_and_transcribe_internal`

- [ ] **Step 1: Update `stop_and_transcribe_internal` to wait for chunk task and use partials**

Replace the beginning of `stop_and_transcribe_internal` — specifically the capture extraction and the `spawn_blocking` call — to first stop the chunk task and inject partials as `initial_prompt`.

Replace the entire `stop_and_transcribe_internal` function with:

```rust
pub async fn stop_and_transcribe_internal<R: Runtime>(app: AppHandle<R>) {
    let capture = CAPTURE.lock().unwrap().take();
    IS_RECORDING.store(false, Ordering::SeqCst);
    app.emit("recording-state", false).ok();

    // Stop chunk task and wait up to 2s for any in-progress chunk to finish
    if let Some(handle) = CHUNK_HANDLE.lock().unwrap().take() {
        let _ = tokio::time::timeout(tokio::time::Duration::from_secs(2), handle).await;
    }

    let Some(cap) = capture else {
        app.emit("transcribe-error", "No hay grabación activa").ok();
        return;
    };

    app.emit("transcribing", true).ok();
    let (samples, sample_rate) = cap.stop();

    let rms = crate::transcription::rms_f32(&samples);
    eprintln!("[voz-local] samples: {}, rate: {}, rms: {:.6}", samples.len(), sample_rate, rms);

    if samples.is_empty() {
        app.emit("transcribing", false).ok();
        app.emit("transcribe-error", "No se capturó audio").ok();
        return;
    }

    let duration_secs = samples.len() as f32 / sample_rate as f32;
    if duration_secs < 0.5 {
        app.emit("transcribing", false).ok();
        app.emit("transcribe-error", "Grabación muy corta — mantén presionado para hablar").ok();
        return;
    }

    if rms < 0.0001 {
        app.emit("transcribing", false).ok();
        app.emit("transcribe-error", "Audio silencioso — verifica permisos de micrófono").ok();
        return;
    }

    let settings = settings::load(&app);
    let language = settings.selected_language.clone();
    let word_correction_threshold = settings.word_correction_threshold;
    let model_name = if settings.selected_model.is_empty() {
        "large-v3-turbo"
    } else {
        settings.selected_model.as_str()
    };

    let Some(dir) = models_dir(&app) else {
        app.emit("transcribing", false).ok();
        app.emit("transcribe-error", "Modelo no encontrado. Descárgalo en Ajustes → Modelos.").ok();
        return;
    };
    let primary  = dir.join(format!("ggml-{}.bin", model_name));
    let fallback = dir.join("ggml-base.bin");
    let model_path = if primary.exists() {
        primary
    } else if fallback.exists() {
        fallback
    } else {
        app.emit("transcribing", false).ok();
        app.emit("transcribe-error", "Modelo no encontrado. Descárgalo en Ajustes → Modelos.").ok();
        return;
    };

    let Some(model_path_str) = model_path.to_str().map(str::to_owned) else {
        app.emit("transcribing", false).ok();
        app.emit("transcribe-error", "Ruta del modelo contiene caracteres inválidos").ok();
        return;
    };

    // Assembled partials become the initial_prompt — gives Whisper full context
    // to correct word boundaries, style, and chunk-edge errors in the final pass.
    let initial_prompt = PARTIAL_TRANSCRIPTS.lock().unwrap().join(" ");

    let app_clone = app.clone();
    let samples: Arc<[f32]> = Arc::from(samples);
    let samples_for_blocking = Arc::clone(&samples);

    let result = tokio::task::spawn_blocking(move || {
        crate::transcription::transcribe(
            &model_path_str,
            &samples_for_blocking,
            sample_rate,
            &language,
            &initial_prompt,
            word_correction_threshold,
        )
    })
    .await;

    app.emit("transcribing", false).ok();

    match result {
        Ok(Ok(text)) if !text.is_empty() => {
            history::save_entry(&app_clone, text.clone(), &samples, sample_rate);
            app.emit("transcription-done", &text).ok();
            let text_for_paste = text.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                paste_text(&text_for_paste);
            });
        }
        Ok(Ok(_)) => {
            app.emit("transcribe-error", "No se detectó voz").ok();
        }
        Ok(Err(e)) => {
            app.emit("transcribe-error", e.to_string()).ok();
        }
        Err(e) => {
            app.emit("transcribe-error", e.to_string()).ok();
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```

Expected: `Finished` with no errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands.rs
git commit -m "feat: use accumulated partials as Whisper initial_prompt on final pass"
```

---

### Task 4: Show partial transcript in widget

**Files:**
- Modify: `src/routes/widget/+page.svelte`

- [ ] **Step 1: Read current widget to understand existing state variables**

Read `src/routes/widget/+page.svelte` and identify:
- Where `transcription-done` is handled
- Where `transcribing` state is displayed
- The text display area

- [ ] **Step 2: Add partial transcript handling**

In the `<script>` section, add a `partialText` variable and listener alongside the existing ones:

```javascript
let partialText = '';

// Add inside onMount, alongside existing listeners:
const unlistenPartial = await listen('partial-transcript', (event) => {
    partialText = event.payload;
});

// Add to the unlisten calls in onDestroy:
unlistenPartial();
```

Reset `partialText` on `recording-state` false and on `transcription-done`:
```javascript
// In the recording-state listener, when state is false:
partialText = '';

// In the transcription-done listener:
partialText = '';
```

- [ ] **Step 3: Display partial text in the widget UI**

In the template, add the partial text display in the transcribing state section. Show it when `transcribing` OR when `partialText` is non-empty during recording:

```svelte
{#if partialText && isRecording}
  <p class="partial-text">{partialText}</p>
{/if}
```

Add minimal CSS:
```css
.partial-text {
  font-size: 13px;
  color: rgba(255,255,255,0.7);
  margin: 4px 8px;
  line-height: 1.4;
  max-height: 80px;
  overflow: hidden;
}
```

- [ ] **Step 4: Build and verify with `npm run tauri dev`**

```bash
npm run tauri dev
```

Test: hold Alt+Space, speak for 10+ seconds, verify partial text appears in widget after ~5s.

- [ ] **Step 5: Commit**

```bash
git add src/routes/widget/+page.svelte
git commit -m "feat: display partial transcript in widget during recording"
```
