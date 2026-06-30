use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Runtime};

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
    /// When true, repeated manual corrections in the history are learned as
    /// replacement rules automatically (Phase 3 — auto-learn from corrections).
    #[serde(default = "default_true")]
    pub auto_learn: bool,
    /// Running tally of word-level corrections the user has made. Once a given
    /// from→to reaches the promotion threshold it is added to `replacements`.
    #[serde(default)]
    pub learned_corrections: Vec<LearnedCorrection>,
    /// Visual sensitivity of the widget mic animation: how strongly the wave
    /// reacts to the audio spectrum. 1.0 = default; lower = calmer, higher = more reactive.
    #[serde(default = "default_mic_sensitivity")]
    pub mic_sensitivity: f32,
    /// Opt-in anonymous usage analytics (Aptabase). Default false — no events
    /// are sent until the user consents. Never includes transcribed content.
    #[serde(default)]
    pub analytics_enabled: bool,
}

/// A correction observed from user edits, with how many times it's been seen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedCorrection {
    pub from: String,
    pub to: String,
    pub count: u32,
    /// True once this correction has been promoted into `replacements`.
    #[serde(default)]
    pub promoted: bool,
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
fn default_mic_sensitivity() -> f32 { 1.0 }

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            shortcut: "Alt+Space".to_string(),
            push_to_talk: false,
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
            auto_learn: true,
            learned_corrections: Vec::new(),
            mic_sensitivity: default_mic_sensitivity(),
            analytics_enabled: false,
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
    crate::paths::data_dir(app).join("settings.json")
}

pub fn init<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let path = settings_path(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    if !path.exists() {
        // First run: choose model based on hardware (arch + RAM).
        // Default::default() keeps selected_model = "large-v3-turbo" for
        // deserialisation of existing settings.json — do NOT change that field.
        // Only override it here, in the first-run branch.
        let mut s = AppSettings::default();
        s.selected_model = crate::compat::hardware_default_model().to_string();
        let json = serde_json::to_string_pretty(&s).unwrap();
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
    // Write to a temp file then rename — an atomic swap, so a concurrent writer
    // (e.g. save_settings vs correct_history_entry) can never leave a half-written
    // or corrupt settings.json.
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json).map_err(|e| tauri::Error::Anyhow(e.into()))?;
    fs::rename(&tmp, &path).map_err(|e| tauri::Error::Anyhow(e.into()))?;
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

    #[test]
    fn analytics_disabled_by_default_and_for_old_settings() {
        // New default
        assert!(!AppSettings::default().analytics_enabled);
        // Old settings.json without the field must deserialize to false
        let old = r#"{"shortcut":"Alt+Space","push_to_talk":true,"selected_language":"auto","selected_model":"base","autostart":false,"onboarding_done":true,"widget_position":"center"}"#;
        let s: AppSettings = serde_json::from_str(old).expect("old settings.json must deserialize");
        assert!(!s.analytics_enabled);
    }

    #[test]
    fn default_recording_mode_is_toggle() {
        // toggle = push_to_talk falso por defecto
        assert_eq!(AppSettings::default().push_to_talk, false);
    }
}
