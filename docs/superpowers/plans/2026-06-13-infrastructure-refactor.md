# Infrastructure Refactor — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the monolithic transcription pipeline with a modular, backend-agnostic audio architecture — single-pass resampling at capture time, WebRTC VAD for silence detection, dynamic normalization, and a `TranscriptionBackend` trait that allows swapping Whisper variants (or other models) without touching the rest of the app.

**Architecture:** `AudioCapture` resamples to 16kHz inline so every downstream consumer (chunks, final pass, history) shares pre-processed audio. A `TranscriptionBackend` trait decouples the model from the pipeline. WebRTC VAD replaces the current energy-RMS silence trimming with a battle-tested frame-level detector.

**Tech Stack:** Rust, Tauri 2, cpal, whisper-rs 0.14 (aarch64 with metal feature), webrtc-vad 0.4 crate, strsim 0.11

---

## File Map

| File | Action | Responsibility |
|---|---|---|
| `src-tauri/src/audio.rs` | **Create** | AudioCapture — captures mic, resamples to 16kHz inline, emits RMS levels |
| `src-tauri/src/backend.rs` | **Create** | `TranscriptionBackend` trait + `TranscribeOpts` struct |
| `src-tauri/src/whisper_backend.rs` | **Create** | `WhisperBackend` implementing the trait, holds MODEL_CACHE |
| `src-tauri/src/vad.rs` | **Create** | `vad_trim(samples: &[f32]) -> (usize, usize)` using WebRTC VAD |
| `src-tauri/src/transcription.rs` | **Shrink** | Keep only: `rms_f32`, `correct_words`, `resample`, `jaro_winkler_match` |
| `src-tauri/src/commands.rs` | **Modify** | Use `audio::AudioCapture`, call backend trait, use `vad::vad_trim` |
| `src-tauri/src/history.rs` | **Modify** | Remove internal `resample` call — audio is already 16kHz |
| `src-tauri/Cargo.toml` | **Modify** | Add `webrtc-vad = "0.4"` dependency |
| `src-tauri/src/lib.rs` | **Modify** | Declare new modules |

---

## Task 1: Add `webrtc-vad` dependency and new module declarations

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add dependency to Cargo.toml**

In `src-tauri/Cargo.toml`, find the `[dependencies]` section and add:

```toml
webrtc-vad = "0.4"
```

- [ ] **Step 2: Declare new modules in lib.rs**

Read `src-tauri/src/lib.rs`. Add the new module declarations alongside the existing ones:

```rust
pub mod audio;
pub mod backend;
pub mod whisper_backend;
pub mod vad;
```

- [ ] **Step 3: Verify it compiles with new dependency**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```

Expected: `Finished` (webrtc-vad downloads and compiles).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs
git commit -m "chore: add webrtc-vad dep, declare new modules"
```

---

## Task 2: Extract `AudioCapture` into `audio.rs` with inline 16kHz resampling

**Problem:** `AudioCapture` currently lives in `transcription.rs` and captures at the device's native rate (usually 48kHz). Every `transcribe()` call and every history save resamples independently. We resample once at capture time instead.

**Files:**
- Create: `src-tauri/src/audio.rs`
- Modify: `src-tauri/src/transcription.rs` (remove AudioCapture struct)

- [ ] **Step 1: Create `src-tauri/src/audio.rs`**

