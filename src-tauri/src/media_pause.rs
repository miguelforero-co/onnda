//! Mute/unmute system audio output while dictating.
//!
//! Earlier this module synthesized the hardware Play/Pause media key, but that
//! was wrong: toggling Play/Pause RESUMES already-paused music, and there's no
//! way to know "is something playing". The user wants the opposite — while
//! listening, SILENCE whatever is playing (Spotify/YouTube/etc.) and restore it
//! afterward, and NEVER start playback. So we mute the system output instead.
//!
//! ## Why CoreAudio instead of AppleScript
//!
//! The previous implementation used `osascript` (`set volume output muted true`
//! / `output muted of (get volume settings)`). That silently FAILS on many
//! output devices: with Bluetooth headphones (AirPods), external DACs, and other
//! non-built-in outputs, macOS does NOT expose software volume/mute via the
//! AppleScript `volume settings` interface — `output volume of (get volume
//! settings)` and `output muted` both return the literal string `"missing
//! value"`, so the mute is a no-op and the user's music keeps playing.
//!
//! Instead we drive the CoreAudio HAL directly against the DEFAULT OUTPUT
//! DEVICE, which works regardless of the device type (built-in, Bluetooth,
//! USB/DAC, etc.):
//!
//!   - On recording start (`mute_outputs`): find the default output device, then
//!     mute it. Primary path is the device's `mute` property
//!     (`kAudioDevicePropertyMute`); if that device has no settable mute (some
//!     devices don't), fall back to setting the virtual main volume
//!     (`kAudioHardwareServiceDeviceProperty_VirtualMainVolume`) to 0.0 and
//!     remembering the prior level. Only act if it wasn't already silent, and
//!     record (via `WE_MUTED`) that WE silenced it.
//!   - On stop/cancel (`restore_outputs`): only if WE muted, reverse the EXACT
//!     method used (unmute, or restore the saved prior volume) and clear the
//!     flag. We never unmute/raise something the user had already silenced.
//!
//! The `WE_MUTED` AtomicBool keeps mute/unmute symmetric (threat T-01-08), and
//! `MUTE_METHOD` records which path was used so restore reverses the right one.
//! The whole feature is gated behind the opt-in `pause_media` setting.
//!
//! CoreAudio calls are synchronous and complete in microseconds, so unlike the
//! AppleScript version we do NOT defer the work to a delayed background thread.
//! That earlier 280ms delay caused a race: a recording shorter than the delay
//! would run `restore_outputs` BEFORE the queued `mute_outputs` fired, leaving
//! the system stuck muted. Doing it inline removes the race entirely. (The start
//! cue may now be clipped by the immediate mute; that's acceptable while the cue
//! set is being redesigned separately.) No private APIs are used — only the
//! public CoreAudio HAL.

