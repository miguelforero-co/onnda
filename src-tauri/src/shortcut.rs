use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

fn register_shortcut<R: Runtime>(app: &AppHandle<R>, shortcut_str: &str) -> tauri::Result<()> {
    let app_press = app.clone();
    let app_release = app.clone();

    app.global_shortcut()
        .on_shortcut(shortcut_str, move |_app, _shortcut, event| {
            let settings = crate::settings::load(&app_press);

            match event.state {
                ShortcutState::Pressed => {
                    if settings.push_to_talk {
                        // Push-to-talk: start recording on press
                        show_widget(&app_press);
                        if let Err(e) = crate::commands::start_recording_internal(&app_press) {
                            eprintln!("[shortcut] start_recording error: {e}");
                        }
                    } else {
                        // Toggle mode
                        if crate::commands::is_recording() {
                            let app = app_press.clone();
                            tauri::async_runtime::spawn(async move {
                                crate::commands::stop_and_transcribe_internal(app).await;
                            });
                        } else {
                            show_widget(&app_press);
                            if let Err(e) = crate::commands::start_recording_internal(&app_press) {
                                eprintln!("[shortcut] start_recording error: {e}");
                            }
                        }
                    }
                }
                ShortcutState::Released => {
                    if settings.push_to_talk && crate::commands::is_recording() {
                        let app = app_release.clone();
                        tauri::async_runtime::spawn(async move {
                            crate::commands::stop_and_transcribe_internal(app).await;
                        });
                    }
                }
            }
        })
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!("{}", e)))
}

pub fn register<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let shortcut = crate::settings::load(app).shortcut;
    register_shortcut(app, &shortcut)
}

pub fn re_register<R: Runtime>(app: &AppHandle<R>, new_shortcut: &str) -> tauri::Result<()> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!("{}", e)))?;
    register_shortcut(app, new_shortcut)
}

fn show_widget<R: Runtime>(app: &AppHandle<R>) {
    if let Some(widget) = app.get_webview_window("widget") {
        // Show on the screen the user is working on — the one under the cursor.
        // (Tracking text focus across apps is unreliable; the mouse is the proxy
        // every menu-bar/notch app uses.) Fall back to the primary monitor.
        let pos_xy: Option<(i32, i32)> = pick_monitor_under_cursor(app, &widget).map(|monitor| {
            let scale = monitor.scale_factor();
            let pos = monitor.position(); // physical, global coords
            let size = monitor.size(); // physical
            let widget_w_phys = 300.0 * scale;
            // Top-center of that monitor, flush with its top edge so the widget
            // sits in (or simulates) the notch.
            let x = pos.x as f64 + (size.width as f64 - widget_w_phys) / 2.0;
            let y = pos.y as f64;
            (x.round() as i32, y.round() as i32)
        });

        let set_pos = |x: i32, y: i32| {
            widget
                .set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(x, y)))
                .ok();
        };

        // Position once while hidden (best effort), show, then position AGAIN:
        // positioning a hidden window on macOS lags a cycle (tauri), so the
        // post-show call is the one that reliably lands it on the right screen.
        if let Some((x, y)) = pos_xy {
            set_pos(x, y);
        }
        // Re-assert the window level every time — macOS can drop it across
        // Spaces/fullscreen transitions (tauri#5566).
        #[cfg(target_os = "macos")]
        crate::notch::elevate_widget(&widget);
        widget.show().ok();
        if let Some((x, y)) = pos_xy {
            set_pos(x, y);
        }
        // Do NOT steal focus — user is in another app
    }
}

/// The monitor whose bounds contain the mouse cursor, or the primary monitor.
fn pick_monitor_under_cursor<R: Runtime>(
    app: &AppHandle<R>,
    widget: &tauri::WebviewWindow<R>,
) -> Option<tauri::Monitor> {
    if let (Ok(cursor), Ok(monitors)) = (app.cursor_position(), widget.available_monitors()) {
        let hit = monitors.into_iter().find(|m| {
            let p = m.position();
            let s = m.size();
            let (left, top) = (p.x as f64, p.y as f64);
            let (right, bottom) = (left + s.width as f64, top + s.height as f64);
            cursor.x >= left && cursor.x < right && cursor.y >= top && cursor.y < bottom
        });
        if hit.is_some() {
            return hit;
        }
    }
    widget.primary_monitor().ok().flatten()
}
