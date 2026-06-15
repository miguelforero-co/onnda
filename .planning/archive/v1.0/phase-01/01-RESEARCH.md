# Phase 01: Rediseño del panel de configuración estilo WhisprFlow - Research

**Researched:** 2026-06-14
**Domain:** Tauri 2 (Rust + WebView) desktop app — settings panel, OS integration, audio, models
**Confidence:** MEDIUM-HIGH (core Tauri plugins HIGH; some crate versions ASSUMED — verify against crates.io before locking)

## Summary

This phase rebuilds the configuration panel for a Tauri 2 macOS dictation app (Voz Local) to match the feature surface of WhisprFlow: feedback sounds, pause-media-while-recording, launch-at-login, an in-app global-hotkey recorder, an expanded Whisper model picker with download management, in-app auto-update, data/privacy management, drag-and-drop file transcription, dictionary persistence/migration, and a resizable/repositionable settings window.

Most capabilities map directly to first-party Tauri plugins (`tauri-plugin-autostart`, `tauri-plugin-global-shortcut`, `tauri-plugin-updater`, `tauri-plugin-store`, `tauri-plugin-dialog`, `tauri-plugin-opener`). The non-plugin work is concentrated in three areas: (1) audio decoding for arbitrary dropped files (use `symphonia` to decode → resample to 16 kHz mono f32 for Whisper), (2) macOS "pause media" which has no clean public API and requires either simulating the Play/Pause media key or using `MPNowPlayingInfoCenter`/`MediaRemote` (private) — recommend the media-key simulation path, and (3) a hotkey recorder UI, which is a frontend capture problem feeding into `global-shortcut` registration.

**Primary recommendation:** Lean on first-party Tauri 2 plugins for autostart, global-shortcut, updater, store, and dialog. Build only three custom pieces: symphonia-based file decode→resample, a frontend keycombo recorder that validates against `global-shortcut` accelerator syntax, and a media-key-press shim for pause-media. Verify every crate/plugin version with `cargo search` / crates.io before writing the plan — versions below are ASSUMED from training and the Tauri 2 plugin line moves fast.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Feedback sounds | Frontend (WebView) | Rust (asset bundling) | HTMLAudioElement is simplest; no native dep |
| Pause media while recording | Rust (native macOS) | — | Requires OS-level media-key / MediaRemote access |
| Launch at login (autostart) | Rust (plugin) | — | OS login-items registration is native |
| Hotkey recorder | Frontend (capture) | Rust (registration) | Key capture is a DOM event; registration is `global-shortcut` |
| Expanded Whisper models + download | Rust (download/verify) | Frontend (picker UI) | File I/O, hashing, model store live in Rust |
| Auto-updater | Rust (plugin) | Frontend (prompt UI) | Signature verify + install is native |
| Data management (clear/export) | Rust (fs + store) | Frontend (UI) | Filesystem and store ownership is Rust |
| File transcription (drag-drop) | Rust (decode + ASR) | Frontend (drop zone) | Audio decode + Whisper inference is Rust |
| Dictionary migration | Rust (store/fs) | — | Persisted data format owned by Rust |
| Window resize/position | Rust (window cfg) | Frontend (trigger) | WebviewWindow geometry is Rust API |

## Standard Stack

### Core (Tauri 2 first-party plugins)
| Plugin / Crate | Version (ASSUMED — verify) | Purpose | Why Standard |
|---------------|---------|---------|--------------|
| `tauri-plugin-autostart` | `2.x` | Launch at login | First-party, cross-platform login-items |
| `tauri-plugin-global-shortcut` | `2.x` | Global hotkey registration | First-party; replaces removed core API |
| `tauri-plugin-updater` | `2.x` | In-app auto-update | First-party; signed update verification |
| `tauri-plugin-store` | `2.x` | Settings/dictionary persistence (JSON) | First-party key-value store |
| `tauri-plugin-dialog` | `2.x` | Open-file / save dialogs | First-party native dialogs |
| `tauri-plugin-opener` | `2.x` | Reveal in Finder / open URLs | First-party (replaces deprecated `shell.open`) |
| `tauri-plugin-fs` | `2.x` | Read/write app data, export | First-party scoped filesystem |

