//! Recording state machine: statics, start/stop/cancel internals, streaming loop,
//! file transcription. This module owns the audio capture lifecycle and all
//! incremental (streaming) transcription state.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tauri::{AppHandle, Emitter, Runtime};

use crate::backend::{TranscribeOpts, TranscriptionBackend};
use crate::settings;
use crate::audio::AudioCapture;
use crate::whisper_backend::WhisperBackend;

// ── Statics ───────────────────────────────────────────────────────────────────

pub(crate) static IS_RECORDING: AtomicBool = AtomicBool::new(false);
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

// ── Internal helpers ──────────────────────────────────────────────────────────

pub(crate) fn is_recording() -> bool {
    IS_RECORDING.load(Ordering::SeqCst)
}

// ── Recording state machine ───────────────────────────────────────────────────

pub(crate) fn start_recording_internal<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
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
            crate::compat::hardware_default_model().to_string()
        } else {
            settings.selected_model.clone()
        };
        // Apple SpeechAnalyzer is effectively instant (~0.15s, ANE) and has no
        // model to warm — incremental pre-commits would only add overhead. Skip
        // the streaming loop entirely; the full audio is transcribed once at stop.
        if model_name == crate::speech_backend::APPLE_MODEL_ID {
            return;
        }
        let Some(dir) = crate::models::models_dir(&app_for_stream) else { return; };
        let Some(model_path) = crate::models::resolve_model_path(&dir, &model_name) else { return; };
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
            match text {
                Ok(Ok(t)) => {
                    let t = t.trim().to_string();
                    if !t.is_empty() {
                        COMMITTED_TEXT.lock().unwrap().push(t);
                    }
                }
                Ok(Err(e)) => {
                    log::warn!("[voz-local] streaming segment failed: {e}");
                    app_for_stream.emit("transcribe-warning", "Part of the dictation could not be processed").ok();
                }
                Err(e) => {
                    log::warn!("[voz-local] streaming segment JoinError: {e}");
                    app_for_stream.emit("transcribe-warning", "Part of the dictation could not be processed").ok();
                }
            }
        }
    });

    *STREAM_HANDLE.lock().unwrap() = Some(handle);

    Ok(())
}

pub(crate) async fn stop_and_transcribe_internal<R: Runtime>(app: AppHandle<R>) {
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
        app.emit("transcribe-error", "No active recording").ok();
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
    log::debug!("[voz-local] samples: {}, rate: {}, rms: {:.6}", samples.len(), sample_rate, rms);

    if samples.is_empty() {
        app.emit("transcribing", false).ok();
        app.emit("transcribe-error", "No audio captured").ok();
        return;
    }

    // Reject clips shorter than 500ms (common in accidental push-to-talk taps)
    let duration_secs = samples.len() as f32 / sample_rate as f32;
    if duration_secs < 0.5 {
        app.emit("transcribing", false).ok();
        app.emit("transcribe-error", "Recording too short — hold to talk").ok();
        return;
    }

    if rms < 0.0001 {
        app.emit("transcribing", false).ok();
        app.emit("transcribe-error", "Silent audio — check microphone permissions").ok();
        return;
    }

    let settings = settings::load(&app);
    let language = settings.selected_language.clone();
    let custom_words = settings.custom_words.clone();
    let word_correction_threshold = settings.word_correction_threshold;
    let model_name = if settings.selected_model.is_empty() {
        crate::compat::hardware_default_model().to_string()
    } else {
        settings.selected_model.clone()
    };
    let is_apple = model_name == crate::speech_backend::APPLE_MODEL_ID;

    // Whisper needs a downloaded .bin; Apple SpeechAnalyzer transcribes via the
    // sidecar with no local model file, so skip the model-path resolution.
    let model_path_str = if is_apple {
        String::new()
    } else {
        let Some(dir) = crate::models::models_dir(&app) else {
            app.emit("transcribing", false).ok();
            app.emit("transcribe-error", "Model not found. Download it in Settings → Models.").ok();
            return;
        };
        let Some(model_path) = crate::models::resolve_model_path(&dir, &model_name) else {
            app.emit("transcribing", false).ok();
            app.emit("transcribe-error", "Model not found. Download it in Settings → Models.").ok();
            return;
        };
        let Some(s) = model_path.to_str().map(str::to_owned) else {
            app.emit("transcribing", false).ok();
            app.emit("transcribe-error", "Model path contains invalid characters").ok();
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

    log::info!(
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
            log::warn!("[voz-local] tail transcription failed: {e}");
            if committed_text.is_empty() {
                app.emit("transcribe-error", e.to_string()).ok();
                return;
            }
            app.emit("transcribe-warning", "Dictation ending had a partial error").ok();
            String::new()
        }
        Err(e) => {
            log::warn!("[voz-local] tail transcription JoinError: {e}");
            if committed_text.is_empty() {
                app.emit("transcribe-error", e.to_string()).ok();
                return;
            }
            app.emit("transcribe-warning", "Dictation ending had a partial error").ok();
            String::new()
        }
    };

    // Stitch committed segments + tail, normalize whitespace, correct vocabulary once.
    let mut parts = committed_text;
    if !tail_text.is_empty() {
        parts.push(tail_text);
    }
    let assembled = parts.join(" ").split_whitespace().collect::<Vec<_>>().join(" ");
    let corrected = crate::transcription::correct_words(&assembled, &custom_words, word_correction_threshold);
    // Deterministic find/replace + snippets (works for both engines; the main
    // way custom vocabulary reaches the Apple engine, which has no prompt).
    let text = crate::replacements::apply_replacements(&corrected, &settings.replacements);

    if text.is_empty() {
        app.emit("transcribe-error", "No speech detected").ok();
        return;
    }

    crate::history::save_entry(&app_clone, text.clone(), &samples, sample_rate, "dictation".to_string(), None);
    // Notify the widget FIRST so it starts its close countdown.
    app.emit("transcription-done", &text).ok();
    let engine = if is_apple { "apple" } else { "whisper" };
    let duration_ms = (duration_secs as f64 * 1000.0) as i64;
    let props = crate::analytics::transcription_props(engine, &model_name, &language, "dictation", &text, duration_ms);
    app.emit("analytics-event", serde_json::json!({ "event": "transcription_completed", "props": props })).ok();
    // 300ms: time for the previously-active app to regain keyboard focus before Cmd+V.
    let text_for_paste = text.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        crate::paste::paste_text(&text_for_paste);
    });
}

