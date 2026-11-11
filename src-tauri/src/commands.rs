use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
#[cfg(windows)]
use windows::Win32::Foundation::{HWND, POINT, RECT};
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetWindowRect, IsZoomed, SetWindowPos, ShowWindow, SWP_NOACTIVATE, SWP_NOZORDER,
    SW_MAXIMIZE, SW_RESTORE,
};

use crate::claude;
use crate::paths;
use crate::settings::{ProfileDto, Settings};
use crate::switcher;
use crate::tray;

static SWITCH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static BUSY: AtomicBool = AtomicBool::new(false);

/// True while an account operation is moving folders; closing the app then could leave data half-moved.
pub fn is_busy() -> bool {
    BUSY.load(Ordering::SeqCst)
}

#[derive(Serialize)]
pub struct StateDto {
    pub profiles: Vec<ProfileDto>,
    pub active_id: String,
    pub language: String,
    pub theme: String,
    pub any_running: bool,
    pub active_running: bool,
    pub live_path: Option<String>,
    pub launcher_found: bool,
    /// User preference: "auto", "light" or "dark". `theme` is the resolved value.
    pub theme_setting: String,
    pub close_to_tray: bool,
    pub data_dir: String,
    pub log_path: String,
}

pub fn build_state() -> StateDto {
    let settings = Settings::load();
    let live = paths::detect_live_data();
    let processes = claude::scan().unwrap_or_default();

    StateDto {
        profiles: settings.profiles_dto(),
        active_id: settings.active_id.clone(),
        language: settings.language.clone(),
        theme: if resolve_dark(&settings.theme) {
            "dark".to_string()
        } else {
            "light".to_string()
        },
        any_running: !processes.is_empty(),
        active_running: processes.iter().any(|process| process.canonical),
        live_path: live.map(|path| path.to_string_lossy().to_string()),
        launcher_found: paths::detect_launcher().is_some(),
        theme_setting: settings.theme.clone(),
        close_to_tray: settings.close_to_tray,
        data_dir: paths::app_root().to_string_lossy().to_string(),
        log_path: paths::log_path().to_string_lossy().to_string(),
    }
}

pub fn apply_window_chrome(app: &AppHandle, state: &StateDto) {
    if let Some(window) = app.get_webview_window("main") {
        let title = crate::i18n::app_title(&state.language);
        let _ = window.set_title(title);
        claude::apply_dark_titlebar(&window, state.theme == "dark");
    }
}

fn lock() -> MutexGuard<'static, ()> {
    SWITCH_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

async fn run_blocking<T, F>(operation: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|_| "error_unexpected".to_string())?
}

async fn mutate<F>(app: AppHandle, operation: F) -> Result<StateDto, String>
where
    F: FnOnce() -> Result<(), String> + Send + 'static,
{
    BUSY.store(true, Ordering::SeqCst);
    let result = run_blocking(move || {
        let _guard = lock();
        operation()?;
        Ok::<StateDto, String>(build_state())
    })
    .await;
    BUSY.store(false, Ordering::SeqCst);
    let state = result?;

    apply_window_chrome(&app, &state);
    let _ = tray::rebuild(&app);
    Ok(state)
}

#[tauri::command]
pub async fn get_state(app: AppHandle) -> Result<StateDto, String> {
    let state = run_blocking(|| Ok(build_state())).await?;
    apply_window_chrome(&app, &state);
    Ok(state)
}

#[tauri::command]
pub async fn add_account(app: AppHandle, name: String) -> Result<StateDto, String> {
    mutate(app, move || {
        let mut settings = Settings::load();
        switcher::add_profile(&mut settings, &name)
    })
    .await
}

#[tauri::command]
pub async fn switch_account(app: AppHandle, id: String) -> Result<StateDto, String> {
    mutate(app, move || {
        let mut settings = Settings::load();
        switcher::switch_to(&mut settings, &id, true)
    })
    .await
}

#[tauri::command]
pub async fn rename_account(app: AppHandle, id: String, name: String) -> Result<StateDto, String> {
    mutate(app, move || {
        let mut settings = Settings::load();
        let cleaned = settings.validate_name(&name, Some(&id))?;
        let Some(profile) = settings.find_mut(&id) else {
            return Err("error_unexpected".to_string());
        };
        if profile.is_default {
            return Err("error_unexpected".to_string());
        }
        profile.name = cleaned;
        settings.save()
    })
    .await
}