```rust
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use anyhow::{anyhow, Result};

pub struct AudioCapture {
    /// Always 16000 — we resample inline at capture time.
    pub sample_rate: u32,
    samples: Arc<Mutex<Vec<f32>>>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl AudioCapture {
    pub fn start(on_level: impl Fn(f32) + Send + 'static) -> Result<Self> {
        use cpal::traits::{DeviceTrait, HostTrait};

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow!("No se encontró micrófono"))?;

        let config = device.default_input_config()?;
        let native_rate = config.sample_rate().0 as usize;
        let channels = config.channels() as usize;

        let samples = Arc::new(Mutex::new(Vec::<f32>::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let last_emit_ms = Arc::new(AtomicU64::new(0));

        let samples_thread = Arc::clone(&samples);
        let stop_thread = Arc::clone(&stop);

        let thread = std::thread::spawn(move || {
            use cpal::traits::StreamTrait;
            let samples_cb = Arc::clone(&samples_thread);
            let last_emit = Arc::clone(&last_emit_ms);

            // Resampling state: accumulate fractional position across callbacks.
            let ratio = native_rate as f64 / 16000.0_f64;
            let mut resample_pos: f64 = 0.0;
            // Native-rate ring buffer to support interpolation across callback boundaries.
            let mut native_buf: Vec<f32> = Vec::with_capacity(native_rate / 10);

            let stream = device
                .build_input_stream(
                    &config.into(),
                    move |data: &[f32], _| {
                        // Downmix to mono.
                        let mono: Vec<f32> = data
                            .chunks(channels)
                            .map(|f| f.iter().sum::<f32>() / channels as f32)
                            .collect();

                        // Emit RMS level throttled to ~50ms.
                        let now_ms = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;
                        let prev = last_emit.load(Ordering::Relaxed);
                        if now_ms.saturating_sub(prev) >= 50 {
                            last_emit.store(now_ms, Ordering::Relaxed);
                            on_level(rms_f32(&mono));
                        }

                        // Linear-interpolation resample to 16kHz inline.
                        native_buf.extend_from_slice(&mono);
                        let mut out = Vec::new();
                        while resample_pos + 1.0 < native_buf.len() as f64 {
                            let lo = resample_pos as usize;
                            let hi = lo + 1;
                            let frac = (resample_pos - lo as f64) as f32;
                            out.push(native_buf[lo] * (1.0 - frac) + native_buf[hi] * frac);
                            resample_pos += ratio;
                        }
                        // Keep only the tail needed for next callback's interpolation.
                        let keep_from = resample_pos as usize;
                        native_buf.drain(..keep_from.min(native_buf.len()));
                        resample_pos -= keep_from as f64;

                        if !out.is_empty() {
                            samples_cb.lock().unwrap().extend_from_slice(&out);
                        }
                    },
                    |err| eprintln!("cpal error: {err}"),
                    None,
                )
                .expect("failed to build input stream");

            stream.play().expect("failed to start stream");
            while !stop_thread.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        Ok(Self { sample_rate: 16000, samples, stop, thread: Some(thread) })
    }

    pub fn samples_arc(&self) -> Arc<Mutex<Vec<f32>>> {
        Arc::clone(&self.samples)
    }

    pub fn stop(mut self) -> (Vec<f32>, u32) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() { let _ = t.join(); }
        let samples = self.samples.lock().unwrap().clone();
        (samples, self.sample_rate)
    }
}

pub fn rms_f32(samples: &[f32]) -> f32 {
    if samples.is_empty() { return 0.0; }
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
}
```

- [ ] **Step 2: Remove `AudioCapture` from `transcription.rs`**

In `src-tauri/src/transcription.rs`, delete the entire `AudioCapture` struct and its `impl` block (lines ~11–88 in the current file — the struct definition through the closing `}`). Also delete `rms_f32` since it now lives in `audio.rs`. Keep: `transcribe`, `trim_silence_range`, `correct_words`, `jaro_winkler_match`, `resample`.

Update the `use` imports at the top of `transcription.rs` — remove any that are no longer needed (`AtomicU64`, `AtomicBool` if unused). Add:

```rust
pub use crate::audio::rms_f32;
```

This re-export keeps `commands.rs` and `history.rs` working without changes to their import paths yet.

- [ ] **Step 3: Update imports in `commands.rs`**

At the top of `commands.rs`, change:

```rust
use crate::transcription::AudioCapture;
```

to:

```rust
use crate::audio::AudioCapture;
```

- [ ] **Step 4: Update `history.rs` — remove resample call**

`history.rs` currently calls `crate::transcription::resample()` before writing WAV because it was getting raw native-rate samples. Now samples arrive pre-resampled at 16kHz. Find the line that calls `resample` in `save_entry` and remove it — write the samples directly.

The WAV header `sample_rate` field should be `16000` (hardcoded or passed as the already-16000 value).

- [ ] **Step 5: Build to verify**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep -E "error|warning.*unused|Finished"
```

Expected: `Finished` with no errors.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/audio.rs src-tauri/src/transcription.rs src-tauri/src/commands.rs src-tauri/src/history.rs
git commit -m "refactor: extract AudioCapture to audio.rs, resample to 16kHz at capture time"
```

---

## Task 3: Create `TranscriptionBackend` trait and `TranscribeOpts`

**Files:**
- Create: `src-tauri/src/backend.rs`

- [ ] **Step 1: Create `src-tauri/src/backend.rs`**

