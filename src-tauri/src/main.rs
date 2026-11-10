#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod claude;
#[cfg(not(windows))]
#[path = "claude_unix.rs"]
mod claude;
mod commands;
mod i18n;
mod paths;
mod settings;
mod switcher;
mod tray;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            claude::show_window(app);
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::add_account,
            commands::switch_account,
            commands::rename_account,
            commands::delete_account,
            commands::close_claude,
            commands::open_active,
            commands::open_folder,
            commands::set_language,
            commands::window_minimize,
            commands::window_toggle_maximize,
            commands::window_is_maximized,
            commands::window_close,
            commands::start_resize_drag,
        ])
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

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Clauwitch");
}
