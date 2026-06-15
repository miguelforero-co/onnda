use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::backend::{TranscribeOpts, TranscriptionBackend};
use crate::history::{self, HistoryEntry};
use crate::settings::{self, AppSettings};
use crate::audio::AudioCapture;
use crate::whisper_backend::WhisperBackend;

static IS_RECORDING: AtomicBool = AtomicBool::new(false);
static CAPTURE: Mutex<Option<AudioCapture>> = Mutex::new(None);

// ── Streaming (incremental) transcription state ─────────────────────────────
// While recording, completed speech segments (audio up to a real pause) are
// transcribed and their text committed here, in order. COMMITTED_SAMPLES tracks
// how many native-rate samples have already been committed, so at stop only the
// short un-committed tail needs transcribing. STREAM_HANDLE is the background
// segment loop, awaited briefly at stop so the last in-flight segment lands.
static COMMITTED_TEXT: Mutex<Vec<String>> = Mutex::new(Vec::new());
static COMMITTED_SAMPLES: AtomicUsize = AtomicUsize::new(0);
static STREAM_HANDLE: Mutex<Option<tauri::async_runtime::JoinHandle<()>>> = Mutex::new(None);

#[cfg(target_os = "macos")]
fn ax_is_trusted() -> bool {
    extern "C" { fn AXIsProcessTrusted() -> bool; }
    unsafe { AXIsProcessTrusted() }
}

// ── Permission checks ──────────────────────────────────────────────────────

#[tauri::command]
pub fn check_mic_permission() -> bool {
    crate::mic_permission::is_granted()
}

#[tauri::command]
pub fn check_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    { ax_is_trusted() }
    #[cfg(not(target_os = "macos"))]
    { true }
}

#[tauri::command]
pub fn open_accessibility_settings() {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn();
    }
}

#[tauri::command]
pub fn open_microphone_settings() {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
            .spawn();
    }
}

// ── Settings ───────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_settings<R: Runtime>(app: AppHandle<R>) -> AppSettings {
    settings::load(&app)
}

#[tauri::command]
pub fn save_settings<R: Runtime>(
    app: AppHandle<R>,
    new_settings: AppSettings,
    shortcut_changed: bool,
) -> Result<(), String> {
    settings::save(&app, &new_settings).map_err(|e| e.to_string())?;

    if shortcut_changed {
        crate::shortcut::re_register(&app, &new_settings.shortcut)
            .map_err(|e| e.to_string())?;
    }

    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        if new_settings.autostart {
            app.autolaunch().enable().ok();
        } else {
            app.autolaunch().disable().ok();
        }
    }

    Ok(())
}

// ── Recording ─────────────────────────────────────────────────────────────

