use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_aptabase::EventTracker;

/// Aptabase App Key. Not secret (ships in client apps). Read from the
/// APTABASE_APP_KEY env at build time; falls back to empty (analytics no-op).
pub fn app_key() -> &'static str {
    option_env!("APTABASE_APP_KEY").unwrap_or("")
}

/// Fire-and-forget event. No-op unless the user opted in. Never panics.
/// Note: EventTracker is implemented for the concrete AppHandle (Wry runtime),
/// not the generic AppHandle<R>, so this function is non-generic.
pub fn track(app: &AppHandle, event: &str, props: Option<serde_json::Value>) {
    if app_key().is_empty() {
        return;
    }
    let enabled = crate::settings::load(app).analytics_enabled;
    if !enabled {
        return;
    }
    let _ = app.track_event(event, props);
}

pub fn transcription_props(
    engine: &str,
    model: &str,
    language: &str,
    source: &str,
    text: &str,
    duration_ms: i64,
) -> serde_json::Value {
    let word_count = text.split_whitespace().count() as i64;
    let char_count = text.chars().count() as i64;
    json!({
        "engine": engine,
        "model": model,
        "language": language,
        "source": source,
        "word_count": word_count,
        "char_count": char_count,
        "duration_ms": duration_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn props_count_not_content() {
        let v = transcription_props("whisper", "large-v3-turbo", "es", "dictation", "hola que tal", 1700);
        assert_eq!(v["word_count"], json!(3));
        assert_eq!(v["char_count"], json!(12));
        assert_eq!(v["engine"], json!("whisper"));
        assert_eq!(v["source"], json!("dictation"));
        // The raw text must never appear anywhere in the payload.
        assert!(!v.to_string().contains("hola"));
    }
}
