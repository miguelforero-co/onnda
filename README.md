<div align="center">

# onnda

**voice to text — 100% local on your Mac. No cloud, no subscription, no internet.**

Press a shortcut, speak, and the text appears wherever your cursor is — in any app.

[![Release](https://img.shields.io/github/v/release/miguelforero-co/voz-local?style=flat-square)](https://github.com/miguelforero-co/voz-local/releases/latest)
[![macOS](https://img.shields.io/badge/macOS-14%2B-black?style=flat-square&logo=apple)](https://github.com/miguelforero-co/voz-local/releases/latest)
[![Apple Silicon + Intel](https://img.shields.io/badge/Apple_Silicon-%2B_Intel-black?style=flat-square)](https://github.com/miguelforero-co/voz-local/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-black?style=flat-square)](LICENSE)

</div>

---

## Download

**[⬇ Download onnda for macOS](https://github.com/miguelforero-co/voz-local/releases/latest)**

> macOS 14 (Sonoma) or later · Apple Silicon or Intel

---

## How it works

onnda lives in your menu bar. Hold (or tap) your shortcut, speak, and release — the audio is transcribed **entirely on your Mac** and the text is pasted right where you were typing. A small wave animation in the notch shows it's listening. Nothing is uploaded; there's no account to sign up for.

## Features

- **Local transcription** — [Whisper](https://github.com/ggerganov/whisper.cpp) accelerated by Metal, or Apple's on-device Speech engine. The model runs entirely on your Mac.
- **Global shortcut** (default `Alt+Space`) — works even when the window is closed. Push-to-talk or toggle mode.
- **Auto-paste** wherever your cursor is — without stealing keyboard focus.
- **Transcribe files** — drop an audio file and get its transcript.
- **History** — review, copy, play back, and **edit** transcriptions. onnda learns from your corrections.
- **Dictionary & replacements** — custom vocabulary plus `from → to` rules for names, brands, and jargon.
- **Light / dark / auto themes.**
- **Notch widget** with a live voice animation and adjustable sensitivity.
- **Model management** from Settings.
- **Apple Silicon and Intel**, launch-at-login support.

## Privacy

All speech processing is **local**. onnda never sends your audio or transcripts to any server. The only optional network calls are downloading voice models and checking for updates.

## Installation

1. Download the `.dmg` from [Releases](https://github.com/miguelforero-co/voz-local/releases/latest).
2. Open it and drag **onnda** to Applications.
3. Launch it — it lives in the menu bar (not the Dock).
4. Grant **Microphone** and **Accessibility** when prompted.
5. Download a voice model from the welcome screen.
6. Press `Alt+Space` and start dictating.

> **If macOS says the app is damaged:** run `sudo xattr -cr /Applications/onnda.app`

## Permissions

| Permission | Why |
|---|---|
| **Microphone** | Capture your voice. |
| **Accessibility** | Paste the transcribed text at your cursor (simulated `Cmd+V`). |

> Without Accessibility, the text is still copied to your clipboard — paste it manually with `Cmd+V`.

## Models

Models aren't bundled with the installer — pick and download one on first launch. They're stored under `~/Library/Application Support/com.vozlocal.app/models/` and work fully offline once downloaded. Apple's on-device Speech engine needs no download on supported Macs.

## Building from source

**Requirements:** [Rust](https://rustup.rs) (stable), [Node.js](https://nodejs.org) ≥ 20, and Xcode Command Line Tools (`xcode-select --install`).

```bash
git clone git@github.com:miguelforero-co/voz-local.git
cd voz-local
npm install
npm run tauri dev      # first run compiles the Rust backend (takes a while)
npm run tauri build    # production bundle → src-tauri/target/release/bundle/
```

> **Dev note:** in development, macOS attributes the **Accessibility** permission to the terminal that launches the app (e.g. Ghostty/Terminal), not to the dev binary. Grant it to that terminal in *System Settings → Privacy & Security → Accessibility* for auto-paste to work. Signed release builds don't have this quirk.

Checks:

```bash
npm run check                                    # types + Svelte
cargo test --manifest-path src-tauri/Cargo.toml  # backend tests
```

## Tech stack

| Layer | Technology |
|---|---|
| Frontend | SvelteKit + Svelte 5 + TypeScript (adapter-static) |
| Backend | Rust + Tauri 2 |
| Transcription | [whisper-rs](https://github.com/tazz4843/whisper-rs) + Metal, and Apple Speech (Swift sidecar) |
| Audio capture | cpal + energy-based VAD |
| Word correction | [strsim](https://github.com/dguo/strsim-rs) (Jaro-Winkler) |
| Design | onnda design system · [Iconoir](https://iconoir.com) icons · Goudy Bookletter 1911 |

## License

MIT
