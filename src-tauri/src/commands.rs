use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::history::{self, HistoryEntry};
use crate::settings::{self, AppSettings};
use crate::audio::AudioCapture;

static IS_RECORDING: AtomicBool = AtomicBool::new(false);
static CAPTURE: Mutex<Option<AudioCapture>> = Mutex::new(None);
static CHUNK_HANDLE: Mutex<Option<tauri::async_runtime::JoinHandle<()>>> = Mutex::new(None);
static PARTIAL_TRANSCRIPTS: Mutex<Vec<String>> = Mutex::new(Vec::new());

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
    let capture = AudioCapture::start(move |rms| {
        app_clone.emit("audio-level", rms).ok();
    })
    .map_err(|e| e.to_string())?;

    PARTIAL_TRANSCRIPTS.lock().unwrap().clear();

    let samples_arc = capture.samples_arc();
    let sample_rate = capture.sample_rate;
    let app_for_chunk = app.clone();

    let handle = tauri::async_runtime::spawn(async move {
        let settings = crate::settings::load(&app_for_chunk);
        let language = settings.selected_language.clone();
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
                if primary.exists() { primary } else if fallback.exists() { fallback } else { return; }
            }
            None => return,
        };
        let model_path_str = match model_path.to_str().map(str::to_owned) {
            Some(s) => s,
            None => return,
        };

        let window_samples = (30 * sample_rate) as usize;
        let min_new_samples = (4 * sample_rate) as usize;
        let mut last_processed_len: usize = 0;
        let mut polls: u32 = 0;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            if !IS_RECORDING.load(Ordering::SeqCst) { break; }

            polls += 1;
            if polls < 10 { continue; } // fire every 5s (10 × 500ms)
            polls = 0;

            let snapshot: Vec<f32> = {
                let buf = samples_arc.lock().unwrap();
                if buf.len() < last_processed_len + min_new_samples { continue; }
                let start = buf.len().saturating_sub(window_samples);
                buf[start..].to_vec()
            };

            last_processed_len = samples_arc.lock().unwrap().len();

            let initial_prompt = PARTIAL_TRANSCRIPTS.lock().unwrap().last().cloned().unwrap_or_default();
            let model_path_clone = model_path_str.clone();
            let lang = language.clone();

            let result = tokio::task::spawn_blocking(move || {
                crate::transcription::transcribe(&model_path_clone, &snapshot, sample_rate, &lang, &initial_prompt, threshold)
            }).await;

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

pub async fn stop_and_transcribe_internal<R: Runtime>(app: AppHandle<R>) {
    let capture = CAPTURE.lock().unwrap().take();
    IS_RECORDING.store(false, Ordering::SeqCst);
    app.emit("recording-state", false).ok();

    // Drop the lock immediately so the guard doesn't cross the await boundary
    let chunk_handle = CHUNK_HANDLE.lock().unwrap().take();
    if let Some(handle) = chunk_handle {
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

    // Final pass runs clean with custom vocabulary — chunks were for visual feedback only
    let initial_prompt = custom_words.clone();

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
            // Notify the widget FIRST so it starts its close countdown.
            app.emit("transcription-done", &text).ok();
            // 300ms: time for the previously-active app to regain keyboard focus before Cmd+V.
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
            id: "large-v3-turbo".to_string(),
            name: "Whisper Large v3 Turbo".to_string(),
            size_mb: 874,
            downloaded: model_exists("large-v3-turbo"),
        },
        ModelInfo {
            id: "base".to_string(),
            name: "Whisper Base".to_string(),
            size_mb: 141,
            downloaded: model_exists("base"),
        },
    ]
}

#[tauri::command]
pub async fn download_model<R: Runtime>(app: AppHandle<R>, model_id: String) -> Result<(), String> {
    let url = match model_id.as_str() {
        "large-v3-turbo" => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q8_0.bin",
        "base"           => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
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
}
