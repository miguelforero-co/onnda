# Phase 1: Rediseño del panel de configuración (estilo WhisprFlow) - Pattern Map

**Mapped:** 2026-06-14
**Files analyzed:** 24 (new + modified)
**Analogs found:** 21 / 24 (3 no-analog: file decode, file resample, media-pause)

> **Stack reality check (read before planning):** This is Tauri 2 (`tauri = "2"` confirmed in `src-tauri/Cargo.toml`) + SvelteKit / Svelte 5 runes. CRITICAL deviations from RESEARCH.md:
> - **Settings do NOT use `tauri-plugin-store`.** They persist as a single serde `AppSettings` struct → `app_data_dir/settings.json`, with an in-memory `CACHE: Mutex<Option<AppSettings>>` and `#[serde(default)]` for non-breaking migration (`src-tauri/src/settings.rs`). New fields follow THIS pattern, not a store. Do not introduce `tauri-plugin-store`.
> - **History does NOT use a DB.** It is `Vec<HistoryEntry>` → `history.json` (`src-tauri/src/history.rs`). The `source` field is added here.
> - **Plugins present today:** `opener`, `autostart`, `global-shortcut`, `notification`, `process`, `shell` (per `Cargo.toml`). `dialog`, `fs`, `updater` are NOT installed — adding them = new Cargo deps + new permissions in `src-tauri/capabilities/default.json` (single capability file, windows `["main","widget"]`).
> - **macOS native = objc2 0.6 / objc2-app-kit 0.3 / block2 0.6** (confirmed). The pattern for NSSound + media-key is `notch.rs` / `escape.rs` / `mic_permission.rs` — NOT cocoa/core-foundation.
> - **Frontend is one monolithic `src/routes/+page.svelte`** with an inline `:root` design system. The sidebar shell + section components are EXTRACTED from this file. There are no existing `.svelte` components to copy — components are new files derived from the inline markup/CSS.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/routes/+page.svelte` (MODIFY → shell) | route/orchestrator | event-driven + request-response | itself (current monolith) | exact (in-place) |
| `src/lib/components/Sidebar.svelte` (NEW) | component | event-driven (nav state) | `header`+`.tabs` block in `+page.svelte` (L178-187, 532-540) | role-match (extract) |
| `src/lib/components/Row.svelte` (NEW) | component | request-response | `.row`/`.rows`/`.sep` (L569-577, 561-567) | role-match (extract) |
| `src/lib/components/Toggle.svelte` (NEW) | component | request-response | `.toggle`/`.knob` (L596-609, 321-327) | exact (extract) |
| `src/lib/components/HotkeyRecorder.svelte` (NEW) | component | event-driven (keydown capture) | shortcut `<input>` (L311-318) + escape monitor pattern | partial (new behavior) |
| `src/lib/components/ModelCard.svelte` (NEW) | component | event-driven (download progress) | `.model-settings-row` block (L404-431) | exact (extract) |
| `src/lib/components/PermissionRow.svelte` (NEW) | component | request-response (poll) | `.perm-row` block (L204-228, 626-646) | exact (extract) |
| `src/routes/(sections)/Home.svelte` (NEW) | component | event-driven | onboarding/settings layout in `+page.svelte` | role-match |
| `src/routes/(sections)/Transcripciones.svelte` (NEW) | component | CRUD + file-I/O | history view (L440-483) | exact (extract) |
| `src/routes/(sections)/Diccionario.svelte` (NEW) | component | CRUD (list items) | `.words-block` textarea (L351-360) | partial (string→items) |
| `src/routes/(sections)/Ajustes.svelte` (NEW) | component | request-response | settings view (L290-433) | exact (extract) |
| `src-tauri/src/settings.rs` (MODIFY) | model/config | CRUD (persist) | itself | exact (in-place) |
| `src-tauri/src/commands.rs` (MODIFY) | controller | event-driven + request-response | itself | exact (in-place) |
| `src-tauri/src/history.rs` (MODIFY) | model/store | CRUD + file-I/O | itself (add `source`) | exact (in-place) |
| `src-tauri/src/lib.rs` (MODIFY) | config/bootstrap | — | itself (invoke_handler + plugins) | exact (in-place) |
| `src-tauri/src/sounds.rs` (NEW) | utility (native) | event-driven (play cue) | `notch.rs` / `mic_permission.rs` (objc2 msg_send) | role-match |
| `src-tauri/src/media_pause.rs` (NEW) | utility (native) | event-driven (key press) | `commands.rs::post_cmd_v` CGEvent (L605-633) | role-match |
| `src-tauri/src/hotkey.rs` (use existing `shortcut.rs`) | controller | request-response | `src-tauri/src/shortcut.rs::re_register` | exact (reuse) |
| `src-tauri/src/audio_decode.rs` (NEW) | service | file-I/O + transform | — | **NO ANALOG** (symphonia) |
| file resample → reuse `transcription.rs::resample` | utility | transform | `src-tauri/src/transcription.rs` (L3-14) | exact (reuse) |
| `src-tauri/src/data_mgmt.rs` (NEW) | service | file-I/O (clear/reveal) | `history.rs::delete` + `commands.rs::open_*_settings` | role-match |
| file-transcription command (in `commands.rs`) | controller | file-I/O + transform | `commands.rs::stop_and_transcribe_internal` (L228-383) | role-match |
| updater check command (in `commands.rs`) | controller | request-response | `commands.rs::download_model` (reqwest) | partial |
| `src-tauri/tauri.conf.json` (MODIFY) | config | — | itself (window `main`) | exact (in-place) |
| `src-tauri/capabilities/default.json` (MODIFY) | config | — | itself | exact (in-place) |

## Pattern Assignments

### `src-tauri/src/settings.rs` (MODIFY — add all new persisted flags)

**Analog:** itself. This is the single source of truth for new persisted state. Add `sound_on_listen`, `sound_on_stop`, `sound_on_cancel` (bool), `pause_media` (bool), and `dictionary: Vec<String>` — each with `#[serde(default)]` so old `settings.json` files load unchanged.

