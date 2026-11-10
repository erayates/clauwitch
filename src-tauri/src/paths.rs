use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const TICKS_AT_UNIX_EPOCH: i64 = 621_355_968_000_000_000;

#[cfg(windows)]
pub fn local_app_data() -> PathBuf {
    PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default())
}

#[cfg(windows)]
pub fn roaming_app_data() -> PathBuf {
    PathBuf::from(std::env::var_os("APPDATA").unwrap_or_default())
}

#[cfg(not(windows))]
fn home() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
}

#[cfg(not(windows))]
fn xdg(var: &str, fallback: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| home().join(fallback))
}

#[cfg(windows)]
fn data_base() -> PathBuf {
    local_app_data()
}

#[cfg(target_os = "macos")]
fn data_base() -> PathBuf {
    home().join("Library").join("Application Support")
}

#[cfg(all(unix, not(target_os = "macos")))]
fn data_base() -> PathBuf {
    xdg("XDG_DATA_HOME", ".local/share")
}

pub fn app_root() -> PathBuf {
    data_base().join("ClauwitchData")
}

pub fn legacy_app_root() -> PathBuf {
    data_base().join("ClaudeAccountSwitch")
}

pub fn migrate_legacy_data() {
    let old = legacy_app_root();
    let new = app_root();
    if !old.exists() {
        return;
    }

    let has_new_settings = new.join("settings.ini").is_file();
    if new.exists() && has_new_settings {
        return;
    }

    if move_directory(&old, &new).is_err() {
        if fs::create_dir_all(&new).is_err() {
            return;
        }
        if copy_dir_all(&old, &new).is_err() {
            return;
        }
    }

    let settings = new.join("settings.ini");
    if let Ok(text) = fs::read_to_string(&settings) {
        let old_text = old.to_string_lossy();
        let new_text = new.to_string_lossy();
        let replaced = text.replace(old_text.as_ref(), new_text.as_ref());
        if replaced != text {
            let _ = fs::write(&settings, replaced);
        }
    }
}

fn copy_dir_all(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_all(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

pub fn profiles_root() -> PathBuf {
    app_root().join("profiles")
}

pub fn default_store() -> PathBuf {
    app_root().join("default-data")
}

pub fn trash_root() -> PathBuf {
    app_root().join("trash")
}

pub fn settings_path() -> PathBuf {
    app_root().join("settings.ini")
}

pub fn state_path() -> PathBuf {
    app_root().join("state.ini")
}

pub fn ensure_dirs() -> io::Result<()> {
    fs::create_dir_all(app_root())?;
    fs::create_dir_all(profiles_root())?;
    fs::create_dir_all(trash_root())?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn detect_live_data() -> Option<PathBuf> {
    Some(data_base().join("Claude")).filter(|path| path.is_dir())
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn detect_live_data() -> Option<PathBuf> {
    Some(xdg("XDG_CONFIG_HOME", ".config").join("Claude")).filter(|path| path.is_dir())
}

/// macOS: path to `Claude.app` (launched with `open -a`).
#[cfg(target_os = "macos")]
pub fn detect_launcher() -> Option<PathBuf> {
    [
        PathBuf::from("/Applications/Claude.app"),
        home().join("Applications").join("Claude.app"),
    ]
    .into_iter()
    .find(|path| path.is_dir())
}

/// Linux: no official build; community packages install `claude-desktop` on PATH.
#[cfg(all(unix, not(target_os = "macos")))]
pub fn detect_launcher() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join("claude-desktop"))
        .find(|candidate| candidate.is_file())
}

#[cfg(windows)]
pub fn detect_live_data() -> Option<PathBuf> {
    let data_root = local_app_data().join("Claude-Data");
    if data_root.is_dir() {
        let mut candidates: Vec<(SystemTime, PathBuf)> = Vec::new();
        let mut fallback: Option<PathBuf> = None;

        if let Ok(entries) = fs::read_dir(&data_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if !path.is_dir() || !name.starts_with("claude_") {
                    continue;
                }
                if fallback.is_none() {
                    fallback = Some(path.clone());
                }
                if path.join("config.json").is_file() {
                    let stamp = fs::metadata(path.join("config.json"))
                        .and_then(|m| m.modified())
                        .unwrap_or(UNIX_EPOCH);
                    candidates.push((stamp, path));
                }
            }
        }

        candidates.sort_by_key(|candidate| std::cmp::Reverse(candidate.0));
        if let Some((_, path)) = candidates.into_iter().next() {
            return Some(path);
        }
        if fallback.is_some() {
            return fallback;
        }
    }

    let roaming = roaming_app_data().join("Claude");
    if roaming.is_dir() {
        return Some(roaming);
    }

    let packages = local_app_data().join("Packages");
    if let Ok(entries) = fs::read_dir(packages) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("Claude_") {
                continue;
            }
            let candidate = entry
                .path()
                .join("LocalCache")
                .join("Roaming")
                .join("Claude");
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
    }

    None
}

