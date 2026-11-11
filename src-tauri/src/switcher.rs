use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::claude;
use crate::paths;
use crate::settings::Settings;

pub fn recover() -> Result<(), String> {
    let mut settings = Settings::load();
    let state_path = paths::state_path();

    let mut phase = String::new();
    let mut from_id = String::new();
    let mut to_id = String::new();

    if let Ok(text) = fs::read_to_string(&state_path) {
        for raw in text.lines() {
            let line = raw.trim();
            if let Some(value) = line.strip_prefix("phase=") {
                phase = value.to_string();
            } else if let Some(value) = line.strip_prefix("from=") {
                from_id = value.to_string();
            } else if let Some(value) = line.strip_prefix("to=") {
                to_id = value.to_string();
            }
        }
    }

    if phase == "switching" {
        let from_profile = settings
            .find(&from_id)
            .cloned()
            .unwrap_or_else(|| settings.default().clone());
        let to_profile = settings.find(&to_id).cloned();

        let from_store = from_profile.stored_folder();
        let to_store = to_profile.as_ref().map(|profile| profile.stored_folder());
        let live = paths::detect_live_data();

        if let Some(live) = live {
            if !live.exists() {
                if from_store.is_dir() {
                    let _ = paths::try_move_directory(&from_store, &live);
                    settings.active_id = from_profile.id.clone();
                } else if let Some(to_store) = to_store.clone() {
                    if to_store.is_dir() {
                        let _ = paths::try_move_directory(&to_store, &live);
                        if let Some(profile) = to_profile.as_ref() {
                            settings.active_id = profile.id.clone();
                        }
                    } else {
                        let _ = fs::create_dir_all(&live);
                        settings.active_id = from_profile.id.clone();
                    }
                } else {
                    let _ = fs::create_dir_all(&live);
                    settings.active_id = from_profile.id.clone();
                }
            } else if let Some(profile) = to_profile.as_ref() {
                if profile.id != from_profile.id
                    && to_store
                        .as_ref()
                        .map(|store| !store.exists())
                        .unwrap_or(false)
                {
                    settings.active_id = profile.id.clone();
                } else {
                    settings.active_id = from_profile.id.clone();
                }
            } else {
                settings.active_id = from_profile.id.clone();
            }
        }

        clear_state();
        let _ = settings.save();
        return Ok(());
    }

    if paths::detect_live_data()
        .map(|live| !live.exists())
        .unwrap_or(true)
    {
        let live = live_data_or_err()?;
        let store = settings.active_profile().stored_folder();
        if store.is_dir() {
            let _ = paths::try_move_directory(&store, &live);
        } else {
            fs::create_dir_all(&live).map_err(|_| "error_unexpected".to_string())?;
        }
    }

    Ok(())
}