pub fn start_recording_internal<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if IS_RECORDING.load(Ordering::SeqCst) {
        return Ok(());
    }
    let app_clone = app.clone();
    let capture = AudioCapture::start(move |rms, bands| {
        app_clone.emit("audio-level", rms).ok();
        app_clone.emit("audio-bands", bands).ok();
    })
    .map_err(|e| e.to_string())?;

    // Reset streaming state for this recording.
    COMMITTED_TEXT.lock().unwrap().clear();
    COMMITTED_SAMPLES.store(0, Ordering::SeqCst);

    let samples_arc = capture.samples_arc();
    let sample_rate = capture.sample_rate;

    *CAPTURE.lock().unwrap() = Some(capture);
    IS_RECORDING.store(true, Ordering::SeqCst);
    app.emit("recording-state", true).ok();

    // Native feedback at the start of a recording, each gated on its opt-in flag.
    // Fires regardless of window visibility (D-07). pause-media toggles play/pause.
    let s = crate::settings::load(app);
    if s.sounds_enabled {
        crate::sounds::play_listen();
    }
    if s.pause_media {
        // Delay the mute briefly so the "listening" start cue is audible before
        // output is silenced; guard with is_recording() so a very short recording
        // (released before the delay fires) never leaves the system stuck muted.
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(140));
            if is_recording() {
                crate::media_pause::mute_outputs();
            }
        });
    }

    // Background streaming loop: warm the model, then commit completed speech
    // segments (audio up to a real pause) while the user is still talking, so at
    // stop only the short tail remains. Model-agnostic — uses whatever model the
    // user selected. Degrades to a single final pass if speech never pauses.
    let app_for_stream = app.clone();
    let handle = tauri::async_runtime::spawn(async move {
        let settings = settings::load(&app_for_stream);
        let language = settings.selected_language.clone();
        let custom_words = settings.custom_words.clone();
        let model_name = if settings.selected_model.is_empty() {
            "large-v3-turbo".to_string()
        } else {
            settings.selected_model.clone()
        };
        // Apple SpeechAnalyzer is effectively instant (~0.15s, ANE) and has no
        // model to warm — incremental pre-commits would only add overhead. Skip
        // the streaming loop entirely; the full audio is transcribed once at stop.
        if model_name == crate::speech_backend::APPLE_MODEL_ID {
            return;
        }
        let Some(dir) = models_dir(&app_for_stream) else { return; };
        let primary = dir.join(format!("ggml-{}.bin", model_name));
        let fallback = dir.join("ggml-base.bin");
        let model_path = if primary.exists() {
            primary
        } else if fallback.exists() {
            fallback
        } else {
            return;
        };
        let Some(model_path_str) = model_path.to_str().map(str::to_owned) else { return; };

        // Warm the model so the first segment (and the final tail) is pure inference.
        {
            let mp = model_path_str.clone();
            let _ = tokio::task::spawn_blocking(move || {
                let _ = WhisperBackend::new(mp).ensure_loaded();
            })
            .await;
        }

        // Whisper processes audio in fixed ~30s blocks (it pads short audio to
        // 30s), so every inference costs roughly the same fixed time regardless
        // of length. Committing on every small pause means many inferences, each
        // paying that toll — slower than a single final pass. So we only
        // pre-commit once the un-committed audio approaches a full block: typical
        // dictation (< ~25s) does ONE final pass at stop (optimal), and only long
        // dictation is chunked here, always cutting at a real pause so no word is
        // split. This keeps the number of inferences minimal — never worse than
        // the single-pass baseline, and faster for long dictation.
        let min_pending_samples = sample_rate as usize * 25;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            if !IS_RECORDING.load(Ordering::SeqCst) {
                break;
            }

            let committed = COMMITTED_SAMPLES.load(Ordering::SeqCst);
            let pending: Vec<f32> = {
                let buf = samples_arc.lock().unwrap();
                // Only act once there's ~a full block of un-committed audio.
                if buf.len() <= committed + min_pending_samples {
                    continue;
                }
                buf[committed..].to_vec()
            };

            let Some(cut) = crate::streaming::find_commit_point(&pending, sample_rate) else {
                continue;
            };
            // Don't bother committing a trivially short head.
            if cut < sample_rate as usize {
                continue;
            }

            let segment = pending[..cut].to_vec();
            let mp = model_path_str.clone();
            let lang = language.clone();
            // Custom vocabulary as prompt helps recognition; word correction is
            // applied once over the whole text at stop, not per segment.
            let prompt = custom_words.clone();
            let text = tokio::task::spawn_blocking(move || {
                WhisperBackend::new(mp).transcribe(
                    &segment,
                    &TranscribeOpts {
                        language: lang,
                        initial_prompt: prompt,
                        word_correction_threshold: 1.0,
                        sample_rate,
                    },
                )
            })
            .await;

            // Advance the committed marker regardless, so we never reprocess this
            // audio even if the segment came back empty or errored.
            COMMITTED_SAMPLES.fetch_add(cut, Ordering::SeqCst);
            if let Ok(Ok(t)) = text {
                let t = t.trim().to_string();
                if !t.is_empty() {
                    COMMITTED_TEXT.lock().unwrap().push(t);
                }
            }
        }
    });

    *STREAM_HANDLE.lock().unwrap() = Some(handle);

    Ok(())
}

