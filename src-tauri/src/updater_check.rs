//! Update check command (tauri-plugin-updater, check-only fallback).
//!
//! D-14: report a clear up-to-date / update-available status to the UI. If no
//! updater `endpoints` + signing keypair are configured yet (RESEARCH Open Q3),
//! `check()` errors — we surface a benign "up to date" with the error captured
//! so the UI can show "Estás al día" without a hard failure. No auto-install in
//! Phase 1; wiring a real endpoint + keypair is deferred user_setup and lands
//! without code changes here.

use tauri::{AppHandle, Runtime};

#[derive(serde::Serialize)]
pub struct UpdateStatus {
    pub up_to_date: bool,
    pub available_version: Option<String>,
    pub current_version: String,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn check_for_updates<R: Runtime>(app: AppHandle<R>) -> Result<UpdateStatus, String> {
    let current = app.package_info().version.to_string();
    #[cfg(desktop)]
    {
        use tauri_plugin_updater::UpdaterExt;
        match app.updater() {
            Ok(updater) => match updater.check().await {
                Ok(Some(update)) => Ok(UpdateStatus {
                    up_to_date: false,
                    available_version: Some(update.version.clone()),
                    current_version: current,
                    error: None,
                }),
                Ok(None) => Ok(UpdateStatus {
                    up_to_date: true,
                    available_version: None,
                    current_version: current,
                    error: None,
                }),
                // No endpoint / signing infra yet (RESEARCH Open Q3): report
                // check-only, not a hard failure.
                Err(e) => Ok(UpdateStatus {
                    up_to_date: true,
                    available_version: None,
                    current_version: current,
                    error: Some(e.to_string()),
                }),
            },
            Err(e) => Ok(UpdateStatus {
                up_to_date: true,
                available_version: None,
                current_version: current,
                error: Some(e.to_string()),
            }),
        }
    }
    #[cfg(not(desktop))]
    {
        Ok(UpdateStatus {
            up_to_date: true,
            available_version: None,
            current_version: current,
            error: None,
        })
    }
}
