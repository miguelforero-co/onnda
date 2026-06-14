//! Native feedback sounds. Implemented in plan 01-04; custom cues added later.
//!
//! Three distinct, subtle cues fire at the recording state-machine points:
//! listen (start), stop (transcribe), cancel. Instead of the built-in macOS
//! system cues (Tink/Pop/Funk), we ship three custom-generated WAVs (see
//! `scripts/gen-sounds.py`) for a cooler, more dynamic feel: a rising blip on
//! start, a two-note ascending chime on stop, a soft descending tone on cancel.
//!
//! The WAV bytes are embedded at compile time (`include_bytes!`), wrapped in an
//! `NSData`, and played via `NSSound initWithData:` + `play`. Playback is
//! asynchronous (`NSSound play` returns immediately), so the recording thread
//! never blocks waiting on audio (threat T-01-10). NSSound plays regardless of
//! window visibility — the widget may be hidden while dictating (D-07).

#[cfg(target_os = "macos")]
const LISTEN_WAV: &[u8] = include_bytes!("../sounds/listen.wav");
#[cfg(target_os = "macos")]
const STOP_WAV: &[u8] = include_bytes!("../sounds/stop.wav");
#[cfg(target_os = "macos")]
const CANCEL_WAV: &[u8] = include_bytes!("../sounds/cancel.wav");

#[cfg(target_os = "macos")]
fn play_bytes(bytes: &'static [u8]) {
    use objc2::rc::Retained;
    use objc2::AllocAnyThread;
    use objc2_app_kit::NSSound;
    use objc2_foundation::NSData;

    unsafe {
        // Copy the embedded bytes into an NSData. dataWithBytes:length: copies,
        // so the autoreleased NSData owns its own buffer (no lifetime coupling to
        // our 'static slice beyond this call).
        let data = NSData::dataWithBytes_length(
            bytes.as_ptr() as *const std::ffi::c_void,
            bytes.len(),
        );
        let Some(sound): Option<Retained<NSSound>> = NSSound::initWithData(NSSound::alloc(), &data)
        else {
            return;
        };
        // `play` schedules playback on the shared sound runloop and returns
        // immediately. We retain `sound` for the duration of this fn; the short
        // cue plays out on the runloop even after it drops, because AppKit holds
        // its own reference once playing.
        let _ = sound.play();
    }
}

#[cfg(target_os = "macos")]
pub fn play_listen() {
    play_bytes(LISTEN_WAV);
}
#[cfg(target_os = "macos")]
pub fn play_stop() {
    play_bytes(STOP_WAV);
}
#[cfg(target_os = "macos")]
pub fn play_cancel() {
    play_bytes(CANCEL_WAV);
}

#[cfg(not(target_os = "macos"))]
pub fn play_listen() {}
#[cfg(not(target_os = "macos"))]
pub fn play_stop() {}
#[cfg(not(target_os = "macos"))]
pub fn play_cancel() {}
