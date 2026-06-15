use tauri::{AppHandle, Emitter, Manager, Runtime};
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
                        // Push-to-talk: start recording on press. macOS delivers
                        // repeated `Pressed` events while the key is HELD (auto-
                        // repeat); guard against re-entering start mid-recording,
                        // which glitches the session. Idempotent under key-repeat.
                        if !crate::commands::is_recording() {
                            show_widget(&app_press);
                            if let Err(e) = crate::commands::start_recording_internal(&app_press) {
                                log::error!("[shortcut] start_recording error: {e}");
                            }
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
                                log::error!("[shortcut] start_recording error: {e}");
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
        // Position natively (objc2) at the top-center of the screen under the
        // cursor — atomic and reliable even while hidden, unlike Tauri's
        // set_position which lagged a cycle. Re-assert the window level too,
        // since macOS can drop it across Space/fullscreen transitions.
        #[cfg(target_os = "macos")]
        {
            crate::notch::position_widget_at_notch(&widget);
            crate::notch::elevate_widget(&widget);
        }
        widget.show().ok();
        // Re-apply after show as a belt-and-suspenders against any reflow, and
        // tell the UI whether this screen has a real notch so it can render a
        // compact shape on external displays.
        #[cfg(target_os = "macos")]
        {
            let has_notch = crate::notch::position_widget_at_notch(&widget);
            widget.emit("screen-notch", has_notch).ok();
        }
        // Do NOT steal focus — user is in another app
    }
}
