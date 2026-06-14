//! Pause/resume media playback via the system Play/Pause media key.
//! Implemented in plan 01-04.
//!
//! There is no public API to query "is something playing" (the only path is
//! private MediaRemote, rejected in RESEARCH). So we model pause/resume as a
//! single symmetric toggle of the hardware Play/Pause key (NX_KEYTYPE_PLAY):
//!
//!   - On recording start (`pause_if_playing`): send the toggle once and record
//!     that WE toggled it (`I_PAUSED = true`).
//!   - On stop/cancel (`resume_if_paused`): if WE toggled it, send the toggle
//!     again to undo, and clear the flag.
//!
//! The `I_PAUSED` AtomicBool guards against desync from repeated toggles
//! (threat T-01-08): exactly one pause is ever matched by one resume. The whole
//! feature is gated behind the opt-in `pause_media` setting in commands.rs, so
//! if nothing was playing the user just gets a harmless no-op pause/resume pair.
//!
//! The Play/Pause key is delivered as an NSSystemDefined CGEvent (event type
//! 14, subtype 8) carrying NX_KEYTYPE_PLAY in data1 — the standard way apps
//! synthesize media keys. FFI mirrors `commands::post_cmd_v`.

#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "macos")]
static I_PAUSED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "macos")]
fn send_play_pause() {
    use std::ffi::c_void;

    // NSEvent's otherEventWithType:... builds the NSSystemDefined event; we then
    // post its underlying CGEvent. This is the reliable, public path for media
    // keys (a bare CGEventCreate can't set the system-defined subtype/data).
    use objc2::msg_send;
    use objc2::runtime::AnyClass;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventPost(tap: i32, event: *mut c_void);
    }

    const NS_SYSTEM_DEFINED: u64 = 14; // NSEventTypeSystemDefined
    const SUBTYPE: i16 = 8; // NX_SUBTYPE_AUX_CONTROL_BUTTONS
    const NX_KEYTYPE_PLAY: i64 = 16;
    const HID_TAP: i32 = 0; // kCGHIDEventTap

    unsafe {
        let Some(cls) = AnyClass::get(c"NSEvent") else {
            return;
        };

        // data1 packs the key code in the high 16 bits and the key state in the
        // low bits: 0xA00 = key down, 0xB00 = key up.
        let post = |key_down: bool| {
            let data1: i64 = (NX_KEYTYPE_PLAY << 16) | if key_down { 0xA00 } else { 0xB00 };
            // otherEventWithType:location:modifierFlags:timestamp:windowNumber:context:subtype:data1:data2:
            let event: *mut objc2::runtime::AnyObject = msg_send![
                cls,
                otherEventWithType: NS_SYSTEM_DEFINED,
                location: objc2_foundation::NSPoint { x: 0.0, y: 0.0 },
                modifierFlags: 0xA00_u64,
                timestamp: 0.0_f64,
                windowNumber: 0_i64,
                context: std::ptr::null_mut::<objc2::runtime::AnyObject>(),
                subtype: SUBTYPE,
                data1: data1,
                data2: -1_i64,
            ];
            if event.is_null() {
                return;
            }
            // -[NSEvent CGEvent] returns the underlying CGEventRef (autoreleased,
            // owned by the NSEvent — do not release it ourselves).
            let cg: *mut c_void = msg_send![event, CGEvent];
            if !cg.is_null() {
                CGEventPost(HID_TAP, cg);
            }
        };

        post(true);
        post(false);
    }
}

#[cfg(target_os = "macos")]
pub fn pause_if_playing() {
    send_play_pause();
    I_PAUSED.store(true, Ordering::SeqCst);
}

#[cfg(target_os = "macos")]
pub fn resume_if_paused() {
    if I_PAUSED.swap(false, Ordering::SeqCst) {
        send_play_pause();
    }
}

#[cfg(not(target_os = "macos"))]
pub fn pause_if_playing() {}
#[cfg(not(target_os = "macos"))]
pub fn resume_if_paused() {}
