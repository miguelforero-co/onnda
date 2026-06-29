//! Rutas de datos de usuario. Sin cuentas locales, todos los datos del usuario
//! (settings/history/recordings) viven directamente en el directorio de datos de
//! la app. `models/` también cuelga de aquí pero es global por diseño.

use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime};

/// Directorio de datos de la app, creado si no existe.
pub fn data_dir<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    let dir = app.path().app_data_dir().expect("no app data dir");
    std::fs::create_dir_all(&dir).ok();
    dir
}