pub async fn stop_and_transcribe_internal<R: Runtime>(app: AppHandle<R>) {
    let capture = CAPTURE.lock().unwrap().take();
    IS_RECORDING.store(false, Ordering::SeqCst);
    app.emit("recording-state", false).ok();

    // Native feedback on stop. Unmute FIRST (restore_outputs only unmutes if WE
    // muted on start, symmetric — see media_pause.rs), THEN play the cue so it's
    // audible above any restored output.
    let s = crate::settings::load(&app);
    if s.pause_media {
        crate::media_pause::restore_outputs();
    }
    if s.sounds_enabled {
        crate::sounds::play_stop();
    }

    // Stop the mic IMMEDIATELY so we stop listening the instant the user releases
    // (or taps stop) — do NOT keep the input stream open during transcription.
    // cap.stop() returns a snapshot of the captured samples; the streaming loop
    // holds its own Arc to the (now-frozen) buffer, so it can still finish below.
    let Some(cap) = capture else {
        app.emit("transcribe-error", "No hay grabación activa").ok();
        return;
    };
    let (samples, sample_rate) = cap.stop();
    app.emit("transcribing", true).ok();

    // Now let the streaming loop finish its in-flight segment (short) so its
    // committed text is ready before we assemble the tail. The buffer is frozen
    // (mic stopped above); the loop exits on the next IS_RECORDING check.
    let stream_handle = STREAM_HANDLE.lock().unwrap().take();
    if let Some(handle) = stream_handle {
        let _ = tokio::time::timeout(tokio::time::Duration::from_secs(8), handle).await;
    }

    let rms = crate::transcription::rms_f32(&samples);
    eprintln!("[voz-local] samples: {}, rate: {}, rms: {:.6}", samples.len(), sample_rate, rms);

    if samples.is_empty() {
        app.emit("transcribing", false).ok();
        app.emit("transcribe-error", "No se capturó audio").ok();
        return;
    }

    // Reject clips shorter than 500ms (common in accidental push-to-talk taps)
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
    let custom_words = settings.custom_words.clone();
    let word_correction_threshold = settings.word_correction_threshold;
    let model_name = if settings.selected_model.is_empty() {
        "large-v3-turbo".to_string()
    } else {
        settings.selected_model.clone()
    };
    let is_apple = model_name == crate::speech_backend::APPLE_MODEL_ID;

    // Whisper needs a downloaded .bin; Apple SpeechAnalyzer transcribes via the
    // sidecar with no local model file, so skip the model-path resolution.
    let model_path_str = if is_apple {
        String::new()
    } else {
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
        let Some(s) = model_path.to_str().map(str::to_owned) else {
            app.emit("transcribing", false).ok();
            app.emit("transcribe-error", "Ruta del modelo contiene caracteres inválidos").ok();
            return;
        };
        s
    };

    let app_clone = app.clone();
    let samples: Arc<[f32]> = Arc::from(samples);

    // Segments transcribed while the user spoke; the tail is everything since the
    // last committed pause. Process only the tail, then stitch + correct once.
    let committed_text = COMMITTED_TEXT.lock().unwrap().clone();
    let committed = COMMITTED_SAMPLES.load(Ordering::SeqCst).min(samples.len());
    let tail: Vec<f32> = samples[committed..].to_vec();

    eprintln!(
        "[voz-local] streaming: {} committed segment(s), tail = {:.1}s of {:.1}s total",
        committed_text.len(),
        tail.len() as f32 / sample_rate as f32,
        samples.len() as f32 / sample_rate as f32,
    );

    let lang_tail = language.clone();
    let prompt_tail = custom_words.clone();
    let min_tail = (sample_rate as usize) / 10; // 100ms — skip a negligible tail
    let tail_result = if tail.len() > min_tail {
        if is_apple {
            // Apple SpeechAnalyzer via the async sidecar (no spawn_blocking).
            match crate::speech_backend::apple_transcribe(&app, &tail, sample_rate, &lang_tail).await {
                Ok(t) => Ok(Ok(t)),
                Err(e) => Ok(Err(e)),
            }
        } else {
            tokio::task::spawn_blocking(move || {
                WhisperBackend::new(model_path_str).transcribe(
                    &tail,
                    &TranscribeOpts {
                        language: lang_tail,
                        initial_prompt: prompt_tail,
                        word_correction_threshold: 1.0,
                        sample_rate,
                    },
                )
            })
            .await
        }
    } else {
        Ok(Ok(String::new()))
    };

    app.emit("transcribing", false).ok();

    // If the tail fails but we already committed segments, fall back to those.
    let tail_text = match tail_result {
        Ok(Ok(t)) => t.trim().to_string(),
        Ok(Err(e)) => {
            if committed_text.is_empty() {
                app.emit("transcribe-error", e.to_string()).ok();
                return;
            }
            String::new()
        }
        Err(e) => {
            if committed_text.is_empty() {
                app.emit("transcribe-error", e.to_string()).ok();
                return;
            }
            String::new()
        }
    };

    // Stitch committed segments + tail, normalize whitespace, correct vocabulary once.
    let mut parts = committed_text;
    if !tail_text.is_empty() {
        parts.push(tail_text);
    }
    let assembled = parts.join(" ").split_whitespace().collect::<Vec<_>>().join(" ");
    let text = crate::transcription::correct_words(&assembled, &custom_words, word_correction_threshold);

    if text.is_empty() {
        app.emit("transcribe-error", "No se detectó voz").ok();
        return;
    }

    history::save_entry(&app_clone, text.clone(), &samples, sample_rate, "dictation".to_string(), None);
    // Notify the widget FIRST so it starts its close countdown.
    app.emit("transcription-done", &text).ok();
    // 300ms: time for the previously-active app to regain keyboard focus before Cmd+V.
    let text_for_paste = text.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        paste_text(&text_for_paste);
    });
}

