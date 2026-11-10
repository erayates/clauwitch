use std::collections::HashSet;
use std::ffi::c_void;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use serde::Deserialize;
use windows::core::{w, BOOL, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE};
use windows::Win32::System::Services::{
    CloseServiceHandle, ControlService, OpenSCManagerW, OpenServiceW, QueryServiceStatus,
    StartServiceW, SC_HANDLE, SC_MANAGER_CONNECT, SERVICE_CONTROL_STOP, SERVICE_QUERY_STATUS,
    SERVICE_RUNNING, SERVICE_START, SERVICE_STATUS, SERVICE_STOP, SERVICE_STOPPED,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindow, GetWindowTextLengthW, GetWindowThreadProcessId, IsIconic,
    IsWindowVisible, PostMessageW, SetForegroundWindow, ShowWindow, GW_OWNER, SW_RESTORE, WM_CLOSE,
};
use wmi::WMIConnection;

use super::{parse_user_data_dir, wait_until_closed, ProcessInfo};
use crate::paths;

#[derive(Deserialize)]
struct ProcessRow {
    #[serde(rename = "ProcessId")]
    process_id: u32,
    #[serde(rename = "ExecutablePath")]
    executable_path: Option<String>,
    #[serde(rename = "CommandLine")]
    command_line: Option<String>,
}

pub fn scan() -> Result<Vec<ProcessInfo>, String> {
    let live = paths::detect_live_data();
    let connection = WMIConnection::new().map_err(|_| "error_unexpected".to_string())?;

    let rows: Vec<ProcessRow> = connection
        .raw_query(
            "SELECT ProcessId, ExecutablePath, CommandLine FROM Win32_Process WHERE Name LIKE 'claude%'",
        )
        .map_err(|_| "error_unexpected".to_string())?;

    let mut processes = Vec::new();
    for row in rows {
        if !is_desktop_claude(row.executable_path.as_deref(), row.command_line.as_deref()) {
            continue;
        }

        let canonical = match parse_user_data_dir(row.command_line.as_deref()) {
            Some(dir) => match &live {
                Some(live_path) => paths::same_path(Path::new(&dir), live_path),
                None => true,
            },
            None => true,
        };

        processes.push(ProcessInfo {
            pid: row.process_id,
            canonical,
        });
    }

    Ok(processes)
}

pub fn close() -> Result<(), String> {
    let processes = scan().map_err(|_| "error_close_failed".to_string())?;
    if processes.is_empty() {
        return Ok(());
    }

    let pids: HashSet<u32> = processes.iter().map(|process| process.pid).collect();
    for (pid, hwnd) in visible_windows() {
        if pids.contains(&pid) {
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
    }

    if wait_until_closed(Duration::from_millis(5000)) {
        return Ok(());
    }

    let remaining: Vec<u32> = scan()
        .unwrap_or_default()
        .into_iter()
        .map(|process| process.pid)
        .collect();

    for pid in remaining {
        let mut command = Command::new("taskkill");
        command.args(["/PID", &pid.to_string(), "/T", "/F"]);
        hide_console(&mut command);
        let _ = command.status();
    }

    if wait_until_closed(Duration::from_secs(6)) {
        Ok(())
    } else {
        Err("error_close_failed".to_string())
    }
}

pub fn try_focus() -> bool {
    let canonical: HashSet<u32> = scan()
        .unwrap_or_default()
        .into_iter()
        .filter(|process| process.canonical)
        .map(|process| process.pid)
        .collect();

    if canonical.is_empty() {
        return false;
    }

    let Some(hwnd) = visible_windows()
        .into_iter()
        .find(|(pid, _)| canonical.contains(pid))
        .map(|(_, hwnd)| hwnd)
    else {
        return false;
    };

    unsafe {
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = SetForegroundWindow(hwnd);
    }
    true
}

pub fn launch() -> Result<(), String> {
    let Some(launcher) = paths::detect_launcher() else {
        return Err("error_launcher_missing".to_string());
    };

    let mut command = Command::new(launcher);
    command.current_dir(paths::app_root());
    hide_console(&mut command);
    command
        .spawn()
        .map(|_| ())
        .map_err(|_| "error_launch_failed".to_string())
}

pub fn apply_dark_titlebar(window: &tauri::WebviewWindow, dark: bool) {
    let Some(hwnd) = window_hwnd(window) else {
        return;
    };
    let value: i32 = if dark { 1 } else { 0 };
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &value as *const i32 as *const c_void,
            std::mem::size_of::<i32>() as u32,
        );
    }
}

pub fn service_is_running() -> bool {
    query_service_state() == Some(SERVICE_RUNNING.0)
}

