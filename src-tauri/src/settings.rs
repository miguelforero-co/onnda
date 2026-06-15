use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Runtime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub shortcut: String,
    pub push_to_talk: bool,
    pub selected_language: String,
    pub selected_model: String,
    pub autostart: bool,
    pub onboarding_done: bool,
    pub widget_position: String, // "center" | "left" | "right"
    /// Passed to Whisper as initial_prompt to bias recognition toward these terms.
    /// Comma or newline separated, e.g. "GitHub, Claude Code, Node.js, TypeScript"
    #[serde(default)]
    pub custom_words: String,
    /// Jaro-Winkler similarity threshold for post-transcription word correction (0.0–1.0).
    /// 0.85 catches obvious typos without false positives.
    #[serde(default = "default_word_correction_threshold")]
    pub word_correction_threshold: f32,
    /// Single on/off toggle for all feedback cues (start/stop/cancel).
    /// Replaces the legacy per-event flags below, which are kept only for
    /// settings.json back-compat and are no longer read.
    #[serde(default = "default_true")]
    pub sounds_enabled: bool,
    #[serde(default)]
    pub sound_on_listen: bool,
    #[serde(default)]
    pub sound_on_stop: bool,
    #[serde(default)]
    pub sound_on_cancel: bool,
    #[serde(default)]
    pub pause_media: bool,
    /// Custom vocabulary as discrete items (D-19/D-20). Derived once from
    /// `custom_words` (legacy CSV) on first load. The backend still receives
    /// the joined string (`dictionary.join(", ")`) as Whisper initial_prompt.
    #[serde(default)]
    pub dictionary: Vec<String>,
    /// Deterministic post-transcription find/replace rules + voice snippets.
    /// Applied to every transcription (dictation and file) after vocabulary
    /// correction, for BOTH engines. This is how custom vocabulary reaches the
    /// Apple engine, which has no initial_prompt: e.g. "air table" -> "Airtable".
    #[serde(default)]
    pub replacements: Vec<Replacement>,
}

/// One post-transcription replacement. `from` -> `to`. When `regex` is false the
/// match is a case-insensitive literal (multi-word allowed, e.g. snippets like
/// "mi correo" -> "hello@example.com"); when true, `from` is a regex pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replacement {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub regex: bool,
}

fn default_word_correction_threshold() -> f32 { 0.85 }
fn default_true() -> bool { true }

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            shortcut: "Alt+Space".to_string(),
            push_to_talk: true,
            selected_language: "auto".to_string(),
            selected_model: "large-v3-turbo".to_string(),
            autostart: false,
            onboarding_done: false,
            widget_position: "center".to_string(),
            custom_words: String::new(),
            word_correction_threshold: default_word_correction_threshold(),
            sounds_enabled: true,
            sound_on_listen: false,
            sound_on_stop: false,
            sound_on_cancel: false,
            pause_media: false,
            dictionary: Vec::new(),
            replacements: Vec::new(),
        }
    }
}

/// Idempotent migration: derive `dictionary` items from the legacy
/// comma/newline-separated `custom_words` ONLY when `dictionary` is still empty.
/// Never clobbers user edits once items exist.
pub fn migrate_dictionary(dictionary: &[String], custom_words: &str) -> Vec<String> {
    if !dictionary.is_empty() {
        return dictionary.to_vec();
    }
    custom_words
        .split([',', '\n'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// In-process cache so shortcut handlers don't hit disk on every key event.
static CACHE: Mutex<Option<AppSettings>> = Mutex::new(None);

fn settings_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("no app data dir")
        .join("settings.json")
}

pub fn init<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let path = settings_path(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    if !path.exists() {
        let json = serde_json::to_string_pretty(&AppSettings::default()).unwrap();
        fs::write(&path, json).ok();
    }
    Ok(())
}

pub fn load<R: Runtime>(app: &AppHandle<R>) -> AppSettings {
    if let Some(cached) = CACHE.lock().unwrap().as_ref() {
        return cached.clone();
    }
    let path = settings_path(app);
    let mut settings: AppSettings = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    // Idempotent one-time migration of the legacy CSV custom_words into discrete
    // dictionary items (D-19/D-20). Only derives when dictionary is still empty.
    settings.dictionary = migrate_dictionary(&settings.dictionary, &settings.custom_words);
    *CACHE.lock().unwrap() = Some(settings.clone());
    settings
}

pub fn save<R: Runtime>(app: &AppHandle<R>, settings: &AppSettings) -> tauri::Result<()> {
    let path = settings_path(app);
    let json = serde_json::to_string_pretty(settings).unwrap();
    fs::write(path, json).map_err(|e| tauri::Error::Anyhow(e.into()))?;
    *CACHE.lock().unwrap() = Some(settings.clone());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_new_fields_default_false() {
        let old = r#"{"shortcut":"Alt+Space","push_to_talk":true,"selected_language":"auto","selected_model":"base","autostart":false,"onboarding_done":true,"widget_position":"center"}"#;
        let s: AppSettings = serde_json::from_str(old).expect("old settings.json must deserialize");
        assert!(!s.sound_on_listen);
        assert!(!s.sound_on_stop);
        assert!(!s.sound_on_cancel);
        assert!(!s.pause_media);
        assert!(s.dictionary.is_empty());
    }

    #[test]
    fn dictionary_migration_from_csv() {
        let derived = migrate_dictionary(&[], "GitHub, Claude Code\nNode.js");
        assert_eq!(derived, vec!["GitHub", "Claude Code", "Node.js"]);
    }

    #[test]
    fn dictionary_migration_idempotent() {
        let existing = vec!["x".to_string()];
        let result = migrate_dictionary(&existing, "GitHub, Claude Code");
        assert_eq!(result, vec!["x"]);
    }

    #[test]
    fn dictionary_join_for_prompt() {
        let dict = vec!["GitHub".to_string(), "Claude Code".to_string()];
        assert_eq!(dict.join(", "), "GitHub, Claude Code");
    }
}
