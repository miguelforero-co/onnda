use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};
use tauri_plugin_log::{Target, TargetKind, RotationStrategy};

mod audio;
mod backend;
mod commands;
mod compat;
mod escape;
mod history;
mod mic_permission;
mod models;
#[cfg(target_os = "macos")]
mod notch;
mod paste;
mod settings;
mod shortcut;
mod streaming;
mod transcription;
mod vad;
mod whisper_backend;
mod speech_backend;
mod replacements;
mod learn;

// New module stubs (filled in by downstream Wave 2+ plans).
mod sounds;
mod media_pause;
mod audio_decode;
mod data_mgmt;
mod updater_check;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .target(Target::new(TargetKind::LogDir {
                    file_name: Some("voz-local".to_string()),
                }))
                .max_file_size(5_000_000)
                .rotation_strategy(RotationStrategy::KeepOne)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::start_recording,
            commands::stop_and_transcribe,
            commands::transcribe_file,
            commands::is_recording_cmd,
            models::get_models,
            models::download_model,
            models::check_model_status,
            commands::check_mic_permission,
            commands::check_accessibility_permission,
            commands::open_accessibility_settings,
            commands::open_microphone_settings,
            commands::get_history,
            commands::delete_history_entry,
            commands::correct_history_entry,
            commands::get_recording_audio,
            commands::hide_widget,
            commands::test_paste,
            commands::get_build_hash,
            commands::get_app_version,
            data_mgmt::reveal_data_dir,
            data_mgmt::clear_history,
            data_mgmt::clear_models,
            data_mgmt::get_storage_usage,
            updater_check::check_for_updates,
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
            escape::install(app.handle()); // Escape cancels an in-progress recording

            mic_permission::request_if_needed();

            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);

                if let Some(widget) = app.get_webview_window("widget") {
                    // No vibrancy: the widget draws its own opaque black notch
                    // shape that fuses with the physical notch. Raise it above
                    // the menu bar so it can live in (or simulate) the notch.
                    notch::elevate_widget(&widget);
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
        .inner_size(880.0, 640.0)
        .resizable(true)
        .center()
        .build();
    }
}
