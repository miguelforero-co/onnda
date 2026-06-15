//! Cancel-on-Escape via a passive global NSEvent monitor.
//!
//! The global-shortcut plugin can't be used for this: registering "Escape" from
//! inside another shortcut's callback deadlocks the plugin and froze the whole
//! app. A global NSEvent monitor instead OBSERVES key events without consuming
//! them and without touching any plugin lock, so it can't hang. It fires only
//! while a recording is in progress and cancels it when Escape is pressed.
//!
//! Note: a global monitor receives events destined for OTHER apps (we never
//! hold focus while dictating) and does NOT consume them — so Escape still
//! reaches the focused app. That's an acceptable trade-off for not hanging.

#[cfg(target_os = "macos")]
pub fn install<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    use block2::RcBlock;
    use objc2_app_kit::{NSEvent, NSEventMask};
    use objc2_foundation::MainThreadMarker;
    use std::ptr::NonNull;

    if MainThreadMarker::new().is_none() {
        log::warn!("[escape] not on main thread; skipping monitor install");
        return;
    }
    let app = app.clone();

    let handler = RcBlock::new(move |event: NonNull<NSEvent>| {
        const ESCAPE: u16 = 53; // kVK_Escape
        let key_code = unsafe { event.as_ref().keyCode() };
        if key_code == ESCAPE && crate::recording::is_recording() {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                crate::recording::cancel_recording_internal(app).await;
            });
        }
    });

    // Leak the returned monitor token so the monitor stays installed for the
    // app's lifetime (AppKit keeps it until removeMonitor:, which we never call).
    let monitor =
        NSEvent::addGlobalMonitorForEventsMatchingMask_handler(NSEventMask::KeyDown, &handler);
    std::mem::forget(monitor);
}

#[cfg(not(target_os = "macos"))]
pub fn install<R: tauri::Runtime>(_app: &tauri::AppHandle<R>) {}