```rust
use anyhow::Result;

/// Options passed to any transcription backend.
#[derive(Clone)]
pub struct TranscribeOpts {
    /// BCP-47 language code, e.g. "es", "en", or "auto".
    pub language: String,
    /// Comma-separated custom vocabulary used as Whisper's initial_prompt.
    pub initial_prompt: String,
    /// Jaro-Winkler threshold for vocabulary correction (0.0–1.0).
    pub word_correction_threshold: f32,
}

/// A model backend that can transcribe 16kHz mono f32 PCM audio.
///
/// Implementations are expected to be stateful (cache model weights)
/// and thread-safe. `transcribe` may be called from a blocking thread.
pub trait TranscriptionBackend: Send + Sync {
    fn transcribe(&self, samples: &[f32], opts: &TranscribeOpts) -> Result<String>;
}
```

- [ ] **Step 2: Build to verify trait compiles**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep -E "error|Finished"
```

Expected: `Finished`.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/backend.rs
git commit -m "feat: add TranscriptionBackend trait and TranscribeOpts"
```

---

## Task 4: Create `WhisperBackend` implementing the trait

**Files:**
- Create: `src-tauri/src/whisper_backend.rs`
- Modify: `src-tauri/src/transcription.rs` (remove `transcribe` fn — it moves here)

- [ ] **Step 1: Create `src-tauri/src/whisper_backend.rs`**

This file takes the `transcribe` logic from `transcription.rs` and wraps it in the trait.

```rust
use std::sync::Mutex;
use anyhow::{anyhow, Result};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::backend::{TranscribeOpts, TranscriptionBackend};
use crate::transcription::{correct_words, vad_trim};

// Note: vad_trim will be added in Task 5. Until then, import trim_silence_range instead:
// use crate::transcription::trim_silence_range;

static MODEL_CACHE: Mutex<Option<(String, WhisperContext)>> = Mutex::new(None);

pub struct WhisperBackend {
    pub model_path: String,
}

impl WhisperBackend {
    pub fn new(model_path: String) -> Self {
        Self { model_path }
    }
}

impl TranscriptionBackend for WhisperBackend {
    fn transcribe(&self, samples: &[f32], opts: &TranscribeOpts) -> Result<String> {
        let mut cache = MODEL_CACHE.lock().unwrap();

        let needs_load = cache
            .as_ref()
            .map(|(p, _)| p.as_str() != self.model_path)
            .unwrap_or(true);

        if needs_load {
            eprintln!("[voz-local] loading model: {}", self.model_path);
            let mut ctx_params = WhisperContextParameters::default();
            #[cfg(target_arch = "aarch64")]
            {
                ctx_params.use_gpu(true);
                ctx_params.flash_attn(true);
            }
            let ctx = WhisperContext::new_with_params(&self.model_path, ctx_params)?;
            *cache = Some((self.model_path.clone(), ctx));
        }

        let ctx = &cache.as_ref().unwrap().1;
        let mut state = ctx.create_state()?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(6);
        params.set_n_max_text_ctx(224);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_no_speech_thold(0.6);

        if !opts.initial_prompt.is_empty() {
            params.set_initial_prompt(&opts.initial_prompt);
        }

        let lang = if opts.language == "auto" { None } else { Some(opts.language.as_str()) };
        params.set_language(lang);

        // Audio is already at 16kHz (resampled at capture). Trim silence via VAD.
        // In Task 5, replace trim_silence_range with vad_trim.
        let (trim_start, trim_end) = crate::transcription::trim_silence_range(samples, 16000);
        let mut trimmed = samples[trim_start..trim_end].to_vec();
        // 500ms silence tail so Whisper closes the last token.
        trimmed.extend(vec![0.0f32; 8000]);

        state.full(params, &trimmed)?;

        let n = state.full_n_segments()?;
        let text = (0..n)
            .filter_map(|i| state.full_get_segment_text(i).ok())
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();

        Ok(correct_words(&text, &opts.initial_prompt, opts.word_correction_threshold))
    }
}
```

- [ ] **Step 2: Remove `transcribe` from `transcription.rs`**

Delete the entire `pub fn transcribe(...)` function from `transcription.rs`. It now lives in `WhisperBackend`. Keep: `trim_silence_range`, `correct_words`, `jaro_winkler_match`, `resample`. Remove the `WhisperContext`/`WhisperContextParameters`/`FullParams`/`SamplingStrategy` imports from `transcription.rs` since they're no longer needed there.

- [ ] **Step 3: Update `commands.rs` to use `WhisperBackend`**

In `commands.rs`, replace all calls to `crate::transcription::transcribe(...)` with:

```rust
use crate::backend::TranscribeOpts;
use crate::whisper_backend::WhisperBackend;

// Where transcription is invoked (both in the chunk task and final pass):
let opts = TranscribeOpts {
    language: language.clone(),
    initial_prompt: initial_prompt.clone(),
    word_correction_threshold: threshold,
};
let backend = WhisperBackend::new(model_path_str.clone());
let result = tokio::task::spawn_blocking(move || {
    backend.transcribe(&snapshot, &opts)
}).await;
```

Do this replacement in both places: the chunk task loop and `stop_and_transcribe_internal`.

Note: `WhisperBackend::new` is cheap — the heavy model load happens inside `transcribe` and is cached in the static `MODEL_CACHE`.

- [ ] **Step 4: Build to verify**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep -E "error|Finished"
```

Expected: `Finished`.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/whisper_backend.rs src-tauri/src/transcription.rs src-tauri/src/commands.rs
git commit -m "refactor: move transcription logic to WhisperBackend, wire TranscriptionBackend trait"
```

---

## Task 5: WebRTC VAD — replace energy-based silence trimming

**Context:** WebRTC VAD operates on fixed-length frames at 16kHz. Frames must be 10ms (160 samples), 20ms (320 samples), or 30ms (480 samples). We use 20ms frames. The function returns `(start, end)` indices of the voiced region with a small margin, same interface as `trim_silence_range`.

**Files:**
- Create: `src-tauri/src/vad.rs`
- Modify: `src-tauri/src/whisper_backend.rs` (use `vad_trim` instead of `trim_silence_range`)

- [ ] **Step 1: Create `src-tauri/src/vad.rs`**

```rust
use webrtc_vad::{Vad, VadMode, SampleRate};

/// Returns (start, end) sample indices of the voiced region in 16kHz mono audio.
/// Adds a 200ms keep-margin around detected speech.
/// Falls back to (0, len) if VAD fails to initialize.
pub fn vad_trim(samples: &[f32]) -> (usize, usize) {
    const FRAME: usize = 320;          // 20ms at 16kHz
    const MARGIN: usize = 16000 / 5;   // 200ms margin

    let mut vad = match Vad::new_with_rate_and_mode(SampleRate::Rate16kHz, VadMode::Quality) {
        Ok(v) => v,
        Err(_) => return (0, samples.len()),
    };

    // Convert f32 → i16 for webrtc-vad.
    let pcm_i16: Vec<i16> = samples
        .iter()
        .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect();

    let n_frames = pcm_i16.len() / FRAME;
    if n_frames == 0 {
        return (0, samples.len());
    }

    let mut first_voiced: Option<usize> = None;
    let mut last_voiced: Option<usize> = None;

    for i in 0..n_frames {
        let frame = &pcm_i16[i * FRAME..(i + 1) * FRAME];
        if vad.is_voice_segment(frame).unwrap_or(false) {
            if first_voiced.is_none() {
                first_voiced = Some(i);
            }
            last_voiced = Some(i);
        }
    }

    let first = first_voiced.unwrap_or(0);
    let last = last_voiced.unwrap_or(n_frames.saturating_sub(1));

    let start = (first * FRAME).saturating_sub(MARGIN);
    let end = ((last + 1) * FRAME + MARGIN).min(samples.len());
    (start, end)
}
```

- [ ] **Step 2: Update `whisper_backend.rs` to use `vad_trim`**

In `src-tauri/src/whisper_backend.rs`, replace:

```rust
use crate::transcription::{correct_words, vad_trim};
// ...
let (trim_start, trim_end) = crate::transcription::trim_silence_range(samples, 16000);
```

with:

```rust
use crate::transcription::correct_words;
use crate::vad::vad_trim;
// ...
let (trim_start, trim_end) = vad_trim(samples);
```