#[tauri::command]
pub async fn delete_account(app: AppHandle, id: String) -> Result<StateDto, String> {
    mutate(app, move || {
        let mut settings = Settings::load();
        switcher::delete_profile(&mut settings, &id)
    })
    .await
}

#[tauri::command]
pub async fn close_claude(app: AppHandle) -> Result<StateDto, String> {
    mutate(app, switcher::close_claude).await
}

#[tauri::command]
pub async fn open_active(app: AppHandle) -> Result<StateDto, String> {
    mutate(app, switcher::open_active).await
}

#[tauri::command]
pub async fn open_folder(id: String) -> Result<(), String> {
    run_blocking(move || {
        let settings = Settings::load();
        switcher::open_folder(&settings, &id)
    })
    .await
}

#[tauri::command]
pub async fn set_language(app: AppHandle, lang: String) -> Result<StateDto, String> {
    let language = if lang == "tr" { "tr" } else { "en" };

    mutate(app, move || {
        let mut settings = Settings::load();
        settings.language = language.to_string();
        settings.save()
    })
    .await
}

pub fn show_and_emit_switch(app: &AppHandle, id: String) {
    claude::show_window(app);
    let _ = app.emit("tray-switch", id);
}

#[tauri::command]
pub fn window_minimize(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "error_unexpected".to_string())?;
    window
        .minimize()
        .map_err(|_| "error_unexpected".to_string())
}

#[cfg(windows)]
#[tauri::command]
pub fn window_toggle_maximize(app: AppHandle) -> Result<bool, String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "error_unexpected".to_string())?;
    let hwnd = claude::window_hwnd(&window).ok_or_else(|| "error_unexpected".to_string())?;
    let maximized = unsafe { IsZoomed(hwnd).as_bool() };

    unsafe {
        if maximized {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        } else {
            let _ = ShowWindow(hwnd, SW_MAXIMIZE);
        }
    }

    Ok(!maximized)
}

#[cfg(windows)]
#[tauri::command]
pub fn window_is_maximized(app: AppHandle) -> Result<bool, String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "error_unexpected".to_string())?;
    let hwnd = claude::window_hwnd(&window).ok_or_else(|| "error_unexpected".to_string())?;
    Ok(unsafe { IsZoomed(hwnd).as_bool() })
}

#[tauri::command]
pub fn window_close(app: AppHandle) -> Result<(), String> {
    if Settings::load().close_to_tray {
        if let Some(window) = app.get_webview_window("main") {
            return window.hide().map_err(|_| "error_unexpected".to_string());
        }
    }
    if is_busy() {
        return Err("error_busy".to_string());
    }
    app.exit(0);
    Ok(())
}

#[cfg(windows)]
#[tauri::command]
pub fn start_resize_drag(window: tauri::WebviewWindow, direction: String) -> Result<(), String> {
    let hwnd = claude::window_hwnd(&window).ok_or_else(|| "error_unexpected".to_string())?;
    let raw_hwnd = hwnd.0 as isize;

    std::thread::spawn(move || unsafe {
        let hwnd = HWND(raw_hwnd as *mut std::ffi::c_void);
        let mut start = POINT::default();
        if GetCursorPos(&mut start).is_err() {
            return;
        }
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return;
        }
        let start_rect = rect;
        let west = direction.contains("West");
        let east = direction.contains("East");
        let north = direction.contains("North");
        let south = direction.contains("South");
        if !(west || east || north || south) {
            return;
        }

        loop {
            if GetAsyncKeyState(VK_LBUTTON.0 as i32) >= 0 {
                break;
            }
            let mut point = POINT::default();
            if GetCursorPos(&mut point).is_err() {
                break;
            }
            let dx = point.x - start.x;
            let dy = point.y - start.y;
            let mut left = start_rect.left;
            let mut top = start_rect.top;
            let mut right = start_rect.right;
            let mut bottom = start_rect.bottom;

            if west {
                left = start_rect.left + dx;
            }
            if east {
                right = start_rect.right + dx;
            }
            if north {
                top = start_rect.top + dy;
            }
            if south {
                bottom = start_rect.bottom + dy;
            }

            if right - left < 820 {
                if west {
                    left = right - 820;
                } else if east {
                    right = left + 820;
                }
            }
            if bottom - top < 600 {
                if north {
                    top = bottom - 600;
                } else if south {
                    bottom = top + 600;
                }
            }

            let _ = SetWindowPos(
                hwnd,
                None,
                left,
                top,
                right - left,
                bottom - top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            std::thread::sleep(std::time::Duration::from_millis(8));
        }
    });

    Ok(())
}