### Supporting (Rust crates for custom work)
| Crate | Version (ASSUMED — verify) | Purpose | When to Use |
|-------|---------|---------|-------------|
| `symphonia` | `0.5.x` | Decode dropped audio/video → PCM | File transcription; pure-Rust, no ffmpeg |
| `rubato` | `0.15.x`–`0.16.x` | Resample arbitrary rate → 16 kHz | After symphonia decode, before Whisper |
| `sha2` | `0.10.x` | Verify model download integrity | Model manager |
| `reqwest` | `0.12.x` | Stream model downloads w/ progress | Model manager (with `stream` feature) |
| `objc2` / `objc2-app-kit` | `0.5.x`+ | Media-key simulation / native calls | Pause-media shim |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `symphonia` | bind to `ffmpeg` | ffmpeg handles every container but adds a heavy native dep + licensing/bundling pain; symphonia is pure Rust and covers wav/mp3/flac/aac/ogg/m4a |
| `rubato` | linear interpolation by hand | Hand-rolled resampling aliases badly and hurts WER; rubato (sinc/FFT) is the standard |
| HTMLAudioElement sounds | `rodio` (Rust playback) | rodio is more robust for system-level audio but overkill for two short UI cues; only switch if WebView audio is unreliable |
| media-key simulation | `MediaRemote.framework` | MediaRemote is private API → App Store rejection risk and breaks across macOS versions; media-key press is more portable |

**Installation (verify each version first):**
```bash
# Rust side
cargo add tauri-plugin-autostart tauri-plugin-global-shortcut tauri-plugin-updater \
  tauri-plugin-store tauri-plugin-dialog tauri-plugin-opener tauri-plugin-fs
cargo add symphonia --features all
cargo add rubato sha2
cargo add reqwest --features stream
# Frontend side (JS guest bindings)
npm i @tauri-apps/plugin-autostart @tauri-apps/plugin-global-shortcut \
  @tauri-apps/plugin-updater @tauri-apps/plugin-store @tauri-apps/plugin-dialog \
  @tauri-apps/plugin-opener @tauri-apps/plugin-fs
```

**Version verification (DO THIS BEFORE PLANNING):**
```bash
cargo search tauri-plugin-updater
cargo search symphonia
cargo search rubato
npm view @tauri-apps/plugin-store version
```
Tauri 2 plugins are versioned independently and the `2.x` line ships frequently. Treat all versions above as `[ASSUMED]` until confirmed.

## Architecture Patterns

### System Architecture Diagram

```
                ┌─────────────────────────────────────────────┐
                │            Settings WebView (UI)             │
                │  hotkey recorder · model picker · toggles    │
                │  drop zone · data mgmt · update prompt       │
                └───────────────┬─────────────────────────────┘
                                │ invoke() / plugin JS bindings
                                ▼
        ┌───────────────────────────────────────────────────────────┐
        │                    Rust core (commands)                    │
        │                                                            │
        │  settings ──▶ tauri-plugin-store (settings.json)           │
        │  hotkey   ──▶ tauri-plugin-global-shortcut (register)      │
        │  autostart──▶ tauri-plugin-autostart (login items)         │
        │  update   ──▶ tauri-plugin-updater (check/download/install)│
        │  data mgmt──▶ tauri-plugin-fs + store (clear/export)       │
        │  models   ──▶ reqwest(stream)+sha2 ─▶ app_data_dir/models/ │
        │  pause    ──▶ objc2 media-key press (CGEvent)              │
        │                                                            │
        │  file drop ──▶ symphonia(decode) ─▶ rubato(→16kHz mono)    │
        │                        └────────────▶ Whisper engine       │
        └───────────────────────────────────────────────────────────┘
                                │
                                ▼
        macOS: NSScreen · Login Items · CGEvent · Filesystem · Updater
```

### Recommended Project Structure (additive to existing src-tauri)
```
src-tauri/src/
├── settings.rs       # store-backed config struct + get/set commands
├── hotkey.rs         # global-shortcut register/unregister + validation
├── models/
│   ├── catalog.rs    # static list of Whisper models (name,url,sha256,size)
│   └── download.rs   # streamed download + sha2 verify + progress events
├── audio/
│   ├── decode.rs     # symphonia file → f32 PCM
│   └── resample.rs   # rubato → 16kHz mono
├── media_pause.rs    # CGEvent media-key press (objc2)
└── data_mgmt.rs      # clear history / export / open data dir
```