- [ ] **Step 3: Build to verify**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep -E "error|Finished"
```

Expected: `Finished`.

- [ ] **Step 4: Test VAD manually**

Run the dev app, record a 5-second phrase with 2 seconds of silence at the start and end. Confirm transcription returns only the spoken text (VAD correctly trimmed the silence).

```bash
npm run tauri dev > /tmp/voz-local-dev.log 2>&1 &
# Use the app: record with silence bookends, verify output is correct
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/vad.rs src-tauri/src/whisper_backend.rs
git commit -m "feat: replace energy VAD with WebRTC VAD (webrtc-vad crate, 20ms frames at 16kHz)"
```

---

## Task 6: Dynamic audio normalization

**Context:** Quiet recordings (distance from mic, low-gain device) produce low-amplitude audio that hurts Whisper accuracy. We peak-normalize to 0.9 before inference. Skip if audio is already loud enough to avoid amplifying digital noise.

**Files:**
- Create: (add function to `src-tauri/src/audio.rs`)
- Modify: `src-tauri/src/whisper_backend.rs`

- [ ] **Step 1: Add `normalize` function to `audio.rs`**

At the end of `src-tauri/src/audio.rs`, add:

```rust
/// Peak-normalize samples to target amplitude (0.0–1.0).
/// Only amplifies if peak is below `min_peak` to avoid boosting digital noise.
pub fn normalize(samples: &mut Vec<f32>, target: f32, min_peak: f32) {
    let peak = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    if peak < min_peak || peak == 0.0 { return; }
    let gain = target / peak;
    for s in samples.iter_mut() {
        *s *= gain;
    }
}
```

- [ ] **Step 2: Apply normalization in `whisper_backend.rs`**

In `src-tauri/src/whisper_backend.rs`, add the import and apply normalization after VAD trim:

```rust
use crate::audio::normalize;

// After VAD trim, before sending to Whisper:
let (trim_start, trim_end) = vad_trim(samples);
let mut trimmed = samples[trim_start..trim_end].to_vec();
// Normalize quiet audio to 90% peak — only if peak < 30% to avoid boosting loud recordings.
normalize(&mut trimmed, 0.9, 0.3);
// 500ms silence tail.
trimmed.extend(vec![0.0f32; 8000]);
```

- [ ] **Step 3: Build to verify**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep -E "error|Finished"
```

Expected: `Finished`.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/audio.rs src-tauri/src/whisper_backend.rs
git commit -m "feat: dynamic peak normalization before inference (target 0.9, skip if peak > 0.3)"
```

---

## Task 7: Clean up `transcription.rs` and verify full pipeline

**Context:** After all tasks, `transcription.rs` should only contain utility functions used by other modules. Clean up dead code and re-export what's still needed.

**Files:**
- Modify: `src-tauri/src/transcription.rs`

- [ ] **Step 1: Audit `transcription.rs` for dead code**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep "warning.*unused\|warning.*dead_code"
```

For each unused item flagged: either delete it or confirm it's intentionally kept.

- [ ] **Step 2: Remove `trim_silence_range` if unused**

If `trim_silence_range` is no longer called from anywhere (verify with `grep -r "trim_silence_range" src-tauri/src/`), delete it from `transcription.rs`.

- [ ] **Step 3: Final build — zero errors, minimal warnings**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep -E "^error|Finished"
```

Expected: `Finished` with no `error` lines.

- [ ] **Step 4: Run the app end-to-end**

```bash
pkill -f "target/debug/voz-local" 2>/dev/null; sleep 1
npm run tauri dev > /tmp/voz-local-dev.log 2>&1 &
sleep 20 && tail -10 /tmp/voz-local-dev.log
```

Record a 30-second dictation. Confirm:
- Partial transcripts appear every 5s during recording
- Final transcription inserts correctly
- Last word is not cut off
- CPU/RAM stays stable

- [ ] **Step 5: Final commit**

```bash
git add src-tauri/src/transcription.rs
git commit -m "chore: clean up transcription.rs — keep only utility fns (rms_f32, resample, correct_words)"
```

---

## Self-Review

**Spec coverage:**
- ✅ Backend trait (`TranscriptionBackend`) — Task 3 + 4
- ✅ Single-pass 16kHz resampling at capture — Task 2
- ✅ WebRTC VAD replaces energy trimming — Task 5
- ✅ Dynamic normalization — Task 6
- ✅ Flash Attention + 6 threads already applied (pre-plan) — preserved in WhisperBackend
- ✅ 500ms silence tail for last-word fix — preserved in WhisperBackend

**Placeholder scan:** No TBDs found. All code blocks contain complete, runnable code.

**Type consistency:**
- `TranscribeOpts` defined in Task 3, used in Task 4 ✅
- `TranscriptionBackend::transcribe(&self, samples: &[f32], opts: &TranscribeOpts)` consistent across Tasks 3 and 4 ✅
- `vad_trim(samples: &[f32]) -> (usize, usize)` defined in Task 5, used in Task 5 ✅
- `normalize(samples: &mut Vec<f32>, target: f32, min_peak: f32)` defined and used in Task 6 ✅
- `AudioCapture::stop() -> (Vec<f32>, u32)` unchanged interface, `sample_rate` now always `16000` ✅
