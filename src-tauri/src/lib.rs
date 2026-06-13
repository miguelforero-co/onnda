use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};

mod audio;
mod backend;
mod commands;
mod history;
mod mic_permission;
mod settings;
mod shortcut;
mod streaming;
mod transcription;
mod vad;
mod whisper_backend;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::start_recording,
            commands::stop_and_transcribe,
            commands::is_recording_cmd,
            commands::get_models,
            commands::download_model,
            commands::check_mic_permission,
            commands::check_accessibility_permission,
            commands::open_accessibility_settings,
            commands::open_microphone_settings,
            commands::get_history,
            commands::delete_history_entry,
            commands::get_recording_audio,
            commands::hide_widget,
            commands::test_paste,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
                // Widget window is closed/hidden by the widget itself
            }
        })
        .setup(|app| {
            settings::init(app.handle())?;
            history::init(app.handle());
            setup_tray(app.handle())?;
            shortcut::register(app.handle())?;

            mic_permission::request_if_needed();

            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);

                if let Some(widget) = app.get_webview_window("widget") {
                    // Apply native frosted-glass vibrancy.
                    // macOSPrivateApi: true (in tauri.conf.json) enables transparent windows;
                    // wry already sets drawsBackground=false when transparent:true is set,
                    // so no extra objc2 calls needed.
                    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
                    // radius = 26 → perfect pill for 52px tall window (height/2)
                    apply_vibrancy(&widget, NSVisualEffectMaterial::HudWindow, None, Some(26.0))
                        .unwrap_or_else(|e| eprintln!("[vibrancy] {e}"));
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let version_label = MenuItem::with_id(
        app,
        "version",
        format!("Voz Local v{}", app.package_info().version),
        false,
        None::<&str>,
    )?;
    let show_settings = MenuItem::with_id(app, "settings", "Configuración...", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&version_label, &sep, &show_settings, &sep2, &quit])?;

    let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray_idle.png"))
        .expect("failed to load tray icon");

    TrayIconBuilder::with_id("main-tray")
        .icon(tray_icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Voz Local")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => open_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                open_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

fn open_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let _ = tauri::WebviewWindowBuilder::new(
            app,
            "main",
            tauri::WebviewUrl::App("/".into()),
        )
        .title("Voz Local")
        .inner_size(480.0, 600.0)
        .resizable(false)
        .center()
        .build();
    }
}
