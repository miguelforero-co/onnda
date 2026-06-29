<div align="center">

# onnda

### Talk. Your words appear where you type.

**Voice-to-text for macOS that runs entirely on your Mac.**
No cloud. No account. No subscription. Your voice never leaves the machine.

[![Download](https://img.shields.io/badge/⬇_Download_for_macOS-000000?style=for-the-badge)](https://github.com/miguelforero-co/onnda/releases/latest)

[![Release](https://img.shields.io/github/v/release/miguelforero-co/onnda?style=flat-square)](https://github.com/miguelforero-co/onnda/releases/latest)
[![macOS 11+](https://img.shields.io/badge/macOS-11%2B-black?style=flat-square&logo=apple)](https://github.com/miguelforero-co/onnda/releases/latest)
[![Apple Silicon + Intel](https://img.shields.io/badge/Apple_Silicon-%2B_Intel-black?style=flat-square)](https://github.com/miguelforero-co/onnda/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-black?style=flat-square)](LICENSE)

</div>

---

Press `Alt+Space`, say what you mean, and let go. onnda transcribes it **on-device** and drops the text right where your cursor is — your editor, your browser, a chat box, anywhere. A small wave animation in the notch shows it's listening. That's the whole thing.

You talk roughly three times faster than you type. onnda gives you that speed without handing your voice to anyone's servers.

## Why onnda

- **Truly local.** Transcription runs on your Mac with Whisper (Metal-accelerated) or Apple's on-device Speech engine. Nothing is uploaded, ever.
- **Private by design.** No account, no password, no telemetry you didn't ask for. Your dictation, history, and settings live on your machine and nowhere else.
- **Free and open source.** MIT licensed. Read it, fork it, ship it.
- **Fast.** Push-to-talk that fires even when the window is closed, and pastes without stealing focus from the app you're in.

## How it works

onnda lives in your menu bar, not the Dock. Hold (or tap) your shortcut, speak, release — the audio is transcribed locally and pasted at your cursor. There's nothing to sign up for and nothing to configure to get going.

## Features

- **Local transcription** with [Whisper](https://github.com/ggerganov/whisper.cpp) (Metal) or Apple Speech — pick the model, it runs offline.
- **Global push-to-talk** (`Alt+Space` by default), push-to-talk or toggle mode, works app-wide.
- **Auto-paste** at your cursor without stealing keyboard focus.
- **Transcribe files** — drop in an audio file, get the transcript.
- **History you can edit** — review, copy, play back, and correct. onnda learns from your corrections.
- **Dictionary & replacements** — custom vocabulary plus `from → to` rules for names, brands, and jargon.
- **Notch widget** with a live voice animation and adjustable mic sensitivity.
- **Light / dark / auto** themes.
- **Apple Silicon and Intel**, with launch-at-login.

## Privacy

Speech processing is **100% local** — your audio and transcripts never touch a server. Network access is limited to:

- Downloading voice models and checking for updates.
- **Anonymous usage stats** — *opt-in*, off by default. Event counts only (e.g. "transcription completed"), never any text.
- **Your email** — *optional*. Only sent if you type it on the welcome screen, to join the launch-updates list. Leave it blank and nothing leaves your Mac.

No account. No password. Your name and settings stay on this Mac.

## Install

1. Download the `.dmg` from [Releases](https://github.com/miguelforero-co/onnda/releases/latest) — `aarch64` for Apple Silicon, `x86_64` for Intel.
2. Open it and drag **onnda** to Applications.
3. Launch it from Applications. It's signed with an Apple Developer ID and notarized by Apple, so it opens without Gatekeeper warnings.
4. Grant **Microphone** and **Accessibility** when prompted.
5. Download a voice model from the welcome screen.
6. Press `Alt+Space` and start talking.

## Permissions

| Permission | Why |
|---|---|
| **Microphone** | Capture your voice. |
| **Accessibility** | Paste the transcribed text at your cursor (a simulated `Cmd+V`). |

> Without Accessibility, the text is still copied to your clipboard — paste it with `Cmd+V`.

## Models

Models aren't bundled with the installer — pick and download one on first launch. They're stored under `~/Library/Application Support/com.onnda.app/models/` and work fully offline once downloaded. Apple's on-device Speech engine needs no download on supported Macs.

## Building from source

**Requirements:** [Rust](https://rustup.rs) (stable), [Node.js](https://nodejs.org) ≥ 20, and Xcode Command Line Tools (`xcode-select --install`).

```bash
git clone git@github.com:miguelforero-co/onnda.git
cd onnda
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

[MIT](LICENSE) © Miguel Forero
