//! macOS notch integration for the widget window.
//!
//! Two jobs: (1) position the widget at the top-center of the screen under the
//! cursor and report whether that screen has a notch, and (2) raise the widget
//! above the menu bar — Tauri's `set_always_on_top` only reaches the floating
//! level (3), which sits *below* the menu bar (24). We push it to
//! `NSMainMenuWindowLevel + 1` via objc2.

/// Position the widget flush at the top-center of the screen under the cursor,
/// natively via `setFrameOrigin`. This is atomic and works even while the
/// window is hidden — unlike Tauri's `set_position`, which lags a cycle and
/// causes the widget to appear off-center or on the wrong screen.
///
/// Returns whether that screen physically has a notch, so the UI can render a
/// compact shape on external displays instead of leaving the notch-sized gap.
#[cfg(target_os = "macos")]
pub fn position_widget_at_notch<R: tauri::Runtime>(widget: &tauri::WebviewWindow<R>) -> bool {
    use objc2_app_kit::{NSEvent, NSScreen, NSWindow};
    use objc2_foundation::MainThreadMarker;

    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    let ptr = match widget.ns_window() {
        Ok(p) if !p.is_null() => p,
        _ => return false,
    };
    let ns_window: &NSWindow = unsafe { &*(ptr as *const NSWindow) };

    // Cursor location in global screen coordinates (bottom-left origin).
    let mouse = NSEvent::mouseLocation();

    // Find the screen under the cursor; fall back to the main screen. Capture
    // its frame and whether it has a notch in one pass.
    let (mut sx, mut sy, mut sw, mut sh) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
    let mut has_notch = false;
    let mut found = false;
    let screens = NSScreen::screens(mtm);
    for screen in screens.iter() {
        let f = screen.frame();
        if mouse.x >= f.origin.x
            && mouse.x < f.origin.x + f.size.width
            && mouse.y >= f.origin.y
            && mouse.y < f.origin.y + f.size.height
        {
            sx = f.origin.x;
            sy = f.origin.y;
            sw = f.size.width;
            sh = f.size.height;
            has_notch = screen.safeAreaInsets().top > 0.0;
            found = true;
            break;
        }
    }
    if !found {
        match NSScreen::mainScreen(mtm) {
            Some(s) => {
                let f = s.frame();
                sx = f.origin.x;
                sy = f.origin.y;
                sw = f.size.width;
                sh = f.size.height;
                has_notch = s.safeAreaInsets().top > 0.0;
            }
            None => return false,
        }
    }

    // setFrameOrigin places the window's bottom-left corner. Center horizontally,
    // pin the window's top edge to the screen's top edge.
    let win = ns_window.frame();
    let mut origin = win.origin;
    origin.x = sx + (sw - win.size.width) / 2.0;
    origin.y = sy + sh - win.size.height;
    ns_window.setFrameOrigin(origin);

    has_notch
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
