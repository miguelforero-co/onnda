//! macOS notch integration for the widget window.
//!
//! Two jobs: (1) read the real notch geometry from `NSScreen` so the widget can
//! align with it, and (2) raise the widget above the menu bar — Tauri's
//! `set_always_on_top` only reaches the floating level (3), which sits *below*
//! the menu bar (24). We push it to `NSMainMenuWindowLevel + 1` via objc2.

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy)]
pub struct NotchInfo {
    /// Screen width in logical points.
    pub screen_w: f64,
    /// Notch height (safe-area top inset) in points; 0 when there is no notch.
    pub notch_height: f64,
    /// Width of the notch cutout in points; 0 when there is no notch.
    pub notch_width: f64,
    /// Whether this screen physically has a notch.
    pub has_notch: bool,
}

#[cfg(target_os = "macos")]
pub fn read_notch() -> Option<NotchInfo> {
    use objc2_app_kit::NSScreen;
    use objc2_foundation::MainThreadMarker;

    // NSScreen geometry must be read on the main thread.
    let mtm = MainThreadMarker::new()?;
    let screen = NSScreen::mainScreen(mtm)?;

    let frame = screen.frame();
    let screen_w = frame.size.width;

    // A non-zero top safe-area inset is the reliable "this screen has a notch"
    // signal; on screens without one it is 0.
    let notch_height = screen.safeAreaInsets().top;
    let has_notch = notch_height > 0.0;

    // The notch is the gap between the free area to its left and to its right.
    let notch_width = if has_notch {
        let left = screen.auxiliaryTopLeftArea();
        let right = screen.auxiliaryTopRightArea();
        let left_edge = left.origin.x + left.size.width;
        let right_edge = right.origin.x;
        (right_edge - left_edge).max(0.0)
    } else {
        0.0
    };

    Some(NotchInfo { screen_w, notch_height, notch_width, has_notch })
}

/// Raise the widget above the menu bar and make it visible on every Space
/// without participating in window cycling or stealing focus.
#[cfg(target_os = "macos")]
pub fn elevate_widget<R: tauri::Runtime>(widget: &tauri::WebviewWindow<R>) {
    use objc2_app_kit::{NSMainMenuWindowLevel, NSWindow, NSWindowCollectionBehavior};

    let ptr = match widget.ns_window() {
        Ok(p) if !p.is_null() => p,
        _ => return,
    };
    // Tao's NSWindow pointer is ABI-compatible with objc2-app-kit's NSWindow.
    let ns_window: &NSWindow = unsafe { &*(ptr as *const NSWindow) };

    // One above the menu bar (24) so the widget can sit over it, around the notch.
    ns_window.setLevel(NSMainMenuWindowLevel + 1);
    ns_window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );
}
