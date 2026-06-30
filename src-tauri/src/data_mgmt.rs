//! Data management: reveal app data dir, clear history+audio, clear downloaded models.
//!
//! SECURITY: every path is derived from `app_data_dir()` — these commands never
//! accept a path argument from the frontend, so no path traversal / arbitrary
//! deletion is possible. `clear_models` only removes `*.bin` files so it cannot
//! nuke unrelated files. The user-facing confirmation dialog (UI-SPEC) gates the
//! call in the UI; the commands are safe on their own regardless.
//!
//! NOTE: `clear_history`, `get_storage_usage`, and `reveal_data_dir` operan sobre
//! el directorio de datos de la app (donde viven settings/history/recordings).
//! `clear_models` apunta a `models/`, que cuelga del mismo root pero es global.

use std::fs;
use tauri::{AppHandle, Runtime};

/// Open the app data directory in Finder, showing its contents.
///
/// The app identifier is `com.onnda.app`, so the data dir ends in `.app`.
/// Plain `open <dir>` tries to LAUNCH it as an application bundle (silent failure).
/// AppleScript's `open folder` forces folder semantics; fallback to `open -R` (reveal parent).
#[tauri::command]
pub fn reveal_data_dir<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let dir = crate::paths::data_dir(&app);
    let dir_str = dir.to_string_lossy().to_string();
    // El data dir termina en ".app" (identifier com.onnda.app) → macOS lo tipa como
    // application bundle y `open <dir>` lo LANZA en vez de abrir su contenido.
    // AppleScript `open folder` fuerza semántica de carpeta y muestra el contenido.
    let script = format!(
        "tell application \"Finder\"\nopen folder (POSIX file \"{}\")\nactivate\nend tell",
        dir_str.replace('\\', "\\\\").replace('"', "\\\"")
    );
    let status = std::process::Command::new("osascript")
        .args(["-e", &script])
        .status()
        .map_err(|e| format!("osascript spawn failed: {e}"))?;
    if status.success() {
        return Ok(());
    }
    // Fallback: revelar/seleccionar en el padre.
    std::process::Command::new("open")
        .args(["-R", &dir_str])
        .status()
        .map_err(|e| format!("open -R failed: {e}"))?;
    Ok(())
}

/// Delete all transcription history (history.json) and all stored audio recordings.
/// Recreates an empty recordings dir so history::init's invariant holds.
#[tauri::command]
pub fn clear_history<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let dir = crate::paths::data_dir(&app);
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
/// the `recordings` directory plus `history.json` if present, matching what
/// `clear_history` would delete. Feeds the "X MB en uso" metric in the frontend.
/// Missing dir/file → counts as 0.
#[tauri::command]
pub fn get_storage_usage<R: Runtime>(app: AppHandle<R>) -> u64 {
    let dir = crate::paths::data_dir(&app);
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
    let dir = crate::paths::data_dir(&app).join("models");
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
