#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod claude;
mod commands;
mod i18n;
mod paths;
mod settings;
mod switcher;
mod tray;

use tauri::Manager;

/// Passed by the OS login item; the app then starts hidden in the tray.
const AUTOSTART_ARG: &str = "--autostart";

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            claude::show_window(app);
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_ARG]),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::add_account,
            commands::switch_account,
            commands::rename_account,
            commands::delete_account,
            commands::close_claude,
            commands::open_active,
            commands::open_folder,
            commands::export_account,
            commands::import_account,
            commands::update_preferences,
            commands::get_autostart,
            commands::set_autostart,
            commands::reset_window,
            commands::open_app_path,
            commands::app_version,
            commands::set_language,
            commands::window_minimize,
            commands::window_toggle_maximize,
            commands::window_is_maximized,
            commands::window_close,
            commands::start_resize_drag,
        ])
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if commands::is_busy() {
                    api.prevent_close();
                }
            }
        })
        .setup(|app| {
            paths::migrate_legacy_data();
            let _ = paths::ensure_dirs();
            let _ = switcher::recover();
            paths::cleanup_trash();

            let settings = settings::Settings::load();
            let handle = app.handle().clone();
            tray::setup(&handle, &settings)?;

            let state = commands::build_state();
            commands::apply_window_chrome(&handle, &state);

            let autostarted = std::env::args().any(|arg| arg == AUTOSTART_ARG);
            if let Some(window) = app.get_webview_window("main") {
                if autostarted {
                    let _ = window.hide();
                } else {
                    let _ = window.unminimize();
                    let _ = window.show();
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Clauwitch");
}
