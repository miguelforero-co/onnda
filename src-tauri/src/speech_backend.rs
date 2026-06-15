//! Apple SpeechAnalyzer backend (macOS 26, on-device, Neural Engine).
//!
//! Unlike `WhisperBackend` this is not a synchronous `TranscriptionBackend`:
//! transcription runs in an external Swift sidecar (`binaries/asr`) invoked via
//! tauri-plugin-shell, so the entry point is async and needs an `AppHandle`.
//!
//! Why a sidecar: the new Speech framework is Swift-async/CoreML-first and runs
//! on the ANE — an order of magnitude faster than whisper.cpp on Metal
//! (~0.15s vs ~1.7s on real Spanish dictation) and it punctuates/capitalizes
//! automatically. There is no mature Rust→ANE binding, so we shell out.

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{anyhow, Result};
use serde::Deserialize;
use tauri::{AppHandle, Runtime};
use tauri_plugin_shell::ShellExt;

/// Selectable model id that routes transcription to this backend.
pub const APPLE_MODEL_ID: &str = "apple-speech";

#[derive(Deserialize)]
struct SidecarOut {
    ok: bool,
    text: Option<String>,
    #[allow(dead_code)]
    latency_s: Option<f64>,
    #[allow(dead_code)]
    locale: Option<String>,
    error: Option<String>,
}

static WAV_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Write mono f32 samples as a 16-bit PCM WAV to a unique temp file.
/// AVAudioFile on the Swift side resamples/handles whatever rate we pass.
fn write_temp_wav(samples: &[f32], sample_rate: u32) -> Result<PathBuf> {
    let n = WAV_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let path = std::env::temp_dir().join(format!("vozlocal-asr-{pid}-{n}.wav"));

    let num_samples = samples.len() as u32;
    let data_len = num_samples * 2; // 16-bit mono
    let byte_rate = sample_rate * 2;

    let mut buf: Vec<u8> = Vec::with_capacity(44 + data_len as usize);
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&(36 + data_len).to_le_bytes());
    buf.extend_from_slice(b"WAVE");
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // PCM fmt chunk size
    buf.extend_from_slice(&1u16.to_le_bytes()); // audio format = PCM
    buf.extend_from_slice(&1u16.to_le_bytes()); // channels = mono
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes()); // block align
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        buf.extend_from_slice(&v.to_le_bytes());
    }

    let mut f = std::fs::File::create(&path)?;
    f.write_all(&buf)?;
    Ok(path)
}

fn parse_sidecar_stdout(stdout: &[u8]) -> Result<String> {
    let text = String::from_utf8_lossy(stdout);
    // Take the first line that parses as the sidecar JSON object.
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with('{') {
            continue;
        }
        if let Ok(out) = serde_json::from_str::<SidecarOut>(line) {
            if out.ok {
                return Ok(out.text.unwrap_or_default());
            }
            return Err(anyhow!(out.error.unwrap_or_else(|| "ASR sidecar error".into())));
        }
    }
    Err(anyhow!("could not parse ASR sidecar output: {}", text.trim()))
}

/// Transcribe mono f32 samples with Apple SpeechAnalyzer via the sidecar.
/// `language` is "es" / "en" / "auto" etc.; the sidecar maps it to a locale.
pub async fn apple_transcribe<R: Runtime>(
    app: &AppHandle<R>,
    samples: &[f32],
    sample_rate: u32,
    language: &str,
) -> Result<String> {
    let wav = write_temp_wav(samples, sample_rate)?;
    let wav_str = wav.to_string_lossy().to_string();
    let locale = if language.is_empty() { "auto" } else { language };

    let result = app
        .shell()
        .sidecar("asr")
        .map_err(|e| anyhow!("sidecar not available: {e}"))?
        .args([wav_str, locale.to_string()])
        .output()
        .await
        .map_err(|e| anyhow!("failed to run ASR sidecar: {e}"));

    let _ = std::fs::remove_file(&wav);

    let output = result?;
    if output.stdout.is_empty() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("ASR sidecar produced no output. stderr: {}", err.trim()));
    }
    parse_sidecar_stdout(&output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_is_valid_and_unique() {
        let s = vec![0.0f32, 0.5, -0.5, 1.0, -1.0];
        let p1 = write_temp_wav(&s, 16000).unwrap();
        let p2 = write_temp_wav(&s, 16000).unwrap();
        assert_ne!(p1, p2, "temp paths must be unique");
        let bytes = std::fs::read(&p1).unwrap();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[36..40], b"data");
        // 44-byte header + 2 bytes/sample
        assert_eq!(bytes.len(), 44 + s.len() * 2);
        let _ = std::fs::remove_file(&p1);
        let _ = std::fs::remove_file(&p2);
    }

    #[test]
    fn parses_ok_json() {
        let j = br#"{"ok":true,"text":"hola mundo","latency_s":0.1,"locale":"es-ES"}"#;
        assert_eq!(parse_sidecar_stdout(j).unwrap(), "hola mundo");
    }

    #[test]
    fn parses_error_json() {
        let j = br#"{"ok":false,"error":"locale not supported"}"#;
        assert!(parse_sidecar_stdout(j).is_err());
    }

    #[test]
    fn skips_log_noise_before_json() {
        let j = b"[asr] some log\n{\"ok\":true,\"text\":\"x\"}\n";
        assert_eq!(parse_sidecar_stdout(j).unwrap(), "x");
    }
}