#[cfg(windows)]
pub fn detect_launcher() -> Option<PathBuf> {
    let windows_apps = local_app_data().join("Microsoft").join("WindowsApps");
    let alias = windows_apps.join("claude-desktop.exe");
    if alias.is_file() {
        return Some(alias);
    }

    if let Ok(entries) = fs::read_dir(&windows_apps) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.starts_with("claude-desktop") && name.ends_with(".exe") {
                let path = entry.path();
                if path.is_file() {
                    return Some(path);
                }
            }
        }
    }

    let program_files = std::env::var_os("ProgramFiles")
        .map(PathBuf::from)
        .unwrap_or_default();
    let program_files_x86 = std::env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_default();

    let dirs = [
        local_app_data().join("AnthropicClaude"),
        local_app_data().join("Programs").join("Claude"),
        program_files.join("Claude"),
        program_files_x86.join("Claude"),
    ];

    for dir in dirs {
        if !dir.is_dir() {
            continue;
        }
        let direct = dir.join("Claude.exe");
        if direct.is_file() {
            return Some(direct);
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let candidate = entry.path().join("Claude.exe");
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

pub fn normalize(path: &Path) -> String {
    let resolved = dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let text = resolved
        .to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .to_string();
    // Linux filesystems are case-sensitive; Windows/macOS defaults are not.
    if cfg!(all(unix, not(target_os = "macos"))) {
        text
    } else {
        text.to_lowercase()
    }
}

pub fn same_path(first: &Path, second: &Path) -> bool {
    normalize(first) == normalize(second)
}

pub fn move_directory(source: &Path, destination: &Path) -> io::Result<()> {
    let mut last_error: Option<io::Error> = None;

    for _ in 0..12 {
        match fs::rename(source, destination) {
            Ok(()) => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                thread::sleep(Duration::from_millis(300));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| io::Error::other("folder move failed")))
}

pub fn try_move_directory(source: &Path, destination: &Path) -> bool {
    move_directory(source, destination).is_ok()
}

pub fn move_to_trash(folder: &Path) -> Option<PathBuf> {
    if !folder.is_dir() {
        return None;
    }

    fs::create_dir_all(trash_root()).ok()?;
    let name = folder
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "profile".to_string());

    let stamp = chrono_like_timestamp();
    let mut destination = trash_root().join(format!("{name}-{stamp}"));
    let mut counter = 2;
    while destination.exists() {
        destination = trash_root().join(format!("{name}-{stamp}-{counter}"));
        counter += 1;
    }

    move_directory(folder, &destination).ok()?;
    Some(destination)
}

pub fn cleanup_trash() {
    let trash = trash_root();
    let Ok(entries) = fs::read_dir(trash) else {
        return;
    };
    let cutoff = SystemTime::now() - Duration::from_secs(7 * 24 * 60 * 60);

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let modified = fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .unwrap_or(SystemTime::now());
        if modified < cutoff {
            let _ = fs::remove_dir_all(path);
        }
    }
}

pub fn now_ticks() -> i64 {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    TICKS_AT_UNIX_EPOCH + (since_epoch.as_nanos() / 100) as i64
}

pub fn ticks_to_ms(ticks: i64) -> i64 {
    if ticks <= 0 {
        return 0;
    }
    (ticks - TICKS_AT_UNIX_EPOCH) / 10_000
}

fn chrono_like_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let days = seconds.div_euclid(86_400);
    let day_seconds = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}{month:02}{day:02}-{:02}{:02}{:02}",
        day_seconds / 3600,
        (day_seconds % 3600) / 60,
        day_seconds % 60
    )
}

fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}