**Non-breaking field pattern** (L18-26 — copy verbatim shape):
```rust
#[serde(default)]
pub custom_words: String,
#[serde(default = "default_word_correction_threshold")]
pub word_correction_threshold: f32,
```
```rust
fn default_word_correction_threshold() -> f32 { 0.85 }
```
New fields: bool defaults to `false` via `#[serde(default)]` (no fn needed); `Vec<String>` defaults to empty via `#[serde(default)]`. Mirror each in `impl Default` (L28-42).

**Dictionary migration (D-20):** `custom_words: String` (CSV) → `dictionary: Vec<String>`. Add `dictionary` with `#[serde(default)]`. On `load()`, if `dictionary` is empty AND `custom_words` is non-empty, derive it: split on `[',', '\n']`, trim, filter empties (the EXACT split used in `transcription.rs::correct_words` L23-27 — reuse that logic so backend stays consistent). The backend still receives the joined string (`dictionary.join(", ")`) as `initial_prompt`. Migration must be idempotent — only derive when `dictionary` is empty.

**Persistence flow** (L66-85): `load()` reads cache-or-disk; `save()` writes pretty JSON + updates `CACHE`. Do not bypass this — every new setting rides the same `get_settings`/`save_settings` commands.

---

### `src/routes/+page.svelte` (MODIFY → two-column shell) + extracted section components

**Analog:** itself. The current file IS the design system and contains every pattern the new shell needs. The redesign replaces `header`+`.tabs` (L178-187) with a 200px sidebar and routes `view` to section components.

**State + invoke/listen pattern** (L25-104 — keep this orchestration in `+page.svelte`):
```ts
let settings = $state<Settings>({...});
let view = $state<"onboarding" | "settings" | "history">("settings"); // → extend to home|transcripciones|diccionario|ajustes
onMount(async () => {
  settings = await invoke("get_settings");
  models = await invoke("get_models");
  await checkPerms();
  unlisten.push(
    await listen<DownloadProgress>("download-progress", ({ payload }) => {...}),
    await listen<string>("download-complete", async ({ payload }) => {...}),
  );
});
```

**Debounced save pattern** (L113-119 — reuse for every new toggle; `schedSave(true)` only for hotkey changes):
```ts
function schedSave(scChanged = false) {
  if (!initialized) return;
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(async () => {
    await invoke("save_settings", { newSettings: settings, shortcutChanged: scChanged });
  }, 600);
}
```

**Live permission poll** (L121-124, mount L75-79 — reuse in PermissionRow at 3s interval per UI-SPEC):
```ts
async function checkPerms() {
  micGranted  = await invoke<boolean>("check_mic_permission");
  a11yGranted = await invoke<boolean>("check_accessibility_permission");
}
```

**Active-nav affordance** (extract `.tab.on` L540 → sidebar item; UI-SPEC: active = `--text` on `--bg` pill, radius 6px, NO coral on inactive nav):
```css
.tab{ color:var(--faint); border-radius:6px; transition:color .12s, background .12s; }
.tab:hover{ color:var(--muted); }
.tab.on{ color:var(--text); background:var(--bg); }
```