### Pattern 1: Plugin registration (Tauri 2 builder)
**What:** All plugins register in the `tauri::Builder` chain; autostart and global-shortcut take config closures.
**When:** App setup in `lib.rs`/`main.rs`.
```rust
// Source: tauri.app v2 plugin docs (CITED)
tauri::Builder::default()
    .plugin(tauri_plugin_store::Builder::default().build())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_autostart::init(
        tauri_plugin_autostart::MacosLauncher::LaunchAgent,
        None, // launch args
    ))
    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
    .plugin(tauri_plugin_updater::Builder::new().build())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

### Pattern 2: Capabilities/permissions (Tauri 2 requirement)
**What:** Tauri 2 requires explicit permissions in `src-tauri/capabilities/*.json` for each plugin command the frontend calls. Missing permission = silent denial at runtime.
**When:** Every plugin you add.
```json
// Source: tauri.app v2 security/capabilities (CITED)
{
  "permissions": [
    "store:default",
    "dialog:allow-open",
    "fs:allow-app-write-recursive",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "autostart:allow-enable",
    "autostart:allow-disable",
    "updater:default"
  ]
}
```

### Anti-Patterns to Avoid
- **Registering global shortcuts without an unregister-all on rebind:** leaves orphaned handlers. Always `unregister` the old combo before registering the new one.
- **Decoding audio on the main/UI thread:** symphonia decode of a long file blocks. Run in `tauri::async_runtime::spawn_blocking` and emit progress events.
- **Hand-rolling resampling:** causes aliasing → worse transcription. Use rubato.
- **Using private MediaRemote for pause:** App Store rejection + breaks on OS updates.
- **Storing model files in the app bundle:** they're large and bundle is read-only; download to `app_data_dir()/models/`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Launch at login | NSLoginItems / plist writing | `tauri-plugin-autostart` | Handles LaunchAgent + Windows/Linux parity |
| Global hotkey | Raw Carbon/CGEvent tap | `tauri-plugin-global-shortcut` | Accelerator parsing, conflict handling, lifecycle |
| Signed auto-update | Custom download+swap | `tauri-plugin-updater` | minisign signature verification, atomic install |
| Audio container decode | Manual WAV/MP3 parsing | `symphonia` | Covers mp3/flac/aac/m4a/ogg/wav edge cases |
| Resampling to 16kHz | Linear interp | `rubato` | Anti-aliased sinc; protects WER |
| Settings persistence | Custom JSON read/write | `tauri-plugin-store` | Atomic writes, change events, cross-platform path |
| Open in Finder / URLs | `Command::new("open")` | `tauri-plugin-opener` | Scoped, cross-platform, replaces deprecated shell |

**Key insight:** The only genuinely custom logic in this phase is media-pause (no clean API) and the hotkey-recorder UX glue. Everything else is plugin wiring.

## Runtime State Inventory

This phase introduces new persisted state and migrates existing state. Relevant because of the dictionary migration item.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | Existing dictionary + settings (current format — likely a JSON or custom file in app data dir). New `tauri-plugin-store` `settings.json`. | Data migration: read legacy file, transform, write into store; keep a one-shot migration guard (e.g., `schema_version` key) |
| Live service config | None | None — no external service holds state |
| OS-registered state | Login item (created by autostart plugin); registered global shortcut (held by OS while app runs) | Re-register shortcut on settings change; toggle login item via plugin |
| Secrets/env vars | `FAL_KEY` / `COMPOSIO_API_KEY` (image gen, unrelated to this phase) | None — not touched by this phase |
| Build artifacts | Updater requires a signing keypair + `tauri.conf.json` `updater` pubkey + an update endpoint/manifest | Generate keypair (`tauri signer generate`), add pubkey to config, host manifest |

**Dictionary migration specifics:** Whisprflow-style apps keep a custom-words dictionary. If the current app stores it in a bespoke file, write a migration that (1) detects legacy file, (2) loads entries, (3) writes into the new store under a versioned key, (4) marks migration done so it never re-runs and never clobbers user edits. Verify the current storage location/format from the existing codebase before planning the migration (NOT yet confirmed in this research — see Open Questions).

## Common Pitfalls

### Pitfall 1: Tauri 2 capabilities not granted
**What goes wrong:** Plugin JS calls resolve to permission errors or silently no-op.
**Why:** Tauri 2 denies by default; every command needs a capability entry.
**How to avoid:** Add each plugin's permission to `capabilities/default.json` and rebuild (not just dev-reload).
**Warning signs:** "not allowed" / "forbidden" errors in console; toggles that don't persist.

### Pitfall 2: Updater signature/pubkey mismatch
**What goes wrong:** Update check finds a release but install fails with signature error.
**Why:** The `pubkey` in `tauri.conf.json` must match the private key used to sign the bundle; mismatched or missing key blocks install.
**How to avoid:** Generate keypair once, store private key as a CI secret (`TAURI_SIGNING_PRIVATE_KEY`), embed pubkey in config. Test against a staging manifest.
**Warning signs:** "signature verification failed".

### Pitfall 3: Whisper expects 16 kHz mono f32
**What goes wrong:** Dropped 44.1 kHz stereo file transcribes as garbage.
**Why:** Whisper models require 16 kHz mono PCM; skipping resample/downmix produces wrong results.
**How to avoid:** symphonia decode → downmix to mono (average channels) → rubato resample to 16 kHz → feed f32.
**Warning signs:** transcription length wrong, gibberish output.

### Pitfall 4: macOS pause-media has no public API
**What goes wrong:** Trying to "pause Spotify/Music" cleanly fails — there's no sanctioned cross-app pause.
**Why:** Media control is owned by the now-playing app; MediaRemote is private.
**How to avoid:** Simulate the F8/Play-Pause media key via a `CGEvent` of type `NSSystemDefined` (subtype 8, key `NX_KEYTYPE_PLAY`), down+up. This is the portable approach. Track whether *you* paused so you can resume (send the same key again) after recording.
**Warning signs:** App Store review rejection (if using MediaRemote); no effect on some media apps.

### Pitfall 5: Global shortcut leaks on rebind
**What goes wrong:** Old hotkey still triggers after user changes it.
**Why:** Register without unregister accumulates handlers.
**How to avoid:** On rebind: `unregister(old)` → validate new accelerator → `register(new)`; persist only after successful register.

## Code Examples

### Streamed model download with progress + sha256 verify
```rust
// Source: reqwest stream + sha2 standard pattern (ASSUMED shape — verify API)
use futures_util::StreamExt;
use sha2::{Digest, Sha256};

async fn download_model(app: &tauri::AppHandle, url: &str, expected_sha: &str, dest: &std::path::Path)
    -> Result<(), String> {
    let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let total = resp.content_length().unwrap_or(0);
    let mut stream = resp.bytes_stream();
    let mut file = tokio::fs::File::create(dest).await.map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut done = 0u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        hasher.update(&chunk);
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await.map_err(|e| e.to_string())?;
        done += chunk.len() as u64;
        let _ = app.emit("model-download-progress", (done, total));
    }
    let got = format!("{:x}", hasher.finalize());
    if got != expected_sha { return Err("sha256 mismatch".into()); }
    Ok(())
}
```

### Frontend hotkey recorder → register
```ts
// Source: global-shortcut JS binding + DOM capture (ASSUMED shape)
import { register, unregister } from '@tauri-apps/plugin-global-shortcut';

function captureCombo(e: KeyboardEvent): string | null {
  e.preventDefault();
  const mods: string[] = [];
  if (e.metaKey) mods.push('CommandOrControl'); // macOS Cmd
  if (e.altKey) mods.push('Alt');
  if (e.shiftKey) mods.push('Shift');
  if (e.ctrlKey) mods.push('Control');
  const key = e.key.length === 1 ? e.key.toUpperCase() : e.code.replace('Key', '');
  if (['Meta','Alt','Shift','Control'].includes(key)) return null; // modifier-only
  return [...mods, key].join('+');
}

async function rebind(oldAccel: string, newAccel: string, onTrigger: () => void) {
  try { await unregister(oldAccel); } catch {}
  await register(newAccel, onTrigger); // throws if invalid/in-use → show error, keep old
}
```

### Media-key pause shim (concept)
```rust
// Source: CGEvent NSSystemDefined media-key pattern (ASSUMED — verify objc2 API surface)
// Send NX_KEYTYPE_PLAY (key 16, subtype 8) down then up via a CGEvent of type 14.
// Wrap in a helper; track `i_paused` bool so recording-stop can resume.
fn send_play_pause() { /* build NSEvent.otherEventWithType systemDefined, subtype 8 */ }
```

### Resize/reposition settings window
```rust
// Source: tauri v2 WebviewWindow API (CITED)
use tauri::{Manager, LogicalSize, LogicalPosition};
if let Some(win) = app.get_webview_window("settings") {
    win.set_size(LogicalSize::new(820.0, 600.0)).ok();
    win.set_resizable(true).ok();
    win.center().ok();
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri 1 core global-shortcut API | `tauri-plugin-global-shortcut` | Tauri 2 (2024) | Must use plugin; core API removed |
| `shell.open` for files/URLs | `tauri-plugin-opener` | Tauri 2.x | shell-open deprecated/split out |
| Allowlist in tauri.conf | Capabilities files | Tauri 2 (2024) | Per-window permission JSON required |
| ffmpeg bindings for decode | `symphonia` pure-Rust | ~2023+ | No native ffmpeg dep / licensing |

**Deprecated/outdated:**
- Tauri 1 `tauri.conf.json` allowlist — replaced by capabilities.
- `@tauri-apps/api/shell` open — use `@tauri-apps/plugin-opener`.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Crate/plugin versions (autostart, global-shortcut, updater, store, symphonia 0.5, rubato 0.15, reqwest 0.12) | Standard Stack | Plan pins wrong version → build break; LOW risk, easy to fix at install |
| A2 | App currently uses Tauri 2 (not 1) | All | If Tauri 1, plugin API and capabilities advice is wrong; MEDIUM — verify in Cargo.toml |
| A3 | Current dictionary/settings storage format & location | Runtime State / Migration | Migration could clobber or miss data; MEDIUM-HIGH — must inspect codebase |
| A4 | Whisper engine wants 16 kHz mono f32 (whisper.cpp/Parakeet convention) | Pitfalls | Wrong sample rate → garbage transcription; LOW — well-established |
| A5 | Pause-media via CGEvent media-key works for target media apps | Pause-media | Some apps ignore media keys; MEDIUM — test against Music/Spotify/browser |
| A6 | Updater endpoint/manifest will be self-hosted (e.g., GitHub releases) | Updater | Affects config + CI; MEDIUM — depends on distribution choice (not decided) |
| A7 | objc2 is the chosen FFI crate (vs `cocoa`/`core-foundation`) | Pause-media | API surface differs; LOW — pick whatever the project already uses |

## Open Questions

1. **Is the app on Tauri 1 or Tauri 2?**
   - Known: Memory notes mention a "Tauri+objc2" plan for the notch widget.
   - Unclear: Exact Tauri major version and existing plugin set.
   - Recommendation: `grep tauri src-tauri/Cargo.toml` before planning; all plugin guidance assumes v2.

2. **Where/how is the current dictionary stored today?**
   - Known: A dictionary feature exists (migration is in scope).
   - Unclear: File path + format (JSON? sqlite? custom?).
   - Recommendation: Inspect existing settings/dictionary code; the migration task depends entirely on this.

3. **How will updates be distributed (update manifest host)?**
   - Unclear: GitHub Releases vs self-hosted endpoint.
   - Recommendation: Decide before configuring `updater` in `tauri.conf.json`; affects CI signing setup.

4. **Which Whisper backend is in use (whisper.cpp Metal vs FluidAudio/Parakeet ANE)?**
   - Known: Memory research compared both; current binary engine unconfirmed for this build.
   - Recommendation: Confirm — it determines model catalog format and the exact PCM feed contract for file transcription.

5. **Does pause-media need to *resume* after recording, and for which apps?**
   - Recommendation: Confirm UX expectation (pause-only vs pause+resume) — resume requires tracking that we initiated the pause.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain / cargo | All Rust crates | ✓ (project builds) | — | — |
| Tauri CLI (`tauri signer`) | Updater keypair gen | ? verify | — | Generate via cargo install tauri-cli |
| Node ≥ 23 | Frontend + genimg script (per CLAUDE.md) | ✓ | ≥23 | — |
| macOS frameworks (AppKit/CoreGraphics) | Pause-media, window | ✓ (macOS) | system | — |
| Update manifest host | Updater | ✗ (not chosen) | — | GitHub Releases |

**Missing/blocking:** Update manifest host + signing keypair must be set up before the updater item ships (otherwise build-time config is incomplete).

## Validation Architecture

> Include unless `workflow.nyquist_validation` is explicitly false. Confirm test setup exists; Rust side likely `cargo test`, frontend likely none yet.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `cargo test` (Rust units); frontend — verify (likely none / Wave 0) |
| Config file | none for frontend — see Wave 0 |
| Quick run command | `cargo test --manifest-path src-tauri/Cargo.toml` |
| Full suite command | `cargo test --manifest-path src-tauri/Cargo.toml --all-features` |

### Phase Requirements → Test Map
| Behavior | Test Type | Automated Command | File Exists? |
|----------|-----------|-------------------|-------------|
| Hotkey accelerator parsing/validation | unit | `cargo test hotkey` | ❌ Wave 0 |
| Model sha256 verify rejects bad download | unit | `cargo test models::download` | ❌ Wave 0 |
| Audio decode→16kHz mono produces correct length | unit | `cargo test audio` | ❌ Wave 0 |
| Dictionary migration is idempotent | unit | `cargo test migration` | ❌ Wave 0 |
| Settings round-trip through store | unit | `cargo test settings` | ❌ Wave 0 |
| Autostart toggle / updater / pause-media | manual | manual checklist | manual-only (OS side effects) |

### Sampling Rate
- **Per task commit:** `cargo test --manifest-path src-tauri/Cargo.toml <module>`
- **Per wave merge:** full `cargo test`
- **Phase gate:** full suite green + manual OS-integration checklist (login item, hotkey fires, update installs, media pauses)

### Wave 0 Gaps
- [ ] `src-tauri/src/audio/decode.rs` + test fixture (small wav/mp3) — covers file transcription
- [ ] `src-tauri/src/models/download.rs` test (hash-mismatch case)
- [ ] `src-tauri/src/hotkey.rs` accelerator validation tests
- [ ] migration idempotency test with a legacy-format fixture
- [ ] Decide/confirm frontend test framework (or accept manual-only for UI)

## Project Constraints (from CLAUDE.md)

- **Image generation routing (global CLAUDE.md):** Not directly relevant to this phase, BUT any UI placeholder icons in the redesigned settings panel MUST be hand-written SVG in code — never call an image model for icons/logos/placeholders.
- **API keys only via env vars** (`FAL_KEY` / `COMPOSIO_API_KEY`) — never in code/commits.
- **Feedback prefs (from memory):** Iterate in dev, don't compile a release per change; test before declaring done; respond in Spanish. These affect execution workflow, not architecture.

## Sources

### Primary (HIGH confidence)
- tauri.app v2 plugin docs — autostart, global-shortcut, updater, store, dialog, opener, fs (CITED)
- tauri.app v2 capabilities/security model (CITED)
- symphonia / rubato crate docs — pure-Rust decode + resample (CITED)

### Secondary (MEDIUM confidence)
- whisper.cpp / Parakeet 16 kHz mono f32 input convention (project memory + general STT knowledge)
- CGEvent / NX_KEYTYPE_PLAY media-key simulation pattern (community/macOS dev sources)

### Tertiary (LOW confidence — validate)
- Exact crate versions (ASSUMED from training; verify on crates.io/npm)
- objc2 API surface for media-key event construction (verify against current objc2)

## Metadata

**Confidence breakdown:**
- Standard stack (which plugins/crates): HIGH — first-party Tauri 2 plugins are the documented standard.
- Versions: LOW — must verify before pinning.
- Architecture/patterns: HIGH for plugin wiring; MEDIUM for pause-media (no public API).
- Migration: LOW until current storage format inspected.
- Pitfalls: MEDIUM-HIGH — well-known Tauri 2 + Whisper gotchas.

**Research date:** 2026-06-14
**Valid until:** ~2026-07-14 (Tauri 2 plugin line moves fast — re-verify versions if planning later)
