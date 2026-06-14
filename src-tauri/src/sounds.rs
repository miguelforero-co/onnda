//! Native feedback sounds. Implemented in plan 01-04.
//!
//! Three distinct, subtle macOS system cues fire at the recording state-machine
//! points: listen (start), stop (transcribe), cancel. We use NSSound's
//! `soundNamed:` against the built-in system sounds — no asset bundling, fully
//! dependency-free and reliable. Playback is asynchronous (`NSSound play`
//! returns immediately), so the recording thread never blocks waiting on audio
//! (threat T-01-10).
//!
//! Why native (Rust) instead of WebView audio: the main window / widget may be
//! hidden while dictating, and WebView audio is unreliable then (D-07, PATTERNS
//! authoritative correction). NSSound plays regardless of window visibility.

#[cfg(target_os = "macos")]
fn play_named(name: &str) {
    use objc2::msg_send;
    use objc2::runtime::AnyClass;
    use objc2_foundation::NSString;
    unsafe {
        let Some(cls) = AnyClass::get(c"NSSound") else {
            return;
        };
        let ns_name = NSString::from_str(name);
        // `soundNamed:` returns an autoreleased NSSound* (or nil). We don't
        // retain it: `play` schedules playback on the shared sound runloop and
        // the cue is short, so the autoreleased instance outlives the cue.
        let sound: *mut objc2::runtime::AnyObject = msg_send![cls, soundNamed: &*ns_name];
        if sound.is_null() {
            return;
        }
        let _: bool = msg_send![sound, play];
    }
}

#[cfg(target_os = "macos")]
pub fn play_listen() {
    play_named("Tink");
}
#[cfg(target_os = "macos")]
pub fn play_stop() {
    play_named("Pop");
}
#[cfg(target_os = "macos")]
pub fn play_cancel() {
    play_named("Funk");
}

#[cfg(not(target_os = "macos"))]
pub fn play_listen() {}
#[cfg(not(target_os = "macos"))]
pub fn play_stop() {}
#[cfg(not(target_os = "macos"))]
pub fn play_cancel() {}
