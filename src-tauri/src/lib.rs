use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};
use tauri_plugin_log::{Target, TargetKind, RotationStrategy};

mod analytics;
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
mod paths;
mod recording;
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
    // tauri-plugin-aptabase's setup spawns its flush loop with a bare
    // `tokio::spawn`, which panics ("there is no reactor running") because Tauri
    // runs plugin setup on the main thread, outside any Tokio runtime context.
    // Enter a multi-threaded Tokio runtime for the whole app lifetime so that
    // spawn finds a reactor. Tauri's own async_runtime is unaffected (it manages
    // its own handle); command-context spawns keep running on Tauri's workers.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");
    let _runtime_guard = runtime.enter();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .target(Target::new(TargetKind::LogDir {
                    file_name: Some("onnda".to_string()),
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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_aptabase::Builder::new(analytics::app_key()).build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::start_recording,
            commands::stop_and_transcribe,
            recording::transcribe_file,
            commands::is_recording_cmd,
            commands::warm_apple_engine,
            models::get_models,
            models::download_model,
            models::check_model_status,
            commands::check_mic_permission,
            commands::check_accessibility_permission,
            commands::request_accessibility,
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
            commands::track_event,
            commands::log_frontend,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                    // Ventana oculta → salir del Dock (modo Accessory = solo barra de menú).
                    #[cfg(target_os = "macos")]
                    let _ = window.app_handle().set_activation_policy(tauri::ActivationPolicy::Accessory);
                }
                // Widget window is closed/hidden by the widget itself
            }
        })
        .setup(|app| {
            // Detectar si arrancamos por login item (tauri-plugin-autostart pasa "--hidden").
            // Task 6 reutilizará esta variable para la política de Dock dinámica.
            let launched_hidden = std::env::args().any(|a| a == "--hidden");

            // Si arrancamos por login item, ocultamos la ventana principal:
            // la app vive solo en la barra de menú hasta que el usuario la abra.
            if launched_hidden {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }

            settings::init(app.handle())?;
            history::init(app.handle());
            setup_tray(app.handle())?;
            shortcut::register(app.handle())?;
            escape::install(app.handle()); // Escape cancels an in-progress recording

            mic_permission::request_if_needed();
            analytics::track(app.handle(), "app_launched", None);

            // Migración one-time Intel: pasa el default histórico `small` al más
            // rápido `base-q5_1` (CPU). Corre en background (descarga 57 MB si
            // hace falta) para no bloquear el arranque ni interrumpir a `small`.
            #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
            {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    settings::migrate_intel_to_base_q5(&handle).await;
                });
            }

            #[cfg(target_os = "macos")]
            {
                // Política inicial: Regular (visible en Dock) si la ventana arranca visible;
                // Accessory (solo barra de menú) si arrancamos ocultos por login item.
                let policy = if launched_hidden {
                    tauri::ActivationPolicy::Accessory
                } else {
                    tauri::ActivationPolicy::Regular
                };
                app.set_activation_policy(policy);

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
        format!("onnda v{}", app.package_info().version),
        false,
        None::<&str>,
    )?;
    let show_settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&version_label, &sep, &show_settings, &sep2, &quit])?;

    let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray_idle.png"))
        .expect("failed to load tray icon");

    TrayIconBuilder::with_id("main-tray")
        .icon(tray_icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("onnda")
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
    // Ventana visible → aparecer en el Dock (modo Regular). Se hace ANTES de show()/set_focus():
    // una app Accessory no puede traer su ventana al frente de forma fiable, así que primero
    // pasamos a Regular y luego mostramos/enfocamos para que la ventana suba bien.
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let _ = tauri::WebviewWindowBuilder::new(
            app,
            "main",
            tauri::WebviewUrl::App("/".into()),
        )
        .title("onnda")
        .inner_size(880.0, 640.0)
        .resizable(true)
        .center()
        .build();
    }
}