#[cfg(target_os = "macos")]
mod imp {
    use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};

    /// True only while WE are the ones holding the output silenced, so restore is
    /// symmetric and we never touch a mute/volume the user set themselves.
    static WE_MUTED: AtomicBool = AtomicBool::new(false);
    /// Which path silenced the device: 0 = none, 1 = mute property, 2 = volume.
    static MUTE_METHOD: AtomicU8 = AtomicU8::new(0);
    /// Prior virtual main volume (f32 bits), saved only for the volume fallback.
    static PRIOR_VOLUME: AtomicU32 = AtomicU32::new(0);

    const METHOD_NONE: u8 = 0;
    const METHOD_MUTE: u8 = 1;
    const METHOD_VOLUME: u8 = 2;

    // --- CoreAudio HAL FFI (public API; no private symbols) ----------------

    #[allow(non_camel_case_types)]
    type OSStatus = i32;

    #[repr(C)]
    struct AudioObjectPropertyAddress {
        m_selector: u32,
        m_scope: u32,
        m_element: u32,
    }

    #[link(name = "CoreAudio", kind = "framework")]
    extern "C" {
        fn AudioObjectGetPropertyData(
            in_object: u32,
            in_addr: *const AudioObjectPropertyAddress,
            in_qual_size: u32,
            in_qual: *const core::ffi::c_void,
            io_data_size: *mut u32,
            out_data: *mut core::ffi::c_void,
        ) -> OSStatus;
        fn AudioObjectSetPropertyData(
            in_object: u32,
            in_addr: *const AudioObjectPropertyAddress,
            in_qual_size: u32,
            in_qual: *const core::ffi::c_void,
            in_data_size: u32,
            in_data: *const core::ffi::c_void,
        ) -> OSStatus;
        fn AudioObjectHasProperty(
            in_object: u32,
            in_addr: *const AudioObjectPropertyAddress,
        ) -> bool;
        fn AudioObjectIsPropertySettable(
            in_object: u32,
            in_addr: *const AudioObjectPropertyAddress,
            out_settable: *mut bool,
        ) -> OSStatus;
    }

    /// Build a FourCC selector/scope constant from a 4-byte literal.
    const fn fcc(s: &[u8; 4]) -> u32 {
        ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ((s[2] as u32) << 8) | (s[3] as u32)
    }

    const K_AUDIO_OBJECT_SYSTEM_OBJECT: u32 = 1;
    const K_AUDIO_OBJECT_UNKNOWN: u32 = 0;
    /// kAudioHardwarePropertyDefaultOutputDevice.
    const SEL_DEFAULT_OUTPUT: u32 = 0x644F7574; // 'dOut'
    /// kAudioObjectPropertyScopeGlobal.
    const SCOPE_GLOBAL: u32 = 0x676C6F62; // 'glob'
    /// kAudioObjectPropertyScopeOutput.
    const SCOPE_OUTPUT: u32 = 0x6F757470; // 'outp'
    /// kAudioObjectPropertyElementMain.
    const ELEMENT_MAIN: u32 = 0;
    /// kAudioDevicePropertyMute (u32: 0 = unmuted, 1 = muted).
    const SEL_MUTE: u32 = 0x6D757465; // 'mute'
    /// kAudioHardwareServiceDeviceProperty_VirtualMainVolume (f32 0.0..=1.0).
    const SEL_VIRTUAL_MAIN_VOLUME: u32 = 0x766D7663; // 'vmvc'

    const OK: OSStatus = 0;

    // Compile-time sanity: the hand-written hex matches the spec's byte literals.
    const _: () = {
        assert!(SEL_DEFAULT_OUTPUT == fcc(b"dOut"));
        assert!(SCOPE_GLOBAL == fcc(b"glob"));
        assert!(SCOPE_OUTPUT == fcc(b"outp"));
        assert!(SEL_MUTE == fcc(b"mute"));
        assert!(SEL_VIRTUAL_MAIN_VOLUME == fcc(b"vmvc"));
    };

    fn addr(selector: u32, scope: u32) -> AudioObjectPropertyAddress {
        AudioObjectPropertyAddress {
            m_selector: selector,
            m_scope: scope,
            m_element: ELEMENT_MAIN,
        }
    }

    /// The current default output device, or `None` if unavailable/unknown.
    fn default_output_device() -> Option<u32> {
        let a = addr(SEL_DEFAULT_OUTPUT, SCOPE_GLOBAL);
        let mut dev_id: u32 = 0;
        let mut size: u32 = core::mem::size_of::<u32>() as u32;
        let status = unsafe {
            AudioObjectGetPropertyData(
                K_AUDIO_OBJECT_SYSTEM_OBJECT,
                &a,
                0,
                core::ptr::null(),
                &mut size,
                &mut dev_id as *mut u32 as *mut core::ffi::c_void,
            )
        };
        if status != OK || dev_id == K_AUDIO_OBJECT_UNKNOWN {
            return None;
        }
        Some(dev_id)
    }

    /// Whether a property exists AND is settable on the device.
    fn has_settable(dev: u32, a: &AudioObjectPropertyAddress) -> bool {
        if !unsafe { AudioObjectHasProperty(dev, a) } {
            return false;
        }
        let mut settable = false;
        let status = unsafe { AudioObjectIsPropertySettable(dev, a, &mut settable) };
        status == OK && settable
    }

    fn get_u32(dev: u32, a: &AudioObjectPropertyAddress) -> Option<u32> {
        let mut value: u32 = 0;
        let mut size: u32 = core::mem::size_of::<u32>() as u32;
        let status = unsafe {
            AudioObjectGetPropertyData(
                dev,
                a,
                0,
                core::ptr::null(),
                &mut size,
                &mut value as *mut u32 as *mut core::ffi::c_void,
            )
        };
        (status == OK).then_some(value)
    }

    fn set_u32(dev: u32, a: &AudioObjectPropertyAddress, value: u32) -> bool {
        let size = core::mem::size_of::<u32>() as u32;
        let status = unsafe {
            AudioObjectSetPropertyData(
                dev,
                a,
                0,
                core::ptr::null(),
                size,
                &value as *const u32 as *const core::ffi::c_void,
            )
        };
        status == OK
    }

    fn get_f32(dev: u32, a: &AudioObjectPropertyAddress) -> Option<f32> {
        let mut value: f32 = 0.0;
        let mut size: u32 = core::mem::size_of::<f32>() as u32;
        let status = unsafe {
            AudioObjectGetPropertyData(
                dev,
                a,
                0,
                core::ptr::null(),
                &mut size,
                &mut value as *mut f32 as *mut core::ffi::c_void,
            )
        };
        (status == OK).then_some(value)
    }

    fn set_f32(dev: u32, a: &AudioObjectPropertyAddress, value: f32) -> bool {
        let size = core::mem::size_of::<f32>() as u32;
        let status = unsafe {
            AudioObjectSetPropertyData(
                dev,
                a,
                0,
                core::ptr::null(),
                size,
                &value as *const f32 as *const core::ffi::c_void,
            )
        };
        status == OK
    }

    pub fn mute_outputs() {
        // Idempotent: never double-mute (would lose the saved prior state).
        if WE_MUTED.load(Ordering::SeqCst) {
            return;
        }
        let Some(dev) = default_output_device() else {
            return;
        };

        // Primary path: the device's hardware/software mute property.
        let mute_addr = addr(SEL_MUTE, SCOPE_OUTPUT);
        if has_settable(dev, &mute_addr) {
            if let Some(current) = get_u32(dev, &mute_addr) {
                if current == 0 {
                    if set_u32(dev, &mute_addr, 1) {
                        MUTE_METHOD.store(METHOD_MUTE, Ordering::SeqCst);
                        WE_MUTED.store(true, Ordering::SeqCst);
                    }
                }
                // Already muted by the user → leave it; nothing to restore.
                return;
            }
        }

        // Fallback path: drop the virtual main volume to 0 and remember it.
        let vol_addr = addr(SEL_VIRTUAL_MAIN_VOLUME, SCOPE_OUTPUT);
        if has_settable(dev, &vol_addr) {
            if let Some(prior) = get_f32(dev, &vol_addr) {
                if prior > 0.0 {
                    if set_f32(dev, &vol_addr, 0.0) {
                        PRIOR_VOLUME.store(prior.to_bits(), Ordering::SeqCst);
                        MUTE_METHOD.store(METHOD_VOLUME, Ordering::SeqCst);
                        WE_MUTED.store(true, Ordering::SeqCst);
                    }
                }
                // Already silent → leave it; nothing to restore.
            }
        }
        // Neither property settable → no-op.
    }

    pub fn restore_outputs() {
        if !WE_MUTED.swap(false, Ordering::SeqCst) {
            return;
        }
        let method = MUTE_METHOD.swap(METHOD_NONE, Ordering::SeqCst);
        let Some(dev) = default_output_device() else {
            return;
        };
        match method {
            METHOD_MUTE => {
                let mute_addr = addr(SEL_MUTE, SCOPE_OUTPUT);
                set_u32(dev, &mute_addr, 0);
            }
            METHOD_VOLUME => {
                let prior = f32::from_bits(PRIOR_VOLUME.load(Ordering::SeqCst));
                let vol_addr = addr(SEL_VIRTUAL_MAIN_VOLUME, SCOPE_OUTPUT);
                set_f32(dev, &vol_addr, prior);
            }
            _ => {}
        }
    }
}

#[cfg(target_os = "macos")]
pub use imp::{mute_outputs, restore_outputs};

#[cfg(not(target_os = "macos"))]
pub fn mute_outputs() {}
#[cfg(not(target_os = "macos"))]
pub fn restore_outputs() {}