/// Cancel an in-progress recording: discard the captured audio, stop the
/// streaming loop, and tell the widget to close — no transcription, no paste.
pub async fn cancel_recording_internal<R: Runtime>(app: AppHandle<R>) {
    if !IS_RECORDING.load(Ordering::SeqCst) {
        return;
    }
    let capture = CAPTURE.lock().unwrap().take();
    IS_RECORDING.store(false, Ordering::SeqCst);
    app.emit("recording-state", false).ok();

    // Native feedback on cancel. Unmute FIRST (restore_outputs only unmutes if WE
    // muted on start), THEN play the cue so it's audible above restored output.
    let s = crate::settings::load(&app);
    if s.pause_media {
        crate::media_pause::restore_outputs();
    }
    if s.sounds_enabled {
        crate::sounds::play_cancel();
    }

    // Abort the streaming loop (don't wait — we're throwing the audio away).
    if let Some(handle) = STREAM_HANDLE.lock().unwrap().take() {
        handle.abort();
    }
    // Drop the captured audio.
    if let Some(cap) = capture {
        let _ = cap.stop();
    }
    COMMITTED_TEXT.lock().unwrap().clear();
    COMMITTED_SAMPLES.store(0, Ordering::SeqCst);

    // Tell the widget to fade out & collapse without pasting anything.
    app.emit("recording-cancelled", ()).ok();
}

/// Returns "ok" if paste should work, or an error description if not
#[tauri::command]
pub fn test_paste() -> String {
    #[cfg(target_os = "macos")]
    {
        if !ax_is_trusted() {
            return "no_accessibility".to_string();
        }
        // Try posting a harmless zero-length CGEvent to verify the API works
        unsafe {
            use std::ffi::c_void;
            #[link(name = "CoreGraphics", kind = "framework")]
            extern "C" {
                fn CGEventSourceCreate(state_id: i32) -> *mut c_void;
                fn CFRelease(cf: *mut c_void);
            }
            let src = CGEventSourceCreate(1);
            if src.is_null() { return "cg_source_null".to_string(); }
            CFRelease(src);
        }
        return "ok".to_string();
    }
    #[cfg(not(target_os = "macos"))]
    "ok".to_string()
}