/// Cancel an in-progress recording: discard the captured audio, stop the
/// streaming loop, and tell the widget to close — no transcription, no paste.
pub(crate) async fn cancel_recording_internal<R: Runtime>(app: AppHandle<R>) {
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
            let msg = "Could not transcribe the file. Supported formats: WAV, MP3, M4A.";
            app.emit("file-transcribe-error", msg).ok();
            return Err(msg.to_string());
        }
    };

    // 2. Resample to the 16kHz mono f32 Whisper expects.
    let samples = crate::transcription::resample(&samples_native, rate as usize, 16000);

    // 3. Load settings + resolve the model path (mirrors stop_and_transcribe_internal).
    let settings = crate::settings::load(&app);
    let language = settings.selected_language.clone();
    let custom_words = settings.custom_words.clone();
    let word_correction_threshold = settings.word_correction_threshold;
    let model_name = if settings.selected_model.is_empty() {
        crate::compat::hardware_default_model().to_string()
    } else {
        settings.selected_model.clone()
    };
    let is_apple = model_name == crate::speech_backend::APPLE_MODEL_ID;

    let model_path_str = if is_apple {
        String::new()
    } else {
        let Some(dir) = crate::models::models_dir(&app) else {
            let msg = "Model not found. Download it in Settings → Models.";
            app.emit("file-transcribe-error", msg).ok();
            return Err(msg.to_string());
        };
        let Some(model_path) = crate::models::resolve_model_path(&dir, &model_name) else {
            let msg = "Model not found. Download it in Settings → Models.";
            app.emit("file-transcribe-error", msg).ok();
            return Err(msg.to_string());
        };
        let Some(s) = model_path.to_str().map(str::to_owned) else {
            let msg = "Model path contains invalid characters";
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

    // 5. Correct vocabulary, then apply deterministic find/replace + snippets.
    let corrected = crate::transcription::correct_words(raw.trim(), &custom_words, word_correction_threshold);
    let text = crate::replacements::apply_replacements(&corrected, &settings.replacements);
    if text.is_empty() {
        let msg = "No speech detected";
        app.emit("file-transcribe-error", msg).ok();
        return Err(msg.to_string());
    }

    // 6. Save as a unified entry tagged source="file" (samples already 16k mono).
    crate::history::save_entry(
        &app,
        text.clone(),
        &samples,
        16000,
        "file".to_string(),
        original_filename,
    );

    // 7. Done — no paste (file transcription is not a dictation).
    app.emit("file-transcribe-done", &text).ok();
    let engine = if is_apple { "apple" } else { "whisper" };
    let duration_ms = (samples.len() as f64 / 16000.0 * 1000.0) as i64;
    let props = crate::analytics::transcription_props(engine, &model_name, &language, "file", &text, duration_ms);
    app.emit("analytics-event", serde_json::json!({ "event": "transcription_completed", "props": props })).ok();
    Ok(text)
}
