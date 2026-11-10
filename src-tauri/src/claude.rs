use std::thread;
use std::time::{Duration, Instant};

use tauri::Manager;

use crate::paths;

#[cfg_attr(windows, path = "claude_windows.rs")]
#[cfg_attr(not(windows), path = "claude_unix.rs")]
mod platform;

pub use platform::*;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub canonical: bool,
}

pub fn is_running() -> bool {
    scan()
        .map(|processes| !processes.is_empty())
        .unwrap_or(false)
}

pub fn theme_is_dark() -> bool {
    if let Ok(value) = std::env::var("CLAUDE_ACCOUNT_SWITCH_THEME") {
        match value.as_str() {
            "dark" => return true,
            "light" => return false,
            _ => {}
        }
    }

    if let Some(path) = paths::detect_live_data() {
        let config = path.join("config.json");
        if let Ok(text) = std::fs::read_to_string(config) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(mode) = value.get("userThemeMode").and_then(|mode| mode.as_str()) {
                    if mode.eq_ignore_ascii_case("dark") {
                        return true;
                    }
                    if mode.eq_ignore_ascii_case("light") {
                        return false;
                    }
                }
            }
        }
    }

    system_is_dark()
}

pub fn show_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub(crate) fn wait_until_closed(timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if scan()
            .map(|processes| processes.is_empty())
            .unwrap_or(false)
        {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(150));
    }
}

pub(crate) fn parse_user_data_dir(command_line: Option<&str>) -> Option<String> {
    let command_line = command_line?;
    let lower = command_line.to_lowercase();
    let marker = "--user-data-dir";
    let index = lower.find(marker)?;
    let rest = &command_line[index + marker.len()..];
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();

    if let Some(stripped) = rest.strip_prefix('"') {
        return stripped.split('"').next().map(|value| value.to_string());
    }
    if let Some(stripped) = rest.strip_prefix('\'') {
        return stripped.split('\'').next().map(|value| value.to_string());
    }

    rest.split_whitespace()
        .next()
        .map(|value| value.to_string())
}