pub fn stop_service() -> bool {
    let Some((manager, service)) = open_service(SERVICE_STOP | SERVICE_QUERY_STATUS) else {
        return false;
    };

    let mut status = SERVICE_STATUS::default();
    unsafe {
        if QueryServiceStatus(service, &mut status).is_err() {
            close_handles(service, manager);
            return false;
        }
    }

    if status.dwCurrentState == SERVICE_STOPPED {
        close_handles(service, manager);
        return true;
    }

    unsafe {
        if ControlService(service, SERVICE_CONTROL_STOP, &mut status).is_err() {
            close_handles(service, manager);
            return false;
        }
    }

    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let current = query_service_state_with(service);
        if current == Some(SERVICE_STOPPED.0) {
            close_handles(service, manager);
            return true;
        }
        if Instant::now() >= deadline {
            close_handles(service, manager);
            return false;
        }
        thread::sleep(Duration::from_millis(200));
    }
}

pub fn start_service() {
    let Some((manager, service)) = open_service(SERVICE_START | SERVICE_QUERY_STATUS) else {
        return;
    };

    let current = query_service_state_with(service);
    if current == Some(SERVICE_RUNNING.0) {
        close_handles(service, manager);
        return;
    }

    unsafe {
        let _ = StartServiceW(service, None);
    }

    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if query_service_state_with(service) == Some(SERVICE_RUNNING.0) {
            break;
        }
        if Instant::now() >= deadline {
            break;
        }
        thread::sleep(Duration::from_millis(200));
    }

    close_handles(service, manager);
}

pub fn window_hwnd(window: &tauri::WebviewWindow) -> Option<HWND> {
    let handle = window.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(win32) => Some(HWND(win32.hwnd.get() as *mut c_void)),
        _ => None,
    }
}

fn visible_windows() -> Vec<(u32, HWND)> {
    let mut windows: Vec<(u32, HWND)> = Vec::new();
    let lparam = LPARAM(&mut windows as *mut _ as isize);

    unsafe {
        let _ = EnumWindows(Some(enum_window), lparam);
    }

    windows
}

unsafe extern "system" fn enum_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = unsafe { &mut *(lparam.0 as *mut Vec<(u32, HWND)>) };
    let mut pid = 0u32;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }

    let visible = unsafe { IsWindowVisible(hwnd).as_bool() };
    let owner_none = unsafe { GetWindow(hwnd, GW_OWNER) }
        .map(|owner| owner.0.is_null())
        .unwrap_or(false);
    let title_length = unsafe { GetWindowTextLengthW(hwnd) };

    if visible && owner_none && title_length > 0 {
        windows.push((pid, hwnd));
    }

    BOOL(1)
}

fn query_service_state() -> Option<u32> {
    let (manager, service) = open_service(SERVICE_QUERY_STATUS)?;
    let state = unsafe {
        let mut status = SERVICE_STATUS::default();
        if QueryServiceStatus(service, &mut status).is_ok() {
            Some(status.dwCurrentState.0)
        } else {
            None
        }
    };
    close_handles(service, manager);
    state
}

fn query_service_state_with(service: SC_HANDLE) -> Option<u32> {
    unsafe {
        let mut status = SERVICE_STATUS::default();
        if QueryServiceStatus(service, &mut status).is_ok() {
            Some(status.dwCurrentState.0)
        } else {
            None
        }
    }
}

fn open_service(rights: u32) -> Option<(SC_HANDLE, SC_HANDLE)> {
    unsafe {
        let manager = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT).ok()?;
        match OpenServiceW(manager, w!("CoworkVMService"), rights) {
            Ok(service) => Some((manager, service)),
            Err(_) => {
                let _ = CloseServiceHandle(manager);
                None
            }
        }
    }
}

fn close_handles(service: SC_HANDLE, manager: SC_HANDLE) {
    unsafe {
        let _ = CloseServiceHandle(service);
        let _ = CloseServiceHandle(manager);
    }
}

pub fn system_is_dark() -> bool {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    match key {
        Ok(key) => key
            .get_value::<u32, _>("AppsUseLightTheme")
            .map(|value| value == 0)
            .unwrap_or(false),
        Err(_) => false,
    }
}

fn is_desktop_claude(executable: Option<&str>, command_line: Option<&str>) -> bool {
    let exe = executable.unwrap_or_default().to_lowercase();
    let command = command_line.unwrap_or_default().to_lowercase();

    if exe.is_empty() && command.is_empty() {
        return false;
    }
    if exe.contains("\\claude-code\\") || command.contains("\\claude-code\\") {
        return false;
    }

    if exe.ends_with("\\claude.exe") || exe.ends_with("\\claude-desktop.exe") {
        if exe.contains("\\windowsapps\\claude_") {
            return true;
        }
        if exe.contains("\\microsoft\\windowsapps\\claude") {
            return true;
        }
        if let Some(launcher) = paths::detect_launcher() {
            if exe == launcher.to_string_lossy().to_lowercase() {
                return true;
            }
        }
        if exe.contains("\\anthropicclaude\\")
            || exe.contains("\\programs\\claude\\")
            || exe.contains("\\program files\\claude\\")
            || exe.contains("\\program files (x86)\\claude\\")
        {
            return true;
        }
    }

    command.contains("\\windowsapps\\claude_")
        || command.contains("\\microsoft\\windowsapps\\claude-desktop")
}

fn hide_console(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}
