//! Model catalogue, download, status-check, and path-resolution helpers.
//!
//! `resolve_model_path` is the single canonical implementation of the
//! "primary = ggml-{model}.bin / fallback = ggml-base.bin / exists()" logic
//! that was previously duplicated 4× in commands.rs.

use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager, Runtime};

// ── Path helpers ──────────────────────────────────────────────────────────────

/// Returns the directory where Whisper model files are stored.
pub(crate) fn models_dir<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|p| p.join("models"))
}

/// Resolve the path to a Whisper model binary inside `dir`.
///
/// Strategy (same logic previously duplicated 4× in commands.rs):
///   1. Primary: `dir/ggml-{model_name}.bin`  — exact match for the selected model.
///   2. Fallback: `dir/ggml-base.bin`          — always-present safe fallback.
///   3. Neither exists → `None`.
pub(crate) fn resolve_model_path(dir: &Path, model_name: &str) -> Option<PathBuf> {
    let primary = dir.join(format!("ggml-{}.bin", model_name));
    let fallback = dir.join("ggml-base.bin");
    if primary.exists() {
        Some(primary)
    } else if fallback.exists() {
        Some(fallback)
    } else {
        None
    }
}

// ── Structs ───────────────────────────────────────────────────────────────────

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
    /// Some(msg) cuando el modelo existe pero no está disponible en este hardware —
    /// la UI lo muestra deshabilitado con el mensaje como explicación (tooltip o subtítulo).
    /// None cuando el modelo puede usarse normalmente en esta máquina.
    pub disabled_reason: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ModelStatus {
    pub ready: bool,
    pub model_id: String,
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_models<R: Runtime>(app: AppHandle<R>) -> Vec<ModelInfo> {
    let dir = models_dir(&app);
    let model_exists = |name: &str| {
        dir.as_ref()
            .map(|p| p.join(format!("ggml-{}.bin", name)).exists())
            .unwrap_or(false)
    };

    // Gate del motor Apple: requiere Apple Silicon (aarch64) + macOS ≥ 26 + sidecar presente.
    // En el binario x86_64 el bloque #[cfg(target_arch = "aarch64")] se elimina en compilación.
    let apple_disabled_reason: Option<String> = {
        #[cfg(target_arch = "aarch64")]
        {
            if crate::compat::macos_major_version() >= 26 && crate::compat::sidecar_available(&app) {
                None
            } else if crate::compat::macos_major_version() >= 26 {
                Some("ASR sidecar not found in this bundle".to_string())
            } else {
                Some("Requires macOS 26 (Tahoe) or later".to_string())
            }
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            Some("Requires Apple Silicon + macOS 26".to_string())
        }
    };

    let mut models = vec![
        ModelInfo {
            id: "small".to_string(),
            name: "Whisper Small".to_string(),
            size_mb: 466,
            downloaded: model_exists("small"),
            coming_soon: false,
            disabled_reason: None,
        },
        ModelInfo {
            id: "medium".to_string(),
            name: "Whisper Medium".to_string(),
            size_mb: 1536,
            downloaded: model_exists("medium"),
            coming_soon: false,
            disabled_reason: None,
        },
        ModelInfo {
            id: "large-v3-turbo".to_string(),
            name: "Whisper Large v3 Turbo".to_string(),
            size_mb: 874,
            downloaded: model_exists("large-v3-turbo"),
            coming_soon: false,
            disabled_reason: None,
        },
        ModelInfo {
            id: crate::speech_backend::APPLE_MODEL_ID.to_string(),
            name: "Apple (Neural Engine)".to_string(),
            size_mb: 0,           // on-device, assets managed by macOS
            downloaded: apple_disabled_reason.is_none(),
            coming_soon: false,
            disabled_reason: apple_disabled_reason,
        },
    ];

    // El modelo "base" va primero (es el default de hardware). En Intel (CPU)
    // usamos el cuantizado q5_1 — misma calidad que el base normal pero ~2.8×
    // más rápido y 1/3 del tamaño — así que en x86_64 es el ÚNICO base (no se
    // muestra el full-precision, sería redundante y más lento). En Apple Silicon
    // (Metal) va el base full-precision.
    #[cfg(not(target_arch = "aarch64"))]
    models.insert(
        0,
        ModelInfo {
            id: "base-q5_1".to_string(),
            name: "Whisper Base".to_string(),
            size_mb: 57,
            downloaded: model_exists("base-q5_1"),
            coming_soon: false,
            disabled_reason: None,
        },
    );
    #[cfg(target_arch = "aarch64")]
    models.insert(
        0,
        ModelInfo {
            id: "base".to_string(),
            name: "Whisper Base".to_string(),
            size_mb: 141,
            downloaded: model_exists("base"),
            coming_soon: false,
            disabled_reason: None,
        },
    );

    models
}

#[tauri::command]
pub async fn download_model<R: Runtime>(app: AppHandle<R>, model_id: String) -> Result<(), String> {
    use sha2::{Sha256, Digest};

    // Pinned URLs and expected SHA256 hashes (verified 2026-06-15 via HF x-linked-etag)
    let (url, expected_sha256) = match model_id.as_str() {
        "large-v3-turbo" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/0b364b566045a405be7225ee1e415a073e04da77/ggml-large-v3-turbo-q8_0.bin",
            "317eb69c11673c9de1e1f0d459b253999804ec71ac4c23c17ecf5fbe24e259a1",
        ),
        "base" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/80da2d8bfee42b0e836fc3a9890373e5defc00a6/ggml-base.bin",
            "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        ),
        // Cuantizado q5_1 — default en Intel (CPU): ≈2.8× más rápido que `small`
        // con calidad casi idéntica. Verificado 2026-06-30 vía HF x-linked-etag.
        "base-q5_1" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-base-q5_1.bin",
            "422f1ae452ade6f30a004d7e5c6a43195e4433bc370bf23fac9cc591f01a8898",
        ),
        "small" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/80da2d8bfee42b0e836fc3a9890373e5defc00a6/ggml-small.bin",
            "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
        ),
        "medium" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/80da2d8bfee42b0e836fc3a9890373e5defc00a6/ggml-medium.bin",
            "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
        ),
        other => return Err(format!("Unknown model: {}", other)),
    };

    let dir = models_dir(&app).ok_or("Could not get data directory")?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let dest = dir.join(format!("ggml-{}.bin", model_id));
    let tmp  = dir.join(format!("ggml-{}.bin.tmp", model_id));

    // Clean up any leftover .tmp from a previous failed download
    let _ = tokio::fs::remove_file(&tmp).await;

    let client = reqwest::Client::builder()
        .user_agent("onnda/0.1")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} downloading model", resp.status()));
    }

    let total = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut hasher = Sha256::new();

    let mut file = tokio::fs::File::create(&tmp).await.map_err(|e| e.to_string())?;
    let mut stream = resp.bytes_stream();

    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            e.to_string()
        })?;
        hasher.update(&chunk);
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

    // Flush OS buffers before rename (tokio File does not guarantee flush on drop)
    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);

    // Verify SHA256 before making the file visible — never rename a corrupt download
    let computed = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect::<String>();
    if computed != expected_sha256 {
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(format!(
            "Download of '{}' is corrupt (hash mismatch). Please try again.",
            model_id
        ));
    }

    tokio::fs::rename(&tmp, &dest).await.map_err(|e| e.to_string())?;
    app.emit("download-complete", &model_id).ok();
    app.emit("analytics-event", serde_json::json!({ "event": "model_downloaded", "props": { "model": model_id } })).ok();
    Ok(())
}

