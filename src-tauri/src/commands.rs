//! Thin #[tauri::command] wrappers + permissions + settings + history + misc.
//!
//! The heavy subsystems live in focused modules:
//!   - paste.rs     — clipboard write + CGEvent Cmd+V
//!   - models.rs    — catalogue, download, path resolution
//!   - recording.rs — state machine, streaming loop, transcribe_file
//!
//! This file only contains commands that don't belong to any of those modules,
//! plus thin wrappers that delegate to recording::* / models::*.

use tauri::{AppHandle, Emitter, Manager, Runtime};
use crate::history::{self, HistoryEntry};
use crate::settings::{self, AppSettings};

// ── Permission checks ──────────────────────────────────────────────────────

#[tauri::command]
pub fn check_mic_permission() -> bool {
    crate::mic_permission::is_granted()
}

#[tauri::command]
pub fn check_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    { crate::paste::ax_is_trusted() }
    #[cfg(not(target_os = "macos"))]
    { true }
}

/// Trigger the macOS Accessibility prompt so the app is registered in the list
/// (the toggle appears) and the system dialog is shown when not yet trusted.
/// Returns the current trust state.
#[tauri::command]
pub fn request_accessibility() -> bool {
    #[cfg(target_os = "macos")]
    { crate::paste::prompt_ax_trust() }
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
    let prev_model = settings::load(&app).selected_model.clone();
    settings::save(&app, &new_settings).map_err(|e| e.to_string())?;

    if new_settings.selected_model != prev_model {
        let engine_of = |m: &str| if m == crate::speech_backend::APPLE_MODEL_ID { "apple" } else { "whisper" };
        let new_engine = engine_of(&new_settings.selected_model);
        if engine_of(&prev_model) != new_engine {
            app.emit("analytics-event", serde_json::json!({ "event": "engine_changed", "props": { "engine": new_engine } })).ok();
        }
    }

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

// ── Recording wrappers ─────────────────────────────────────────────────────

#[tauri::command]
pub fn start_recording<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    crate::recording::start_recording_internal(&app)
}

#[tauri::command]
pub async fn stop_and_transcribe<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    crate::recording::stop_and_transcribe_internal(app).await;
    Ok(())
}

#[tauri::command]
pub fn is_recording_cmd() -> bool {
    crate::recording::is_recording()
}

/// Calienta el motor Apple SpeechAnalyzer: dispara en background el aprovisionamiento
/// del modelo Speech on-device de macOS (que en el primer uso descarga/inicializa y
/// puede tardar), para que el PRIMER dictado real no sufra el cold-start que dejaba el
/// notch colgado. Fire-and-forget: ignora el resultado. Llamar al seleccionar Apple.
#[tauri::command]
pub fn warm_apple_engine(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let silence = vec![0.0f32; 8000]; // 0.5s @ 16kHz, sólo para provocar la carga
        let _ = crate::speech_backend::apple_transcribe(&app, &silence, 16000, "auto").await;
    });
}

// ── Paste / build info ─────────────────────────────────────────────────────

/// Returns "ok" if paste should work, or an error description if not
#[tauri::command]
pub fn test_paste() -> String {
    #[cfg(target_os = "macos")]
    {
        if !crate::paste::ax_is_trusted() {
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

/// Save a user correction to a transcription. Updates the stored text and, when
/// auto-learn is on, diffs the ORIGINAL ASR output against the correction and
/// learns recurring word substitutions as replacement rules (Phase 3).
/// Returns what was learned/promoted so the UI can give feedback.
#[tauri::command]
pub fn correct_history_entry<R: Runtime>(
    app: AppHandle<R>,
    id: String,
    new_text: String,
) -> crate::learn::LearnOutcome {
    let Some((prev_text, entry)) = history::update_text(&app, &id, &new_text) else {
        return crate::learn::LearnOutcome::default();
    };

    let mut settings = settings::load(&app);
    if !settings.auto_learn {
        return crate::learn::LearnOutcome::default();
    }
    // Diff against the original ASR text (captured on first edit), not a prior
    // manual edit, so we learn from the true machine output.
    let base = entry.original_text.clone().unwrap_or(prev_text);
    let pairs = crate::learn::word_diff(&base, &new_text);
    if pairs.is_empty() {
        return crate::learn::LearnOutcome::default();
    }
    let outcome = crate::learn::record_corrections(&mut settings, &pairs);
    let _ = settings::save(&app, &settings);
    if !outcome.promoted.is_empty() {
        app.emit("analytics-event", serde_json::json!({ "event": "correction_learned", "props": {} })).ok();
    }
    outcome
}

#[tauri::command]
pub fn get_recording_audio<R: Runtime>(app: AppHandle<R>, filename: String) -> Option<String> {
    history::get_audio_base64(&app, &filename)
}

// ── Analytics ──────────────────────────────────────────────────────────────

/// Fire-and-forget analytics event. The opt-in guard lives in `analytics::track`;
/// this command is a thin bridge from the frontend. Uses the concrete AppHandle
/// because EventTracker is only implemented for the concrete (Wry) runtime.
#[tauri::command]
pub fn track_event(app: tauri::AppHandle, event: String, props: Option<serde_json::Value>) {
    crate::analytics::track(&app, &event, props);
}

/// Permite al frontend escribir al log de disco (p.ej. errores del updater JS,
/// que de otro modo solo irían a la consola del webview, inaccesible en prod).
#[tauri::command]
pub fn log_frontend(msg: String) {
    log::warn!("{msg}");
}