**Design tokens (`:root` L488-499) — DO NOT redefine.** Move to a shared stylesheet or `:global` so all components reference `--bg/--panel/--text/--muted/--faint/--line/--coral/--amber/--blue/--r` verbatim (D-04 locks the palette).

---

### `src/lib/components/Toggle.svelte` + `Row.svelte` (NEW — extract verbatim)

**Analog:** `+page.svelte` L321-327 (markup) + L596-609 (CSS). Toggle is `36×20px`, knob `16px` — UI-SPEC says do not alter.
```svelte
<label class="row" for={id}>
  <span class="row-label">{label}</span>
  <div class="toggle" class:on={checked}>
    <input id={id} type="checkbox" bind:checked onchange={onchange} />
    <span class="knob"></span>
  </div>
</label>
```
Row block container (L561-577): `.rows` (panel surface, radius `--r`, overflow hidden), `.sep` hairline (`1px var(--line)`, margin `0 12px`), `.row` (`padding:10px 14px; min-height:42px`). Preserve these exact values (UI-SPEC spacing exceptions).

---

### `src/lib/components/ModelCard.svelte` (NEW — extract + expand)

**Analog:** `+page.svelte` model settings rows (L404-431) + download-progress CSS (L749-760).

**Download progress bar** (L416-421, 750-759 — reuse `.dl-bar`/`.dl-pct`):
```svelte
{#if m.downloaded}
  <span class="badge installed">Instalado</span>   <!-- --blue, L615 -->
{:else if prog}
  <div class="dl-bar-wrap"><div class="dl-bar" style="width:{prog.percent}%"></div></div>
  <span class="dl-pct">{Math.round(prog.percent)}%</span>
{:else}
  <button class="link-btn" onclick={() => startDownload(m.id)}>Descargar</button>
{/if}
```
Selected state = coral 1px ring (UI-SPEC). Parakeet card (D-13) = disabled, muted, badge "Próximamente" — NO coral (UI-SPEC color contract).

---

### `src/lib/components/HotkeyRecorder.svelte` (NEW — replaces `<input type=text>` L311-318)

**Analog (idle state):** existing shortcut input + `kbd` styling (L730-734). **Analog (capture/cancel):** the passive-Escape philosophy of `escape.rs` — capture keydown in the DOM, ignore lone modifiers, Escape cancels. Use the RESEARCH.md `captureCombo` shape (L264-283) but validate to global-shortcut accelerator syntax (`Alt+Space` default).

**GOTCHA (project memory + UI-SPEC L163):** re-entrant global-shortcut registration hangs the app. Capture in the frontend; only call `schedSave(true)` AFTER capture completes (which triggers `shortcut::re_register` in Rust). Never register mid-capture.

---

### `src-tauri/src/commands.rs` (MODIFY — hook sounds + pause-media into state machine)

**Analog:** itself. The recording state machine already emits at exactly the three points D-07/D-08 need. Hook native calls at these lines:

**Hook points** (call `sounds::play_*` and `media_pause::*` here):
- **Start listening** → `start_recording_internal` after `app.emit("recording-state", true)` (L120): `sounds::play_listen()`, `media_pause::pause_if_playing()`.
- **Stop + transcribe** → `stop_and_transcribe_internal` after `app.emit("recording-state", false)` (L231): `sounds::play_stop()`, `media_pause::resume_if_paused()`.
- **Cancel** → `cancel_recording_internal` after `app.emit("recording-state", false)` (L393): `sounds::play_cancel()`, `media_pause::resume_if_paused()`. Gate each on the corresponding `settings.*` flag (`settings::load(&app)`).

**File-transcription command** — model after `stop_and_transcribe_internal` (L271-383): load settings, resolve model path (L275-302 verbatim), run `WhisperBackend::new(path).transcribe(&samples, &TranscribeOpts{...})` on `spawn_blocking`, then `crate::transcription::correct_words(...)` (L367), then `history::save_entry` (L374) with the new `source:"file"`. Input PCM comes from `audio_decode` (new) + `transcription::resample` (existing). Run decode/transcribe on `spawn_blocking` and emit progress (anti-pattern: decoding on UI thread — RESEARCH.md).

**New model URLs (D-13)** — extend the `match model_id` in `download_model` (L524-528) and the `vec![]` in `get_models` (L506-519). Add `small`, `medium`, `large-v3-turbo` is present. URL shape (HuggingFace ggerganov/whisper.cpp, L525-526):
```rust
"small"  => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
"medium" => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
```

