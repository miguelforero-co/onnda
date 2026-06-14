//! Mute/unmute system audio output while dictating.
//!
//! Earlier this module synthesized the hardware Play/Pause media key, but that
//! was wrong: toggling Play/Pause RESUMES already-paused music, and there's no
//! way to know "is something playing". The user wants the opposite — while
//! listening, SILENCE whatever is playing (Spotify/YouTube/etc.) and restore it
//! afterward, and NEVER start playback.
//!
//! So we mute the macOS system output volume instead:
//!
//!   - On recording start (`mute_outputs`): read the current muted state via
//!     AppleScript (`output muted of (get volume settings)`). If output is NOT
//!     already muted, mute it (`set volume output muted true`) and record that
//!     WE muted it (`WE_MUTED = true`). If it was already muted, do nothing.
//!   - On stop/cancel (`restore_outputs`): only if WE muted, unmute
//!     (`set volume output muted false`) and clear the flag. We never unmute
//!     something the user had already muted, and never start playback.
//!
//! The `WE_MUTED` AtomicBool keeps mute/unmute symmetric (threat T-01-08). The
//! whole feature is gated behind the opt-in `pause_media` setting.
//!
//! Implementation is `osascript` (AppleScript) via `std::process::Command` — no
//! private APIs, no CoreAudio FFI. The mute is applied on a short-delayed
//! background thread (~280ms) so the recording-start cue is heard BEFORE output
//! goes silent.

#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "macos")]
static WE_MUTED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "macos")]
fn osascript(script: &str) -> Option<String> {
    use std::process::Command;
    let out = Command::new("osascript").arg("-e").arg(script).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[cfg(target_os = "macos")]
fn is_output_muted() -> bool {
    matches!(
        osascript("output muted of (get volume settings)").as_deref(),
        Some("true")
    )
}

#[cfg(target_os = "macos")]
pub fn mute_outputs() {
    // Delay so the start cue plays before output is silenced; do the AppleScript
    // off the recording thread so we never block it.
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(280));
        if !is_output_muted() {
            if osascript("set volume output muted true").is_some() {
                WE_MUTED.store(true, Ordering::SeqCst);
            }
        }
    });
}

#[cfg(target_os = "macos")]
pub fn restore_outputs() {
    if WE_MUTED.swap(false, Ordering::SeqCst) {
        // Run off-thread too; unmute is best-effort and shouldn't block stop.
        std::thread::spawn(|| {
            osascript("set volume output muted false");
        });
    }
}

#[cfg(not(target_os = "macos"))]
pub fn mute_outputs() {}
#[cfg(not(target_os = "macos"))]
pub fn restore_outputs() {}
