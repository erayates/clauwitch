//! Linux / macOS process handling. Uses `ps`, `kill` and `open` so no extra crates are needed.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use super::{parse_user_data_dir, wait_until_closed, ProcessInfo};
use crate::paths;

pub fn scan() -> Result<Vec<ProcessInfo>, String> {
    let output = Command::new("ps")
        .args(["-axo", "pid=,args="])
        .stderr(Stdio::null())
        .output()
        .map_err(|_| "error_unexpected".to_string())?;
    if !output.status.success() {
        return Err("error_unexpected".to_string());
    }

    let own_pid = std::process::id();
    let live = paths::detect_live_data();
    let text = String::from_utf8_lossy(&output.stdout);
    let mut processes = Vec::new();

    for line in text.lines() {
        let line = line.trim_start();
        let Some((pid, args)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let Ok(pid) = pid.parse::<u32>() else {
            continue;
        };
        if pid == own_pid || !is_desktop_claude(args) {
            continue;
        }

        let canonical = match (parse_user_data_dir(Some(args)), &live) {
            (Some(dir), Some(live_path)) => paths::same_path(Path::new(&dir), live_path),
            _ => true,
        };
        processes.push(ProcessInfo { pid, canonical });
    }

    Ok(processes)
}

pub fn close() -> Result<(), String> {
    let processes = scan().map_err(|_| "error_close_failed".to_string())?;
    if processes.is_empty() {
        return Ok(());
    }

    // Graceful first: lets Claude flush its session files.
    #[cfg(target_os = "macos")]
    let _ = Command::new("osascript")
        .args(["-e", "tell application \"Claude\" to quit"])
        .status();
    #[cfg(not(target_os = "macos"))]
    signal(&processes, "-TERM");

    // Give Claude time to flush its session files before falling back to a hard kill.
    if wait_until_closed(Duration::from_secs(10)) {
        return Ok(());
    }

    paths::log("Claude did not exit after close request; forcing it to stop");
    signal(&scan().unwrap_or_default(), "-KILL");

    if wait_until_closed(Duration::from_secs(6)) {
        Ok(())
    } else {
        Err("error_close_failed".to_string())
    }
}

fn signal(processes: &[ProcessInfo], sig: &str) {
    if processes.is_empty() {
        return;
    }
    let mut command = Command::new("kill");
    command.arg(sig);
    for process in processes {
        command.arg(process.pid.to_string());
    }
    let _ = command.stderr(Stdio::null()).status();
}

pub fn try_focus() -> bool {
    if !scan()
        .unwrap_or_default()
        .iter()
        .any(|process| process.canonical)
    {
        return false;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("osascript")
            .args(["-e", "tell application \"Claude\" to activate"])
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Electron single-instance lock: launching again focuses the running window.
        launch().is_ok()
    }
}

pub fn launch() -> Result<(), String> {
    let Some(launcher) = paths::detect_launcher() else {
        return Err("error_launcher_missing".to_string());
    };

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg("-a").arg(launcher);
        command
    };
    #[cfg(not(target_os = "macos"))]
    let mut command = Command::new(launcher);

    command
        .current_dir(paths::app_root())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|_| "error_launch_failed".to_string())
}

pub fn apply_dark_titlebar(_window: &tauri::WebviewWindow, _dark: bool) {}

// Cowork VM service exists only on Windows.
pub fn service_is_running() -> bool {
    false
}

pub fn stop_service() -> bool {
    false
}

pub fn start_service() {}

pub fn system_is_dark() -> bool {
    #[cfg(target_os = "macos")]
    let output = Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .stderr(Stdio::null())
        .output();
    #[cfg(not(target_os = "macos"))]
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .stderr(Stdio::null())
        .output();

    output
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .to_lowercase()
                .contains("dark")
        })
        .unwrap_or(false)
}

fn is_desktop_claude(args: &str) -> bool {
    let lower = args.to_lowercase();
    if lower.contains("claude-code") || lower.contains("clauwitch") {
        return false;
    }
    if cfg!(target_os = "macos") {
        return args.contains("/Claude.app/Contents/");
    }
    let program = args.split_whitespace().next().unwrap_or_default();
    let name = Path::new(program)
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    name == "claude-desktop"
        || lower.contains("/claude-desktop/")
        || lower.contains("/claude/claude ")
}

#[cfg(test)]
mod tests {
    use super::is_desktop_claude;

    #[test]
    fn detects_desktop_but_not_cli() {
        if cfg!(target_os = "macos") {
            assert!(is_desktop_claude(
                "/Applications/Claude.app/Contents/MacOS/Claude"
            ));
        } else {
            assert!(is_desktop_claude("/usr/bin/claude-desktop --no-sandbox"));
            assert!(is_desktop_claude(
                "/usr/lib/claude-desktop/electron app.asar"
            ));
        }
        assert!(!is_desktop_claude(
            "node /usr/lib/node_modules/@anthropic-ai/claude-code/cli.js"
        ));
    }
}