pub fn switch_to(settings: &mut Settings, target_id: &str, launch: bool) -> Result<(), String> {
    let active_id = settings.active_id.clone();
    let Some(target) = settings.find(target_id).cloned() else {
        return Err("error_switch_failed".to_string());
    };

    if target.id == active_id {
        if launch {
            if claude::is_running() {
                if !claude::try_focus() {
                    claude::launch()?;
                }
            } else {
                claude::launch()?;
                thread::sleep(Duration::from_millis(1400));
                claude::try_focus();
            }
        }
        return Ok(());
    }

    let active = settings
        .find(&active_id)
        .cloned()
        .unwrap_or_else(|| settings.default().clone());
    let live = live_data_or_err()?;

    write_state(&active.id, &target.id);

    let service_was_running = claude::service_is_running();
    let mut service_stopped = false;
    let was_running = claude::is_running();

    let result = (|| -> Result<(), String> {
        crate::commands::progress("closing");
        claude::close()?;

        if service_was_running {
            service_stopped = claude::stop_service();
        }

        if !live.exists() {
            let current_store = active.stored_folder();
            if current_store.is_dir() {
                paths::move_directory(&current_store, &live).map_err(|e| {
                    paths::folder_error("restore live", &current_store, &e, "error_switch_failed")
                })?;
            } else {
                fs::create_dir_all(&live).map_err(|_| "error_switch_failed".to_string())?;
            }
        }

        let active_store = active.stored_folder();
        if let Some(parent) = active_store.parent() {
            fs::create_dir_all(parent).map_err(|_| "error_switch_failed".to_string())?;
        }

        if active_store.is_dir() && paths::move_to_trash(&active_store).is_none() {
            return Err("error_switch_failed".to_string());
        }

        crate::commands::progress("saving");
        paths::move_directory(&live, &active_store)
            .map_err(|e| paths::folder_error("store active", &live, &e, "error_switch_failed"))?;

        crate::commands::progress("activating");
        let target_store = target.stored_folder();
        if target_store.is_dir() {
            paths::move_directory(&target_store, &live).map_err(|e| {
                paths::folder_error("activate target", &target_store, &e, "error_switch_failed")
            })?;
        } else {
            fs::create_dir_all(&live).map_err(|_| "error_switch_failed".to_string())?;
        }

        // Best effort, never fails the switch: bring each account's sessions to its own folder.
        gather_account_sessions(settings, &live);
        gather_account_sessions(settings, &active_store);

        settings.active_id = target.id.clone();
        if let Some(profile) = settings.find_mut(&target.id) {
            profile.last_used_ticks = paths::now_ticks();
        }
        settings.save()?;
        clear_state();

        if service_stopped {
            claude::start_service();
        }

        crate::commands::progress("launching");
        if launch {
            claude::launch()?;
            thread::sleep(Duration::from_millis(1400));
            claude::try_focus();
        }

        Ok(())
    })();

    if let Err(error) = result {
        paths::log(&format!(
            "switch {} -> {} failed: {error}",
            active.id, target.id
        ));
        let live_exists = live.exists();
        let active_store = active.stored_folder();
        let target_store = target.stored_folder();

        if live_exists {
            if !target_store.is_dir() && target.id != active.id {
                settings.active_id = target.id.clone();
            } else {
                settings.active_id = active.id.clone();
            }
        } else {
            if active_store.is_dir() {
                let _ = paths::try_move_directory(&active_store, &live);
            } else {
                let _ = fs::create_dir_all(&live);
            }
            settings.active_id = active.id.clone();
        }

        let _ = settings.save();
        clear_state();

        if service_stopped {
            claude::start_service();
        }

        if was_running && !claude::is_running() && launch {
            let _ = claude::launch();
        }

        return Err(error);
    }

    Ok(())
}

pub fn delete_profile(settings: &mut Settings, profile_id: &str) -> Result<(), String> {
    let Some(profile) = settings.find(profile_id).cloned() else {
        return Err("error_delete_failed".to_string());
    };
    if profile.is_default {
        return Err("error_delete_failed".to_string());
    }

    let was_running = claude::is_running();
    let was_active = settings.active_id == profile.id;

    if was_active {
        switch_to(settings, &settings.default().id.clone(), false)?;
    }

    let folder = profile.stored_folder();
    if folder.is_dir() && paths::move_to_trash(&folder).is_none() {
        return Err("error_delete_failed".to_string());
    }

    settings.profiles.retain(|item| item.id != profile.id);
    if settings.active_id == profile.id {
        settings.active_id = settings.default().id.clone();
    }
    settings.save()?;

    if was_active && was_running {
        let _ = claude::launch();
        thread::sleep(Duration::from_millis(1200));
        claude::try_focus();
    }

    Ok(())
}

pub fn add_profile(settings: &mut Settings, name: &str) -> Result<(), String> {
    let cleaned = settings.validate_name(name, None)?;
    let profile = settings.create_profile(&cleaned);
    let id = profile.id.clone();
    settings.profiles.push(profile);
    settings.save()?;
    switch_to(settings, &id, true)
}

pub fn open_active() -> Result<(), String> {
    if claude::try_focus() {
        return Ok(());
    }

    claude::launch()?;
    thread::sleep(Duration::from_millis(1200));
    claude::try_focus();
    Ok(())
}

pub fn close_claude() -> Result<(), String> {
    claude::close()
}

pub fn open_folder(settings: &Settings, profile_id: &str) -> Result<(), String> {
    let Some(profile) = settings.find(profile_id) else {
        return Err("error_folder_missing".to_string());
    };

    let folder: PathBuf = if profile.id == settings.active_id {
        live_data_or_err()?
    } else {
        profile.stored_folder()
    };

    if !folder.is_dir() {
        return Err("error_folder_missing".to_string());
    }

    open_path(&folder)
}

/// Opens a folder in the file manager, or a file with its default app.
pub fn open_path(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    let mut command = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut command = Command::new("explorer.exe");
        command.creation_flags(CREATE_NO_WINDOW);
        command
    };
    #[cfg(target_os = "macos")]
    let mut command = Command::new("open");
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = Command::new("xdg-open");

    command
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|_| "error_folder_open_failed".to_string())
}