pub fn is_recording() -> bool {
    IS_RECORDING.load(Ordering::SeqCst)
}

/// Short git commit hash (with `-dirty` suffix on uncommitted builds), embedded
/// at compile time. Lets the UI distinguish builds beyond the SemVer number.
#[tauri::command]
pub fn get_build_hash() -> String {
    env!("GIT_HASH").to_string()
}

/// Authoritative app version straight from Cargo at compile time. The frontend
/// `getVersion()` (Tauri JS) can mismatch the Rust crate version, so the UI
/// should read this instead.
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ── Tauri commands (still expose for any direct frontend use) ─────────────

#[tauri::command]
pub fn start_recording<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    start_recording_internal(&app)
}

#[tauri::command]
pub async fn stop_and_transcribe<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    stop_and_transcribe_internal(app).await;
    Ok(())
}

/// Transcribe an arbitrary audio file (wav/mp3/m4a) picked by the user.
///
/// Pipeline (all heavy work off the async runtime via spawn_blocking):
/// decode → resample to 16kHz → Whisper → vocabulary correction → save a
/// unified history entry with `source = "file"`. Emits `file-transcribe-*`
/// events the frontend (01-08) listens for. Unlike dictation, this never pastes.
#[tauri::command]
pub async fn transcribe_file<R: Runtime>(app: AppHandle<R>, path: String) -> Result<String, String> {
    let original_filename = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .map(str::to_owned);

    // 1. Decode the untrusted file off the async runtime.
    app.emit("file-transcribe-progress", "decoding").ok();
    let decode_path = path.clone();
    let (samples_native, rate) = match tokio::task::spawn_blocking(move || {
        crate::audio_decode::decode_file(&decode_path)
    })
    .await
    {
        Ok(Ok(decoded)) => decoded,
        Ok(Err(_)) | Err(_) => {
            let msg = "No se pudo transcribir el archivo. Formatos admitidos: WAV, MP3, M4A.";
            app.emit("file-transcribe-error", msg).ok();
            return Err(msg.to_string());
        }
    };

    // 2. Resample to the 16kHz mono f32 Whisper expects.
    let samples = crate::transcription::resample(&samples_native, rate as usize, 16000);

    // 3. Load settings + resolve the model path (mirrors stop_and_transcribe_internal).
    let settings = settings::load(&app);
    let language = settings.selected_language.clone();
    let custom_words = settings.custom_words.clone();
    let word_correction_threshold = settings.word_correction_threshold;
    let model_name = if settings.selected_model.is_empty() {
        "large-v3-turbo".to_string()
    } else {
        settings.selected_model.clone()
    };
    let is_apple = model_name == crate::speech_backend::APPLE_MODEL_ID;

    let model_path_str = if is_apple {
        String::new()
    } else {
        let Some(dir) = models_dir(&app) else {
            let msg = "Modelo no encontrado. Descárgalo en Ajustes → Modelos.";
            app.emit("file-transcribe-error", msg).ok();
            return Err(msg.to_string());
        };
        let primary = dir.join(format!("ggml-{}.bin", model_name));
        let fallback = dir.join("ggml-base.bin");
        let model_path = if primary.exists() {
            primary
        } else if fallback.exists() {
            fallback
        } else {
            let msg = "Modelo no encontrado. Descárgalo en Ajustes → Modelos.";
            app.emit("file-transcribe-error", msg).ok();
            return Err(msg.to_string());
        };
        let Some(s) = model_path.to_str().map(str::to_owned) else {
            let msg = "Ruta del modelo contiene caracteres inválidos";
            app.emit("file-transcribe-error", msg).ok();
            return Err(msg.to_string());
        };
        s
    };

    // 4. Transcribe off the async runtime.
    app.emit("file-transcribe-progress", "transcribing").ok();
    let lang = language.clone();
    let prompt = custom_words.clone();
    let transcribe_samples = samples.clone();
    let raw = if is_apple {
        match crate::speech_backend::apple_transcribe(&app, &transcribe_samples, 16000, &lang).await {
            Ok(t) => t,
            Err(e) => {
                app.emit("file-transcribe-error", e.to_string()).ok();
                return Err(e.to_string());
            }
        }
    } else {
        match tokio::task::spawn_blocking(move || {
            WhisperBackend::new(model_path_str).transcribe(
                &transcribe_samples,
                &TranscribeOpts {
                    language: lang,
                    initial_prompt: prompt,
                    word_correction_threshold: 1.0,
                    sample_rate: 16000,
                },
            )
        })
        .await
        {
            Ok(Ok(t)) => t,
            Ok(Err(e)) => {
                app.emit("file-transcribe-error", e.to_string()).ok();
                return Err(e.to_string());
            }
            Err(e) => {
                app.emit("file-transcribe-error", e.to_string()).ok();
                return Err(e.to_string());
            }
        }
    };

    // 5. Correct vocabulary once over the assembled text.
    let text = crate::transcription::correct_words(raw.trim(), &custom_words, word_correction_threshold);
    if text.is_empty() {
        let msg = "No se detectó voz";
        app.emit("file-transcribe-error", msg).ok();
        return Err(msg.to_string());
    }

    // 6. Save as a unified entry tagged source="file" (samples already 16k mono).
    history::save_entry(
        &app,
        text.clone(),
        &samples,
        16000,
        "file".to_string(),
        original_filename,
    );

    // 7. Done — no paste (file transcription is not a dictation).
    app.emit("file-transcribe-done", &text).ok();
    Ok(text)
}

