---
phase: 01-redise-o-del-panel-de-configuraci-n-estilo-whisprflow
plan: 05
subsystem: file-transcription
tags: [symphonia, audio-decode, whisper, tauri-command, spawn-blocking, unified-history]
requires:
  - "01-01 (history::save_entry now source-aware: source + original_filename params)"
  - "01-02 (symphonia dep + features [mp3, isomp4, aac, alac, wav, pcm]; empty audio_decode.rs stub)"
  - "transcription::resample, backend::TranscribeOpts, whisper_backend::WhisperBackend (pre-existing)"
provides:
  - "audio_decode::decode_file(path) -> (Vec<f32> mono, u32 sample_rate) via symphonia 0.6"
  - "commands::transcribe_file(app, path) -> Result<String> — full decode→resample→Whisper→correct→save pipeline"
  - "Tauri events: file-transcribe-progress (\"decoding\"|\"transcribing\"), file-transcribe-done (text), file-transcribe-error (message)"
affects:
  - "src/ frontend (01-08 wires the file-picker + listens for file-transcribe-* events)"
  - "Transcripciones view (01-06/01-08 renders entries with source=\"file\" tagged by original_filename)"
tech-stack:
  added: []
  patterns:
    - "symphonia 0.6 decode: get_probe().probe() → default_track(TrackType::Audio) → CodecParameters::Audio → make_audio_decoder → next_packet() loop → GenericAudioBufferRef::copy_to_vec_interleaved::<f32>"
    - "untrusted-file safety: every fallible call map_err→String, per-packet decode errors skipped (not fatal), streaming packet loop (no whole-file buffering), empty output → graceful error"
    - "heavy work (decode + Whisper inference) on tokio::task::spawn_blocking so the async runtime / UI never blocks"
key-files:
  created: []
  modified:
    - src-tauri/src/audio_decode.rs
    - src-tauri/src/commands.rs
    - src-tauri/src/lib.rs
decisions:
  - "decode_file returns audio at the file's NATIVE rate; caller resamples to 16k (keeps decode pure, reuses existing resample())"
  - "downmix to mono by simple per-frame channel averaging (matches RESEARCH decode→downmix→resample→f32 sequence)"
  - "next_packet() Err is treated as end-of-stream (keep what decoded) rather than a hard failure — robust against truncated files"
  - "file transcription never pastes (Cmd+V) — it is not a dictation; only emits file-transcribe-done with the text"
metrics:
  duration: ~12m
  completed: 2026-06-14
  tasks: 2
  files: 3
---

# Phase 01 Plan 05: File transcription pipeline (symphonia decode → Whisper) Summary

A `decode_file` that turns any wav/mp3/m4a (and aac/alac/pcm) into mono f32 PCM via symphonia 0.6, plus a `transcribe_file` Tauri command that runs the full off-thread pipeline — decode → resample to 16 kHz → existing Whisper backend → vocabulary correction → unified history entry tagged `source: "file"` with the original filename — emitting `file-transcribe-progress/done/error` events for the frontend (01-08) to drive a file-picker UX.

## What Was Built

- **`audio_decode.rs` — `pub fn decode_file(path: &str) -> Result<(Vec<f32>, u32), String>`**: probes the container with `symphonia::default::get_probe().probe(...)`, selects `default_track(TrackType::Audio)`, reads `sample_rate` + channel count off `CodecParameters::Audio`, builds a decoder via `get_codecs().make_audio_decoder(...)`, and loops `next_packet()` decoding each packet to a `GenericAudioBufferRef`. Each buffer is pulled out with `copy_to_vec_interleaved::<f32>` and downmixed to mono by averaging channels per frame. Returns the mono signal at the file's native sample rate.
- **`commands.rs` — `pub async fn transcribe_file<R: Runtime>(app, path)`**: extracts the original filename, decodes on `spawn_blocking` (emitting `file-transcribe-progress: "decoding"`), resamples to 16 kHz via `transcription::resample`, resolves the model path with the exact same logic as `stop_and_transcribe_internal` (primary `ggml-{model}.bin` → fallback `ggml-base.bin`), transcribes on `spawn_blocking` (emitting `"transcribing"`), runs `correct_words`, saves via `history::save_entry(..., "file".to_string(), original_filename)`, and emits `file-transcribe-done` with the text. Does NOT paste.
- **`lib.rs`**: registered `commands::transcribe_file` in the `generate_handler!` list.

