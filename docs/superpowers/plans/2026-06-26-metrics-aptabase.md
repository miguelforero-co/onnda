# Aptabase Metrics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add anonymous, opt-in usage analytics (active users, transcription counts, usage, length) via Aptabase, never sending dictated content.

**Architecture:** A central Rust `analytics` module wraps the official `tauri-plugin-aptabase`. Every event flows through `analytics::track`, which is a no-op unless the new `analytics_enabled` setting is true. The frontend calls a `track_event` command; the opt-in guard lives only in Rust.

**Tech Stack:** Rust, Tauri 2, `tauri-plugin-aptabase`, SvelteKit/Svelte 5.

## Global Constraints

- Never send transcribed text, file names, or any PII — only counts and numeric/categorical props.
- Analytics is **opt-in**: `analytics_enabled` defaults to `false`; no events until consent.
- `track` is fire-and-forget: failures never block UX and never panic.
- App must behave identically offline.
- `cargo check` and `npm run check` must stay green.
- Aptabase key (`APTABASE_APP_KEY`) is an external prerequisite; the code builds with the key read from an env/const.

---

### Task 1: Add `analytics_enabled` setting (default off)

**Files:**
- Modify: `src-tauri/src/settings.rs` (struct `AppSettings`, `Default`, tests module)

**Interfaces:**
- Produces: `AppSettings.analytics_enabled: bool` (serde default `false`).

- [ ] **Step 1: Write the failing test**

In `settings.rs` `mod tests`, add:

```rust
#[test]
fn analytics_disabled_by_default_and_for_old_settings() {
    // New default
    assert!(!AppSettings::default().analytics_enabled);
    // Old settings.json without the field must deserialize to false
    let old = r#"{"shortcut":"Alt+Space","push_to_talk":true,"selected_language":"auto","selected_model":"base","autostart":false,"onboarding_done":true,"widget_position":"center"}"#;
    let s: AppSettings = serde_json::from_str(old).expect("old settings.json must deserialize");
    assert!(!s.analytics_enabled);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml analytics_disabled_by_default`
Expected: FAIL — no field `analytics_enabled`.

- [ ] **Step 3: Add the field and default**

In `AppSettings` struct, after `mic_sensitivity`:

```rust
    /// Opt-in anonymous usage analytics (Aptabase). Default false — no events
    /// are sent until the user consents. Never includes transcribed content.
    #[serde(default)]
    pub analytics_enabled: bool,
```

In the `Default for AppSettings` impl, add `analytics_enabled: false,` alongside the other fields.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml analytics_disabled_by_default`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/settings.rs
git commit -m "feat(analytics): add opt-in analytics_enabled setting (default off)"
```

---

### Task 2: Pure helper for transcription event props

**Files:**
- Create: `src-tauri/src/analytics.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod analytics;`)

**Interfaces:**
- Produces: `analytics::transcription_props(engine: &str, model: &str, language: &str, source: &str, text: &str, duration_ms: i64) -> serde_json::Value` returning an object with `engine, model, language, source, word_count, char_count, duration_ms` and **no** raw text.

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/analytics.rs` with only the test:

```rust
use serde_json::json;

pub fn transcription_props(
    engine: &str,
    model: &str,
    language: &str,
    source: &str,
    text: &str,
    duration_ms: i64,
) -> serde_json::Value {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn props_count_not_content() {
        let v = transcription_props("whisper", "large-v3-turbo", "es", "dictation", "hola que tal", 1700);
        assert_eq!(v["word_count"], json!(3));
        assert_eq!(v["char_count"], json!(11));
        assert_eq!(v["engine"], json!("whisper"));
        assert_eq!(v["source"], json!("dictation"));
        // The raw text must never appear anywhere in the payload.
        assert!(!v.to_string().contains("hola"));
    }
}
```

Add `mod analytics;` to `src-tauri/src/lib.rs` (with the other `mod` lines).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml props_count_not_content`
Expected: FAIL — `unimplemented!()` panics.

- [ ] **Step 3: Implement**

Replace the function body:

```rust
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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml props_count_not_content`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/analytics.rs src-tauri/src/lib.rs
git commit -m "feat(analytics): transcription_props helper (counts, never content)"
```

---

### Task 3: Add the Aptabase plugin and the guarded `track`

**Files:**
- Modify: `src-tauri/Cargo.toml` (dependency)
- Modify: `src-tauri/src/analytics.rs` (`track`, `app_key`)
- Modify: `src-tauri/src/lib.rs` (register plugin, fire `app_launched`)

**Interfaces:**
- Consumes: `AppSettings.analytics_enabled` (Task 1), `transcription_props` (Task 2).
- Produces: `analytics::track<R: Runtime>(app: &AppHandle<R>, event: &str, props: Option<serde_json::Value>)` — no-op when disabled; `analytics::app_key() -> &'static str`.

- [ ] **Step 1: Add the dependency**

In `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
# Anonymous, privacy-first usage analytics (opt-in). Tauri 2 line.
tauri-plugin-aptabase = "1"
```

- [ ] **Step 2: Implement `app_key` and `track`**

Append to `src-tauri/src/analytics.rs`:

```rust
use tauri::{AppHandle, Runtime};
use tauri_plugin_aptabase::EventTracker;

/// Aptabase App Key. Not secret (ships in client apps). Read from the
/// APTABASE_APP_KEY env at build time; falls back to empty (analytics no-op).
pub fn app_key() -> &'static str {
    option_env!("APTABASE_APP_KEY").unwrap_or("")
}

/// Fire-and-forget event. No-op unless the user opted in. Never panics.
pub fn track<R: Runtime>(app: &AppHandle<R>, event: &str, props: Option<serde_json::Value>) {
    if app_key().is_empty() {
        return;
    }
    let enabled = crate::settings::load(app).analytics_enabled;
    if !enabled {
        return;
    }
    app.track_event(event, props);
}
```

- [ ] **Step 3: Register the plugin and fire `app_launched`**

In `src-tauri/src/lib.rs`, add the plugin to the builder chain (after `tauri_plugin_fs::init()`):

```rust
        .plugin(
            tauri_plugin_aptabase::Builder::new(analytics::app_key()).build(),
        )
```

In the `.setup(|app| { ... })` closure, after `mic_permission::request_if_needed();`, add:

```rust
            analytics::track(app.handle(), "app_launched", None);
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: builds clean (warnings ok).

- [ ] **Step 5: Run the analytics tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml analytics`
Expected: PASS (helper tests still green).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/analytics.rs src-tauri/src/lib.rs
git commit -m "feat(analytics): register Aptabase plugin + guarded track + app_launched"
```

---

### Task 4: `track_event` command + frontend helper

**Files:**
- Modify: `src-tauri/src/commands.rs` (new command `track_event`)
- Modify: `src-tauri/src/lib.rs` (register command in `generate_handler!`)
- Create: `src/lib/analytics.ts`

**Interfaces:**
- Consumes: `analytics::track` (Task 3).
- Produces: command `track_event(event: String, props: Option<serde_json::Value>)`; TS `track(event: string, props?: Record<string, unknown>): Promise<void>`.

- [ ] **Step 1: Add the command**

In `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub fn track_event<R: Runtime>(
    app: AppHandle<R>,
    event: String,
    props: Option<serde_json::Value>,
) {
    crate::analytics::track(&app, &event, props);
}
```

- [ ] **Step 2: Register the command**

In `src-tauri/src/lib.rs` `generate_handler!`, add `commands::track_event,` to the list.

- [ ] **Step 3: Frontend helper**

Create `src/lib/analytics.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";

/** Fire-and-forget. The opt-in guard lives in Rust; this never throws. */
export async function track(event: string, props?: Record<string, unknown>): Promise<void> {
  try {
    await invoke("track_event", { event, props: props ?? null });
  } catch {
    // analytics must never break the UI
  }
}
```

- [ ] **Step 4: Verify**

Run: `cargo check --manifest-path src-tauri/Cargo.toml && npm run check`
Expected: both green.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src/lib/analytics.ts
git commit -m "feat(analytics): track_event command + TS helper"
```

---

### Task 5: Instrument `transcription_completed` (dictation + file)

**Files:**
- Modify: `src-tauri/src/commands.rs` (end of dictation transcription) and/or `src-tauri/src/recording.rs` (`transcribe_file`)

**Interfaces:**
- Consumes: `analytics::track`, `analytics::transcription_props`.

- [ ] **Step 1: Instrument dictation**

In `commands.rs`, at the point where a dictation transcription has produced its final `text` (just before/after it is pasted and pushed to history), add:

```rust
    let props = crate::analytics::transcription_props(
        engine_name,           // "whisper" | "apple"
        &settings.selected_model,
        &settings.selected_language,
        "dictation",
        &final_text,
        elapsed_ms,            // i64 milliseconds of the recording
    );
    crate::analytics::track(&app, "transcription_completed", Some(props));
```

Use the engine/model/language/duration values already in scope at that call site; if `engine_name` or `elapsed_ms` are not yet bound, derive them from the existing variables (the selected backend label and the recording timer).

- [ ] **Step 2: Instrument file import**

In `recording.rs` `transcribe_file`, after the file transcription text is produced, add the analogous call with `source = "file"`:

```rust
    let props = crate::analytics::transcription_props(
        engine_name, &model, &language, "file", &text, duration_ms,
    );
    crate::analytics::track(&app, "file_imported", Some(props));
```

- [ ] **Step 3: Verify compile**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: clean.

- [ ] **Step 4: Manual smoke (opt-in off)**

With `analytics_enabled=false` (default), dictate once. Confirm via `read_network_requests` (or Aptabase dashboard) that **no** event was sent.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/recording.rs
git commit -m "feat(analytics): emit transcription_completed + file_imported"
```

---

### Task 6: Opt-in toggle in Settings + onboarding consent step

**Files:**
- Modify: `src/routes/+page.svelte` (Settings section: add a Privacy/Datos toggle bound to `settings.analytics_enabled`; onboarding: add a consent step)

**Interfaces:**
- Consumes: `get_settings`/`save_settings` (existing), the `analytics_enabled` field.

- [ ] **Step 1: Settings toggle**

In the Ajustes section, add a Toggle row (reusing the existing `Toggle`/`Row` components) labeled "Estadísticas anónimas de uso" with helper text "Nunca enviamos lo que dictas." bound to `settings.analytics_enabled`, saving via the existing settings-save path.

- [ ] **Step 2: Onboarding consent**

Add a step in the onboarding flow (after models, before `onboarding_done = true`) that presents the same opt-in with an explicit "Permitir" / "No, gracias" choice, writing `settings.analytics_enabled` accordingly.

- [ ] **Step 3: Verify**

Run: `npm run check`
Expected: green.

- [ ] **Step 4: Manual smoke (opt-in on)**

Enable the toggle, dictate once; confirm exactly one `transcription_completed` event with correct props and **no** text in Aptabase.

- [ ] **Step 5: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat(analytics): opt-in toggle in Settings + onboarding consent"
```

---

### Task 7: Instrument secondary events

**Files:**
- Modify: `src-tauri/src/models.rs` (`download_model`), `src-tauri/src/commands.rs` (engine change path), `src-tauri/src/learn.rs` (promotion)

**Interfaces:**
- Consumes: `analytics::track`.

- [ ] **Step 1: `model_downloaded`**

In `models.rs` `download_model`, after a successful + verified download:

```rust
    crate::analytics::track(&app, "model_downloaded", Some(serde_json::json!({ "model": model_id })));
```

- [ ] **Step 2: `engine_changed`**

Where the selected engine/model changes in settings save, emit:

```rust
    crate::analytics::track(&app, "engine_changed", Some(serde_json::json!({ "engine": engine_label })));
```

- [ ] **Step 3: `correction_learned`**

In `learn.rs`, when a correction is promoted to a replacement rule:

```rust
    crate::analytics::track(&app, "correction_learned", None);
```

- [ ] **Step 4: Verify**

Run: `cargo check --manifest-path src-tauri/Cargo.toml && cargo test --manifest-path src-tauri/Cargo.toml`
Expected: green.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/commands.rs src-tauri/src/learn.rs
git commit -m "feat(analytics): model_downloaded, engine_changed, correction_learned"
```

---

## Self-Review

- **Spec coverage:** opt-in default off (Task 1, 6) ✓; events `app_launched` (3), `transcription_completed` (5), `file_imported` (5), `model_downloaded`/`engine_changed`/`correction_learned` (7) ✓; never-content guarantee (Task 2 test) ✓; central guard in Rust (Task 3) ✓; config via `APTABASE_APP_KEY` env, no-op when empty (Task 3) ✓; fire-and-forget/offline-safe (Task 3, 4) ✓.
- **External prerequisite:** create Aptabase account → set `APTABASE_APP_KEY` before release builds.
- **Placeholder scan:** instrumentation call sites (Task 5/7) reference existing in-scope variables; the implementer binds engine/model/language/duration from the surrounding code — values named, not invented.