**Command error convention** (L72-96): `#[tauri::command]` returning `Result<(), String>` via `.map_err(|e| e.to_string())`; events via `app.emit("event-name", payload)`. New commands follow this exactly.

**Autostart already wired** (L85-93) — D-10 is DONE; just verify the toggle reaches `save_settings`:
```rust
#[cfg(desktop)]
{
    use tauri_plugin_autostart::ManagerExt;
    if new_settings.autostart { app.autolaunch().enable().ok(); }
    else { app.autolaunch().disable().ok(); }
}
```

---

### `src-tauri/src/history.rs` (MODIFY — add `source` field, unified store)

**Analog:** itself. Add `source: TranscriptionSource` (`"dictation" | "file"`) to `HistoryEntry` (L6-13) with `#[serde(default)]` so existing `history.json` migrates (defaults to `"dictation"`). Add a `source` param + optional `original_filename` to `save_entry` (L35-76). The `Transcripciones` view filters on `source`. `delete` (L78-89) and `get_audio_base64` (L91-95) are reused unchanged.

**Serde-default migration on the struct** (mirror settings.rs pattern):
```rust
#[serde(default)]  // old entries → "dictation"
pub source: String,
```

---

### `src-tauri/src/sounds.rs` (NEW — native NSSound via objc2)

**Analog:** `mic_permission.rs` (L17-30 objc2 `msg_send!` + `AnyClass::get`) and `notch.rs` (objc2-app-kit usage). D-07: play from Rust because the widget/main window may be hidden — frontend Audio is unreliable there.

**objc2 msg_send pattern to copy** (`mic_permission.rs` L23-29):
```rust
use objc2::runtime::AnyClass;
use objc2::msg_send;
unsafe {
    let Some(cls) = AnyClass::get(c"NSSound") else { return };
    // NSSound soundNamed: for system sounds, or initWithContentsOfFile: for bundled assets
}
```
Bundle 3 short assets (or use NSSound system names). Gate `#[cfg(target_os = "macos")]` with a no-op `#[cfg(not(...))]` twin, exactly like `escape.rs` L44-45.

---

### `src-tauri/src/media_pause.rs` (NEW — CGEvent media key)

**Analog:** `commands.rs::post_cmd_v` (L605-633) — the EXACT CGEvent FFI scaffolding to copy (extern block, `CGEventSourceCreate`, post, `CFRelease`). D-08 needs an `NSSystemDefined` event (subtype 8, `NX_KEYTYPE_PLAY` key 16) down+up instead of a keyboard event. Track an `AtomicBool i_paused` so `resume_if_paused` only re-sends if WE paused (RESEARCH.md Pitfall 4). Researcher confirmed media-key over private MediaRemote.

**FFI/CFRelease pattern to mirror** (L605-633):
```rust
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventSourceCreate(state_id: i32) -> *mut c_void;
    fn CGEventPost(tap: i32, event: *mut c_void);
    fn CFRelease(cf: *mut c_void);
}
```

---

### `src-tauri/src/lib.rs` (MODIFY — register new commands + maybe plugins)

**Analog:** itself. Add new commands to `tauri::generate_handler![...]` (L34-52) following the existing comma list. If adding dialog/fs/updater, register in the builder chain (L24-33) using the RESEARCH.md Pattern 1 shape, and add modules to the `mod` list (L7-20: add `sounds`, `media_pause`, `audio_decode`, `data_mgmt`). Setup hooks (sounds/media-pause need no setup; updater plugin does).

---

### `src-tauri/tauri.conf.json` (MODIFY — window grows)

**Analog:** itself, window `main` (L14-25). D-05/UI-SPEC: width `480→880`, height `600→640`, `resizable false→true`. GOTCHA: requires app relaunch, not dev-reload.
```json
{ "label": "main", "width": 880, "height": 640, "resizable": true, ... }
```
Also update `open_main_window` fallback in `lib.rs` (L138-148: `inner_size(480.0, 600.0)` / `resizable(false)`) to match, or it diverges when the window is recreated.

---

### `src-tauri/capabilities/default.json` (MODIFY — grant new plugin perms)

**Analog:** itself. Single capability file, `windows: ["main","widget"]`. Existing perms include `global-shortcut:*`, `opener:default`, `shell:allow-open`. RESEARCH Pitfall 1: every new plugin command needs a permission entry here AND a rebuild (not dev-reload). If adding dialog/fs/updater: `dialog:allow-open`, `fs:allow-app-write-recursive`, `updater:default`.

## Shared Patterns

