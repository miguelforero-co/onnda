//! Data management: reveal app data dir, clear history+audio, clear downloaded models.
//!
//! SECURITY: every path is derived from `app_data_dir()` — these commands never
//! accept a path argument from the frontend, so no path traversal / arbitrary
//! deletion is possible. `clear_models` only removes `*.bin` files so it cannot
//! nuke unrelated files. The user-facing confirmation dialog (UI-SPEC) gates the
//! call in the UI; the commands are safe on their own regardless.

use std::fs;
use tauri::{AppHandle, Manager, Runtime};

fn data_dir<R: Runtime>(app: &AppHandle<R>) -> Option<std::path::PathBuf> {
    app.path().app_data_dir().ok()
}

/// Open the app data directory in Finder (macOS).
#[tauri::command]
pub fn reveal_data_dir<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let dir = data_dir(&app).ok_or("sin directorio de datos")?;
    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&dir)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Delete all transcription history (history.json) and all stored audio recordings.
/// Scoped to `app_data_dir`; recreates an empty recordings dir so history::init's
/// invariant holds.
#[tauri::command]
pub fn clear_history<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let dir = data_dir(&app).ok_or("sin directorio de datos")?;
    let hist = dir.join("history.json");
    if hist.exists() {
        fs::remove_file(&hist).map_err(|e| e.to_string())?;
    }
    let rec = dir.join("recordings");
    if rec.exists() {
        fs::remove_dir_all(&rec).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&rec).ok(); // recreate empty so history::init invariant holds
    Ok(())
}

/// Delete downloaded models. Restricted to `*.bin` files inside
/// `app_data_dir/models` so unrelated files are never touched.
#[tauri::command]
pub fn clear_models<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let dir = data_dir(&app)
        .ok_or("sin directorio de datos")?
        .join("models");
    if dir.exists() {
        for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let p = entry.map_err(|e| e.to_string())?.path();
            if p.extension().and_then(|e| e.to_str()) == Some("bin") {
                fs::remove_file(&p).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}
