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

/// Total disk used by saved transcriptions, in bytes: the sum of every file in
/// the `recordings` directory plus `history.json` if present. Reuses the same
/// `app_data_dir` + recordings path logic as `clear_history` / `history.rs`, so
/// the figure matches exactly what `clear_history` would delete. Feeds the
/// "X MB en uso" metric in the frontend. Missing dir/file → counts as 0.
#[tauri::command]
pub fn get_storage_usage<R: Runtime>(app: AppHandle<R>) -> u64 {
    let Some(dir) = data_dir(&app) else {
        return 0;
    };
    let mut total: u64 = 0;

    let rec = dir.join("recordings");
    if let Ok(entries) = fs::read_dir(&rec) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                }
            }
        }
    }

    let hist = dir.join("history.json");
    if let Ok(meta) = fs::metadata(&hist) {
        total += meta.len();
    }

    total
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
