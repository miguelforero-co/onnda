use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp_ms: i64,
    pub text: String,
    pub audio_filename: Option<String>,
    pub duration_secs: f32,
    #[serde(default = "default_source")]
    pub source: String, // "dictation" | "file"
    #[serde(default)]
    pub original_filename: Option<String>, // set for source == "file"
    /// The text exactly as the ASR produced it, captured the first time the user
    /// edits this entry. Lets auto-learn diff original vs corrected. None until edited.
    #[serde(default)]
    pub original_text: Option<String>,
}

fn default_source() -> String {
    "dictation".to_string()
}

fn history_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path().app_data_dir().unwrap().join("history.json")
}

fn recordings_dir<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path().app_data_dir().unwrap().join("recordings")
}

pub fn init<R: Runtime>(app: &AppHandle<R>) {
    fs::create_dir_all(recordings_dir(app)).ok();
}

pub fn load<R: Runtime>(app: &AppHandle<R>) -> Vec<HistoryEntry> {
    let path = history_path(app);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_entry<R: Runtime>(
    app: &AppHandle<R>,
    text: String,
    samples: &[f32],
    sample_rate: u32,
    source: String,
    original_filename: Option<String>,
) -> HistoryEntry {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let id = ts.to_string();
    let duration_secs = samples.len() as f32 / sample_rate as f32;

    let audio_filename = if !samples.is_empty() {
        let filename = format!("{}.wav", id);
        let path = recordings_dir(app).join(&filename);
        let samples_16k = crate::transcription::resample(samples, sample_rate as usize, 16000);
        if write_wav(&path, &samples_16k, 16000).is_ok() {
            Some(filename)
        } else {
            None
        }
    } else {
        None
    };

    let entry = HistoryEntry {
        id,
        timestamp_ms: ts,
        text,
        audio_filename,
        duration_secs,
        source,
        original_filename,
        original_text: None,
    };

    let mut history = load(app);
    history.insert(0, entry.clone());
    history.truncate(200);
    if let Ok(json) = serde_json::to_string_pretty(&history) {
        fs::write(history_path(app), json).ok();
    }
    entry
}

pub fn delete<R: Runtime>(app: &AppHandle<R>, id: &str) {
    let mut history = load(app);
    if let Some(pos) = history.iter().position(|e| e.id == id) {
        let entry = history.remove(pos);
        if let Some(filename) = &entry.audio_filename {
            fs::remove_file(recordings_dir(app).join(filename)).ok();
        }
        if let Ok(json) = serde_json::to_string_pretty(&history) {
            fs::write(history_path(app), json).ok();
        }
    }
}

/// Update an entry's text (a user correction). Captures `original_text` on the
/// first edit so auto-learn can diff ASR-output vs corrected. Returns
/// (previous_text, updated_entry), or None if the id isn't found.
pub fn update_text<R: Runtime>(
    app: &AppHandle<R>,
    id: &str,
    new_text: &str,
) -> Option<(String, HistoryEntry)> {
    let mut history = load(app);
    let pos = history.iter().position(|e| e.id == id)?;
    let prev_text = history[pos].text.clone();
    if history[pos].original_text.is_none() {
        history[pos].original_text = Some(prev_text.clone());
    }
    history[pos].text = new_text.to_string();
    let updated = history[pos].clone();
    if let Ok(json) = serde_json::to_string_pretty(&history) {
        fs::write(history_path(app), json).ok();
    }
    Some((prev_text, updated))
}

pub fn get_audio_base64<R: Runtime>(app: &AppHandle<R>, filename: &str) -> Option<String> {
    let path = recordings_dir(app).join(filename);
    let bytes = fs::read(path).ok()?;
    Some(encode_base64(&bytes))
}

fn encode_base64(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[(n >> 18) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 { TABLE[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[(n & 63) as usize] as char } else { '=' });
    }
    out
}

fn write_wav(path: &PathBuf, samples: &[f32], sample_rate: u32) -> std::io::Result<()> {
    use std::io::Write;
    let data_size = (samples.len() * 2) as u32;
    let mut f = fs::File::create(path)?;
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data_size).to_le_bytes())?;
    f.write_all(b"WAVE")?;
    f.write_all(b"fmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;     // PCM
    f.write_all(&1u16.to_le_bytes())?;     // mono
    f.write_all(&sample_rate.to_le_bytes())?;
    f.write_all(&(sample_rate * 2).to_le_bytes())?; // byte rate
    f.write_all(&2u16.to_le_bytes())?;     // block align
    f.write_all(&16u16.to_le_bytes())?;    // bits per sample
    f.write_all(b"data")?;
    f.write_all(&data_size.to_le_bytes())?;
    for &s in samples {
        let pcm = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        f.write_all(&pcm.to_le_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_history_entry_defaults_to_dictation() {
        let old = r#"{"id":"1","timestamp_ms":0,"text":"hola","audio_filename":null,"duration_secs":1.0}"#;
        let e: HistoryEntry =
            serde_json::from_str(old).expect("old history entry must deserialize");
        assert_eq!(e.source, "dictation");
        assert_eq!(e.original_filename, None);
    }

    #[test]
    fn default_source_constant() {
        assert_eq!(default_source(), "dictation");
    }
}