pub fn live_data_or_err() -> Result<PathBuf, String> {
    paths::detect_live_data().ok_or_else(|| "error_live_data_missing".to_string())
}

fn write_state(from: &str, to: &str) {
    let _ = fs::create_dir_all(paths::app_root());
    let _ = fs::write(
        paths::state_path(),
        format!("phase=switching\nfrom={from}\nto={to}\n"),
    );
}

fn clear_state() {
    let _ = fs::remove_file(paths::state_path());
}

/// Copies a profile's Claude data folder into `destination` as a new backup folder.
pub fn export_profile(
    settings: &Settings,
    profile_id: &str,
    destination: &Path,
) -> Result<PathBuf, String> {
    let Some(profile) = settings.find(profile_id) else {
        return Err("error_folder_missing".to_string());
    };
    let source = if profile.id == settings.active_id {
        if claude::is_running() {
            return Err("error_close_claude_first".to_string());
        }
        live_data_or_err()?
    } else {
        profile.stored_folder()
    };
    if !source.is_dir() {
        return Err("error_folder_missing".to_string());
    }

    let name: String = profile
        .display_name(&settings.language)
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let target = destination.join(format!(
        "Clauwitch-{name}-{}",
        paths::chrono_like_timestamp()
    ));
    paths::copy_dir_all(&source, &target).map_err(|error| {
        let _ = fs::remove_dir_all(&target);
        paths::folder_error("export", &source, &error, "error_export_failed")
    })?;
    paths::log(&format!("exported {} to {}", profile.id, target.display()));
    Ok(target)
}

/// Adds a backup folder (or any Claude data folder) as a new saved account.
pub fn import_profile(settings: &mut Settings, source: &Path) -> Result<(), String> {
    if !source.join("config.json").is_file() {
        return Err("error_invalid_backup".to_string());
    }

    let base = source
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    let base = base.strip_prefix("Clauwitch-").unwrap_or(&base);
    let base: String = base.chars().take(50).collect();
    let base = if base.trim().is_empty() {
        "Imported".to_string()
    } else {
        base.trim().to_string()
    };

    let mut name = base.clone();
    let mut counter = 2;
    while settings.validate_name(&name, None).is_err() {
        name = format!("{base} {counter}");
        counter += 1;
    }

    let profile = settings.create_profile(&name);
    let target = profile.stored_folder();
    paths::copy_dir_all(source, &target).map_err(|error| {
        let _ = fs::remove_dir_all(&target);
        paths::folder_error("import", source, &error, "error_import_failed")
    })?;
    settings.profiles.push(profile);
    settings.save()?;
    paths::log(&format!("imported {} as {name}", source.display()));
    Ok(())
}

/// Claude keeps sessions per signed-in account (`<dir>/<accountUuid>/...`). Before Clauwitch
/// a user may have used several accounts in one data folder, so an account's sessions can sit
/// in another profile's folder. Copy them into `folder` (the one that account now lives in).
const SESSION_DIRS: [&str; 2] = ["claude-code-sessions", "local-agent-mode-sessions"];

pub fn gather_account_sessions(settings: &Settings, folder: &Path) {
    let Some(account) = account_uuid(folder) else {
        return;
    };
    for profile in &settings.profiles {
        let source = profile.stored_folder();
        if !source.is_dir() || paths::same_path(&source, folder) {
            continue;
        }
        for dir in SESSION_DIRS {
            let from = source.join(dir).join(&account);
            if !from.is_dir() {
                continue;
            }
            match paths::copy_newer(&from, &folder.join(dir).join(&account)) {
                Ok(0) => {}
                Ok(count) => paths::log(&format!(
                    "copied {count} {dir} files for account {account} from {}",
                    source.display()
                )),
                Err(error) => paths::log(&format!(
                    "session copy from {} failed: {error}",
                    source.display()
                )),
            }
        }
    }
}

fn account_uuid(folder: &Path) -> Option<String> {
    let text = fs::read_to_string(folder.join("config.json")).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    let uuid = value.get("lastKnownAccountUuid")?.as_str()?;
    // Used as a path component: accept only UUID characters.
    (!uuid.is_empty() && uuid.chars().all(|c| c.is_ascii_hexdigit() || c == '-'))
        .then(|| uuid.to_string())
}
