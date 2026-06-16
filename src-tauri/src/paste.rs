//! Clipboard + CGEvent paste helpers.
//!
//! `paste_text` writes a UTF-8 string to the macOS clipboard via NSPasteboard
//! and then simulates Cmd+V through CoreGraphics so the foreground app receives
//! the paste without Voz Local holding keyboard focus.

/// Check whether Accessibility permission is granted (macOS only).
/// Exposed as `pub(crate)` so `commands.rs` (test_paste, check_accessibility_permission)
/// can call it without duplicating the extern declaration.
#[cfg(target_os = "macos")]
pub(crate) fn ax_is_trusted() -> bool {
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}

/// Like `ax_is_trusted`, but passes the prompt option so macOS **registers this
/// process in the Accessibility list** (making the toggle appear) and shows the
/// system permission dialog when not yet trusted. Returns the current trust
/// state. Without this, just opening the Settings pane leaves the app absent
/// from the list — the user has nothing to toggle.
#[cfg(target_os = "macos")]
pub(crate) fn prompt_ax_trust() -> bool {
    use std::ffi::c_void;

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFDictionaryCreate(
            allocator: *const c_void,
            keys: *const *const c_void,
            values: *const *const c_void,
            num_values: isize,
            key_callbacks: *const c_void,
            value_callbacks: *const c_void,
        ) -> *const c_void;
        fn CFRelease(cf: *mut c_void);
        static kCFBooleanTrue: *const c_void;
        static kCFTypeDictionaryKeyCallBacks: c_void;
        static kCFTypeDictionaryValueCallBacks: c_void;
    }
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        static kAXTrustedCheckOptionPrompt: *const c_void;
        fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
    }

    unsafe {
        let keys: [*const c_void; 1] = [kAXTrustedCheckOptionPrompt];
        let values: [*const c_void; 1] = [kCFBooleanTrue];
        let opts = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks as *const c_void,
            &kCFTypeDictionaryValueCallBacks as *const c_void,
        );
        let trusted = AXIsProcessTrustedWithOptions(opts);
        if !opts.is_null() {
            CFRelease(opts as *mut c_void);
        }
        trusted
    }
}

/// Write `text` to the clipboard and simulate Cmd+V in the foreground app.
/// No-op on non-macOS builds (paste is macOS-only at this time).
pub(crate) fn paste_text(text: &str) {
    #[cfg(target_os = "macos")]
    {
        // 1. Write to clipboard using NSPasteboard directly — avoids pbcopy's locale
        //    encoding issues (pbcopy can mangle UTF-8 when LANG is not set in the env).
        unsafe { write_clipboard_utf8(text) };

        // 2. Simulate Cmd+V via CoreGraphics CGEventPost.
        //    Requires Accessibility permission.
        unsafe { post_cmd_v() };
    }
    // Suppress unused-variable warning on non-macOS.
    #[cfg(not(target_os = "macos"))]
    let _ = text;
}

#[cfg(target_os = "macos")]
unsafe fn write_clipboard_utf8(text: &str) {
    use objc2::{class, msg_send, runtime::AnyObject};
    use objc2_foundation::{NSString, ns_string};

    let pb: *mut AnyObject = msg_send![class!(NSPasteboard), generalPasteboard];
    // Clear existing clipboard content
    let _: i64 = msg_send![pb, clearContents];
    // Create NSString from Rust &str (always UTF-8 → Unicode)
    let ns_str = NSString::from_str(text);
    // Store as public.utf8-plain-text
    let pb_type = ns_string!("public.utf8-plain-text");
    let _: bool = msg_send![pb, setString: &*ns_str, forType: pb_type];
}

#[cfg(target_os = "macos")]
unsafe fn post_cmd_v() {
    use std::ffi::c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceCreate(state_id: i32) -> *mut c_void;
        fn CGEventCreateKeyboardEvent(source: *mut c_void, virtual_key: u16, key_down: bool) -> *mut c_void;
        fn CGEventSetFlags(event: *mut c_void, flags: u64);
        fn CGEventPost(tap: i32, event: *mut c_void);
        fn CFRelease(cf: *mut c_void);
    }

    const V_KEY: u16 = 9;           // kVK_ANSI_V
    const CMD_MASK: u64 = 0x100000; // kCGEventFlagMaskCommand
    const HID_TAP: i32 = 0;         // kCGHIDEventTap
    const HID_STATE: i32 = 1;       // kCGEventSourceStateHIDSystemState

    let src = CGEventSourceCreate(HID_STATE);
    if src.is_null() { return; }

    let dn = CGEventCreateKeyboardEvent(src, V_KEY, true);
    if !dn.is_null() { CGEventSetFlags(dn, CMD_MASK); CGEventPost(HID_TAP, dn); CFRelease(dn); }

    let up = CGEventCreateKeyboardEvent(src, V_KEY, false);
    if !up.is_null() { CGEventSetFlags(up, CMD_MASK); CGEventPost(HID_TAP, up); CFRelease(up); }

    CFRelease(src);
}