#[tauri::command]
pub fn is_recording_cmd() -> bool {
    is_recording()
}

// ── Widget ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn hide_widget<R: Runtime>(app: AppHandle<R>) {
    if let Some(widget) = app.get_webview_window("widget") {
        widget.hide().ok();
    }
}

// ── History ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_history<R: Runtime>(app: AppHandle<R>) -> Vec<HistoryEntry> {
    history::load(&app)
}

#[tauri::command]
pub fn delete_history_entry<R: Runtime>(app: AppHandle<R>, id: String) {
    history::delete(&app, &id);
}

#[tauri::command]
pub fn get_recording_audio<R: Runtime>(app: AppHandle<R>, filename: String) -> Option<String> {
    history::get_audio_base64(&app, &filename)
}

// ── Models ─────────────────────────────────────────────────────────────────

fn models_dir<R: Runtime>(app: &AppHandle<R>) -> Option<std::path::PathBuf> {
    app.path().app_data_dir().ok().map(|p| p.join("models"))
}

#[tauri::command]
pub fn get_models<R: Runtime>(app: AppHandle<R>) -> Vec<ModelInfo> {
    let dir = models_dir(&app);
    let model_exists = |name: &str| {
        dir.as_ref()
            .map(|p| p.join(format!("ggml-{}.bin", name)).exists())
            .unwrap_or(false)
    };

    vec![
        ModelInfo {
            id: "base".to_string(),
            name: "Whisper Base".to_string(),
            size_mb: 141,
            downloaded: model_exists("base"),
            coming_soon: false,
        },
        ModelInfo {
            id: "small".to_string(),
            name: "Whisper Small".to_string(),
            size_mb: 466,
            downloaded: model_exists("small"),
            coming_soon: false,
        },
        ModelInfo {
            id: "medium".to_string(),
            name: "Whisper Medium".to_string(),
            size_mb: 1536,
            downloaded: model_exists("medium"),
            coming_soon: false,
        },
        ModelInfo {
            id: "large-v3-turbo".to_string(),
            name: "Whisper Large v3 Turbo".to_string(),
            size_mb: 874,
            downloaded: model_exists("large-v3-turbo"),
            coming_soon: false,
        },
        ModelInfo {
            id: crate::speech_backend::APPLE_MODEL_ID.to_string(),
            name: "Apple (Neural Engine)".to_string(),
            size_mb: 0,           // on-device, assets managed by macOS
            downloaded: true,     // no .bin to download; always available on macOS 26+
            coming_soon: false,
        },
    ]
}