/// Proactively check whether the currently selected (or default) model is
/// available on disk so the frontend can show a banner at startup instead of
/// discovering the absence only when the user tries to dictate.
#[tauri::command]
pub fn check_model_status<R: Runtime>(app: AppHandle<R>) -> ModelStatus {
    let settings = crate::settings::load(&app);
    let model_name = if settings.selected_model.is_empty() {
        crate::compat::hardware_default_model().to_string()
    } else {
        settings.selected_model.clone()
    };
    // Apple SpeechAnalyzer uses the on-device Neural Engine — no .bin to download.
    if model_name == crate::speech_backend::APPLE_MODEL_ID {
        return ModelStatus { ready: true, model_id: model_name };
    }
    let Some(dir) = models_dir(&app) else {
        return ModelStatus { ready: false, model_id: model_name };
    };
    let ready = resolve_model_path(&dir, &model_name).is_some();
    ModelStatus { ready, model_id: model_name }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Helper: create an empty file at the given path (and parent dirs).
    fn touch(path: &std::path::Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, b"").unwrap();
    }

    #[test]
    fn resolve_model_path_neither_exists_returns_none() {
        let dir = std::env::temp_dir().join(format!("voz-test-resolve-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        // No files created — both primary and fallback are absent.
        let result = resolve_model_path(&dir, "base");
        fs::remove_dir_all(&dir).ok();
        assert!(result.is_none(), "expected None when no model files exist");
    }

    #[test]
    fn resolve_model_path_only_fallback_returns_fallback() {
        let dir = std::env::temp_dir().join(format!("voz-test-fallback-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        // Create only the fallback (ggml-base.bin), not the primary.
        let fallback = dir.join("ggml-base.bin");
        touch(&fallback);
        let result = resolve_model_path(&dir, "small");
        fs::remove_dir_all(&dir).ok();
        assert_eq!(result, Some(fallback), "expected Some(fallback) when only fallback exists");
    }

    #[test]
    fn resolve_model_path_primary_takes_precedence_over_fallback() {
        let dir = std::env::temp_dir().join(format!("voz-test-primary-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        // Both primary and fallback exist — primary must win.
        let primary = dir.join("ggml-large-v3-turbo.bin");
        let fallback = dir.join("ggml-base.bin");
        touch(&primary);
        touch(&fallback);
        let result = resolve_model_path(&dir, "large-v3-turbo");
        fs::remove_dir_all(&dir).ok();
        assert_eq!(result, Some(primary), "expected Some(primary) when both files exist");
    }

    #[test]
    fn resolve_model_path_only_primary_exists_returns_primary() {
        let dir = std::env::temp_dir().join(format!("voz-test-prim-only-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let primary = dir.join("ggml-medium.bin");
        touch(&primary);
        let result = resolve_model_path(&dir, "medium");
        fs::remove_dir_all(&dir).ok();
        assert_eq!(result, Some(primary), "expected Some(primary) when only primary exists");
    }
}