#[cfg(not(windows))]
#[tauri::command]
pub fn window_toggle_maximize(app: AppHandle) -> Result<bool, String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "error_unexpected".to_string())?;
    let maximized = window.is_maximized().unwrap_or(false);
    let result = if maximized {
        window.unmaximize()
    } else {
        window.maximize()
    };
    result.map_err(|_| "error_unexpected".to_string())?;
    Ok(!maximized)
}

#[cfg(not(windows))]
#[tauri::command]
pub fn window_is_maximized(app: AppHandle) -> Result<bool, String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "error_unexpected".to_string())?;
    window
        .is_maximized()
        .map_err(|_| "error_unexpected".to_string())
}

#[cfg(not(windows))]
#[tauri::command]
pub fn start_resize_drag(window: tauri::Window, direction: String) -> Result<(), String> {
    use tauri_runtime::ResizeDirection as D;
    let direction = match direction.as_str() {
        "North" => D::North,
        "South" => D::South,
        "East" => D::East,
        "West" => D::West,
        "NorthEast" => D::NorthEast,
        "NorthWest" => D::NorthWest,
        "SouthEast" => D::SouthEast,
        "SouthWest" => D::SouthWest,
        _ => return Err("error_unexpected".to_string()),
    };
    window
        .start_resize_dragging(direction)
        .map_err(|_| "error_unexpected".to_string())
}

fn pick_folder(app: &AppHandle) -> Option<std::path::PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .file()
        .blocking_pick_folder()
        .and_then(|path| path.into_path().ok())
}

/// Returns the backup folder path, or `None` if the user cancelled the folder picker.
#[tauri::command]
pub async fn export_account(app: AppHandle, id: String) -> Result<Option<String>, String> {
    run_blocking(move || {
        let Some(destination) = pick_folder(&app) else {
            return Ok(None);
        };
        let _guard = lock();
        let settings = Settings::load();
        switcher::export_profile(&settings, &id, &destination)
            .map(|path| Some(path.to_string_lossy().to_string()))
    })
    .await
}

#[tauri::command]
pub async fn import_account(app: AppHandle) -> Result<StateDto, String> {
    let picker = app.clone();
    let source = run_blocking(move || Ok(pick_folder(&picker))).await?;
    let Some(source) = source else {
        return run_blocking(|| Ok(build_state())).await;
    };
    mutate(app, move || {
        let mut settings = Settings::load();
        switcher::import_profile(&mut settings, &source)
    })
    .await
}

fn resolve_dark(setting: &str) -> bool {
    match setting {
        "dark" => true,
        "light" => false,
        _ => claude::theme_is_dark(),
    }
}

#[tauri::command]
pub async fn update_preferences(
    app: AppHandle,
    close_to_tray: bool,
    theme: String,
) -> Result<StateDto, String> {
    if !matches!(theme.as_str(), "auto" | "light" | "dark") {
        return Err("error_unexpected".to_string());
    }
    mutate(app, move || {
        let mut settings = Settings::load();
        settings.close_to_tray = close_to_tray;
        settings.theme = theme;
        settings.save()
    })
    .await
}

#[tauri::command]
pub fn get_autostart(app: AppHandle) -> bool {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    let launcher = app.autolaunch();
    let result = if enabled {
        launcher.enable()
    } else {
        launcher.disable()
    };
    result.map_err(|error| {
        paths::log(&format!("autostart change failed: {error}"));
        "error_unexpected".to_string()
    })?;
    Ok(launcher.is_enabled().unwrap_or(enabled))
}

/// Puts the window back at its default size in the middle of the primary screen
/// (recovers a window left on a disconnected monitor).
#[tauri::command]
pub fn reset_window(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "error_unexpected".to_string())?;
    let _ = window.unmaximize();
    window
        .set_size(tauri::LogicalSize::new(980.0, 700.0))
        .and_then(|_| window.center())
        .map_err(|_| "error_unexpected".to_string())
}

/// Opens the Clauwitch data folder (`"data"`) or its log file (`"log"`).
#[tauri::command]
pub async fn open_app_path(kind: String) -> Result<(), String> {
    run_blocking(move || {
        let path = match kind.as_str() {
            "data" => paths::app_root(),
            "log" => {
                if !paths::log_path().is_file() {
                    paths::log("log opened");
                }
                paths::log_path()
            }
            _ => return Err("error_unexpected".to_string()),
        };
        switcher::open_path(&path)
    })
    .await
}

#[tauri::command]
pub fn app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}