## Decisions Made

- **Native-rate decode, caller resamples.** `decode_file` returns audio at the file's own sample rate and the command resamples to 16 kHz, keeping decode reusable and matching the existing `resample()` contract used elsewhere.
- **`next_packet()` errors = end-of-stream.** A read error mid-stream keeps whatever decoded rather than failing the whole transcription — robust against truncated/partial files.
- **No paste on file transcription.** File transcription is an explicit import action, not a dictation, so it skips the Cmd+V paste path used by `stop_and_transcribe_internal`.

## Deviations from Plan

The plan's code sketch targeted the symphonia 0.5 audio API (`SampleBuffer`, the `Signal` trait, `decoded.spec()`/`decoded.capacity()`, `default_track()` with no args, `next_packet() -> Result<Packet>`). The locked dependency is symphonia **0.6.0**, whose audio/codec/format APIs were reworked. I verified the exact 0.6 symbols against the vendored source and implemented against them — the plan explicitly delegated this ("executor verifies exact symbols against the locked version"), so this is faithful execution, not a behavioral deviation.

### Auto-fixed Issues (Rule 3 — API compatibility to complete the task)

**1. [Rule 3 - Blocking] symphonia 0.6 API surface differs from the 0.5-style sketch**
- **Found during:** Task 1
- **Issue:** `SampleBuffer`/`Signal`/`copy_interleaved_ref`/`decoded.spec()` don't exist in 0.6; `default_track` requires a `TrackType` arg; `next_packet()` returns `Result<Option<Packet>>`; codec params live on `CodecParameters::Audio(AudioCodecParameters)`; the decoder is created with `make_audio_decoder` + `AudioDecoderOptions`.
- **Fix:** Implemented with the 0.6 API — `get_probe().probe()`, `default_track(TrackType::Audio)`, `CodecParameters::Audio` to read `sample_rate`/`channels`, `make_audio_decoder`, `next_packet()` matching `Ok(Some)/Ok(None)/Err`, and `GenericAudioBufferRef::copy_to_vec_interleaved::<f32>` for interleaved samples before per-frame downmix.
- **Files modified:** src-tauri/src/audio_decode.rs
- **Commit:** 36d5679

## Threat Model Compliance

- **T-01-11 (DoS, huge/malformed file):** decode runs on `spawn_blocking` (UI never blocks); the packet loop streams (no whole-file buffer of compressed data). Mitigated.
- **T-01-12 (Tampering, crafted file → panic):** every symphonia call `map_err`s to `String`; the loop matches `Err => break`/`continue` (per-packet errors skipped); empty output → graceful `"no se decodificó audio"`. Proven by `decode_garbage_errs` + `decode_missing_file_errs` tests. Mitigated.
- **T-01-13 (path traversal):** accepted — path originates from the user-chosen dialog in a single-user local app.

## Events for the Frontend (01-08)

| Event | Payload | When |
|-------|---------|------|
| `file-transcribe-progress` | `"decoding"` then `"transcribing"` | stage transitions |
| `file-transcribe-done` | transcribed text (String) | success |
| `file-transcribe-error` | error message (String) | decode/model/empty/Whisper failure |

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml audio_decode` → 3 passed (`decode_wav_fixture`, `decode_missing_file_errs`, `decode_garbage_errs`).
- `cargo build --manifest-path src-tauri/Cargo.toml` → exits 0, no warnings.
- Manual (deferred to 01-09 checklist): pick a real mp3/m4a, confirm it transcribes and appears in Transcripciones tagged as file with its filename.

## Commits

- `36d5679` feat(01-05): decode audio files to mono f32 PCM via symphonia
- `3fa6dd3` feat(01-05): add transcribe_file command for file transcription

## Self-Check: PASSED

- Files: audio_decode.rs, commands.rs, lib.rs all present.
- Commits: 36d5679, 3fa6dd3 both in git history.