#[tauri::command]
pub async fn download_model<R: Runtime>(app: AppHandle<R>, model_id: String) -> Result<(), String> {
    let url = match model_id.as_str() {
        "large-v3-turbo" => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q8_0.bin",
        "base"           => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
        "small"          => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
        "medium"         => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
        other            => return Err(format!("Modelo desconocido: {}", other)),
    };

    let dir = models_dir(&app).ok_or("No se pudo obtener el directorio de datos")?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let dest = dir.join(format!("ggml-{}.bin", model_id));
    let tmp  = dir.join(format!("ggml-{}.bin.tmp", model_id));

    let client = reqwest::Client::builder()
        .user_agent("voz-local/0.1")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let total = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = tokio::fs::File::create(&tmp).await.map_err(|e| e.to_string())?;
    let mut stream = resp.bytes_stream();

    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
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

    drop(file);
    tokio::fs::rename(&tmp, &dest).await.map_err(|e| e.to_string())?;
    app.emit("download-complete", &model_id).ok();
    Ok(())
}

// ── Internals ──────────────────────────────────────────────────────────────

fn paste_text(text: &str) {
    #[cfg(target_os = "macos")]
    {
        // 1. Write to clipboard using NSPasteboard directly — avoids pbcopy's locale
        //    encoding issues (pbcopy can mangle UTF-8 when LANG is not set in the env).
        unsafe { write_clipboard_utf8(text) };

        // 2. Simulate Cmd+V via CoreGraphics CGEventPost.
        //    Requires Accessibility permission.
        unsafe { post_cmd_v() };
    }
}

#[cfg(target_os = "macos")]
unsafe fn write_clipboard_utf8(text: &str) {
    use objc2::{class, msg_send, runtime::AnyObject};
    use objc2_foundation::{NSString, ns_string};

    let pb: *mut AnyObject = msg_send![class!(NSPasteboard), generalPasteboard];
    // Clear existing clipboard content
    let _: i64 = msg_send![pb, clearContents];
    // Create NSString from Rust &str (always UTF-8 → Unicode)
    let ns_str = NSString::from_str(text);
    // Store as public.utf8-plain-text
    let pb_type = ns_string!("public.utf8-plain-text");
    let _: bool = msg_send![pb, setString: &*ns_str, forType: pb_type];
}

#[cfg(target_os = "macos")]
unsafe fn post_cmd_v() {
    use std::ffi::c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceCreate(state_id: i32) -> *mut c_void;
        fn CGEventCreateKeyboardEvent(source: *mut c_void, virtual_key: u16, key_down: bool) -> *mut c_void;
        fn CGEventSetFlags(event: *mut c_void, flags: u64);
        fn CGEventPost(tap: i32, event: *mut c_void);
        fn CFRelease(cf: *mut c_void);
    }

    const V_KEY: u16 = 9;           // kVK_ANSI_V
    const CMD_MASK: u64 = 0x100000; // kCGEventFlagMaskCommand
    const HID_TAP: i32 = 0;         // kCGHIDEventTap
    const HID_STATE: i32 = 1;       // kCGEventSourceStateHIDSystemState

    let src = CGEventSourceCreate(HID_STATE);
    if src.is_null() { return; }

    let dn = CGEventCreateKeyboardEvent(src, V_KEY, true);
    if !dn.is_null() { CGEventSetFlags(dn, CMD_MASK); CGEventPost(HID_TAP, dn); CFRelease(dn); }

    let up = CGEventCreateKeyboardEvent(src, V_KEY, false);
    if !up.is_null() { CGEventSetFlags(up, CMD_MASK); CGEventPost(HID_TAP, up); CFRelease(up); }

    CFRelease(src);
}

#[derive(serde::Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size_mb: u32,
    pub downloaded: bool,
    /// True for catalog entries that are listed but not yet functional (e.g.
    /// Parakeet ANE). The frontend renders these as a "Próximamente" card
    /// without a download action (D-13).
    pub coming_soon: bool,
}