### Native macOS (objc2)
**Source:** `mic_permission.rs` (L17-58), `notch.rs` (L16-100), `escape.rs` (L13-45), `commands.rs::post_cmd_v` (L605-633).
**Apply to:** `sounds.rs`, `media_pause.rs`.
**Pattern:** `#[cfg(target_os = "macos")]` impl + `#[cfg(not)]` no-op twin; `objc2 0.6` `msg_send!` + `AnyClass::get(c"...")`; CoreGraphics/AppKit via `extern "C"` link blocks with manual `CFRelease`; `MainThreadMarker::new()` guard for any AppKit call. Never use cocoa/core-foundation crates — not in the dep tree.

### Settings persistence (serde struct, NOT plugin-store)
**Source:** `src-tauri/src/settings.rs` (L7-85).
**Apply to:** all new flags (sounds×3, pause_media, dictionary).
**Pattern:** add field with `#[serde(default)]` (or `#[serde(default = "fn")]`), mirror in `impl Default`, persist through existing `get_settings`/`save_settings`. CACHE is updated on save — never write `settings.json` directly.

### Frontend persistence (debounced invoke)
**Source:** `+page.svelte::schedSave` (L113-119).
**Apply to:** every new toggle/input/select in the section components.
**Pattern:** `bind:value`/`bind:checked` → `onchange/oninput={() => schedSave()}`; `schedSave(true)` only when the hotkey changes (triggers Rust `re_register`).

### Tauri command convention
**Source:** `commands.rs` (L67-96, L522-573).
**Apply to:** file-transcription, data-mgmt, updater-check, and any new command.
**Pattern:** `#[tauri::command] pub (async) fn name<R: Runtime>(app: AppHandle<R>, ...) -> Result<T, String>`; errors via `.map_err(|e| e.to_string())`; progress/state via `app.emit("kebab-event", payload)`; register in `lib.rs` `generate_handler!`.

### Download-with-progress (reqwest stream)
**Source:** `commands.rs::download_model` (L536-572).
**Apply to:** model downloads (extend), updater check (adapt).
**Pattern:** `reqwest::Client` stream → `bytes_stream()` loop → `file.write_all` + `app.emit("download-progress", json!{...})` → `tokio::fs::rename(tmp, dest)` + `app.emit("download-complete", id)`. Writes to a `.tmp` then renames (atomic).

### Permission panel (poll + reveal)
**Source:** `+page.svelte` perm rows (L204-228, 626-646) + `commands.rs` perm commands (L32-63).
**Apply to:** `PermissionRow.svelte` / Ajustes permissions section (D-09).
**Pattern:** `check_mic_permission` / `check_accessibility_permission` polled every 3s; `open_microphone_settings` / `open_accessibility_settings` open System Settings via `open x-apple.systempreferences:...`. Granted = `--blue` dot + "Concedido"; not granted = muted dot + "Abrir ajustes" link.

## No Analog Found

| File | Role | Data Flow | Reason | Planner action |
|------|------|-----------|--------|----------------|
| `src-tauri/src/audio_decode.rs` | service | file-I/O + transform | No audio-file decoder exists (current capture is live cpal mic only). | Use RESEARCH.md `symphonia` (decode → downmix mono → f32), then feed existing `transcription::resample` to 16kHz. Run on `spawn_blocking`. RESEARCH says rubato preferred over the linear `resample()`; planner decides (existing `resample` is linear — acceptable but RESEARCH Pitfall 3 warns on WER for arbitrary rates). |
| `media_pause.rs` event construction | utility | event-driven | No `NSSystemDefined` event built anywhere (existing CGEvent is keyboard-only). | Borrow FFI scaffolding from `post_cmd_v`; construct the systemDefined media-key event per RESEARCH.md Pitfall 4 (verify objc2 surface). |
| updater integration | controller | request-response | No updater plugin installed; no release/signing infra confirmed (RESEARCH Open Q3, A6). | If infra not ready, ship UI + a check command that reports "Estás al día / Hay versión X" (D-14 fallback). |

## Metadata

**Analog search scope:** `src/routes/`, `src-tauri/src/`, `src-tauri/capabilities/`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`.
**Files scanned:** 14 (full reads): `+page.svelte`, `settings.rs`, `commands.rs`, `history.rs`, `lib.rs`, `notch.rs`, `escape.rs`, `transcription.rs`, `backend.rs`, `mic_permission.rs`, `shortcut.rs`, `tauri.conf.json`, `Cargo.toml`, `capabilities/default.json`.
**Pattern extraction date:** 2026-06-14
