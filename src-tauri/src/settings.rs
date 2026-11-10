use std::fs;
use std::path::PathBuf;

use serde::Serialize;

use crate::paths;

#[derive(Clone, Debug)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub folder: PathBuf,
    pub is_default: bool,
    pub created_ticks: i64,
    pub last_used_ticks: i64,
}

impl Profile {
    pub fn stored_folder(&self) -> PathBuf {
        if self.is_default {
            paths::default_store()
        } else {
            self.folder.clone()
        }
    }

    pub fn display_name(&self, language: &str) -> String {
        if self.is_default {
            default_account_name(language).to_string()
        } else {
            self.name.clone()
        }
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub profiles: Vec<Profile>,
    pub active_id: String,
    pub language: String,
    extra_lines: Vec<String>,
}

#[derive(Serialize)]
pub struct ProfileDto {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub folder: String,
    pub last_used_ms: Option<i64>,
}

impl Settings {
    pub fn load() -> Settings {
        let mut settings = Settings {
            profiles: Vec::new(),
            active_id: "default".to_string(),
            language: system_language(),
            extra_lines: Vec::new(),
        };

        if let Ok(text) = fs::read_to_string(paths::settings_path()) {
            for raw in text.lines() {
                let line = raw.trim_end_matches('\r');
                if let Some(value) = line.strip_prefix("active=") {
                    settings.active_id = value.trim().to_string();
                } else if let Some(value) = line.strip_prefix("lang=") {
                    let code = value.trim();
                    if code == "tr" || code == "en" {
                        settings.language = code.to_string();
                    }
                } else if line.starts_with("profile\t") {
                    if let Some(profile) = parse_profile(line) {
                        if !settings.profiles.iter().any(|item| item.id == profile.id) {
                            settings.profiles.push(profile);
                        }
                    }
                } else if !line.trim().is_empty() && !line.starts_with("v=") {
                    settings.extra_lines.push(line.to_string());
                }
            }
        }

        let default_index = settings
            .profiles
            .iter()
            .position(|profile| profile.is_default);

        if let Some(index) = default_index {
            let mut profile = settings.profiles.remove(index);
            profile.folder = paths::default_store();
            settings.profiles.insert(0, profile);
        } else {
            settings.profiles.insert(0, default_profile());
        }

        if settings.active().is_none() {
            settings.active_id = settings.default().id.clone();
        }

        settings
    }

    pub fn save(&self) -> Result<(), String> {
        paths::ensure_dirs().map_err(|_| "error_unexpected".to_string())?;

        let mut output = String::new();
        output.push_str("v=1\n");
        output.push_str(&format!("active={}\n", self.active_id));
        output.push_str(&format!("lang={}\n", self.language));

        for line in &self.extra_lines {
            output.push_str(line);
            output.push('\n');
        }

        for profile in &self.profiles {
            let created = if profile.created_ticks <= 0 {
                paths::now_ticks()
            } else {
                profile.created_ticks
            };
            let last_used = profile.last_used_ticks.max(0);
            output.push_str(&format!(
                "profile\t{}\t{}\t{}\t{}\t{}\t{}\n",
                profile.id,
                profile.name.replace('\t', " "),
                profile.stored_folder().to_string_lossy(),
                created,
                last_used,
                if profile.is_default { "1" } else { "0" }
            ));
        }

        let temp = paths::settings_path().with_extension("ini.tmp");
        fs::write(&temp, output).map_err(|_| "error_unexpected".to_string())?;
        fs::copy(&temp, paths::settings_path()).map_err(|_| "error_unexpected".to_string())?;
        let _ = fs::remove_file(temp);
        Ok(())
    }

    pub fn default(&self) -> &Profile {
        self.profiles
            .iter()
            .find(|profile| profile.is_default)
            .expect("default profile is always present")
    }

    pub fn active(&self) -> Option<&Profile> {
        self.profiles
            .iter()
            .find(|profile| profile.id == self.active_id)
    }

    pub fn find(&self, id: &str) -> Option<&Profile> {
        self.profiles.iter().find(|profile| profile.id == id)
    }

    pub fn find_mut(&mut self, id: &str) -> Option<&mut Profile> {
        self.profiles.iter_mut().find(|profile| profile.id == id)
    }

    pub fn active_profile(&self) -> &Profile {
        self.active().unwrap_or_else(|| self.default())
    }

    pub fn name_exists(&self, name: &str, except_id: Option<&str>) -> bool {
        let needle = name.trim().to_lowercase();
        if needle.is_empty() {
            return false;
        }

        if needle == "original account" || needle == "orijinal hesap" {
            return true;
        }

        self.profiles.iter().any(|profile| {
            Some(profile.id.as_str()) != except_id
                && !profile.is_default
                && profile.name.trim().to_lowercase() == needle
        })
    }

    pub fn validate_name(&self, name: &str, except_id: Option<&str>) -> Result<String, String> {
        let cleaned = sanitize_name(name);
        if cleaned.is_empty() {
            return Err("error_name_empty".to_string());
        }
        if cleaned.chars().count() > 60 {
            return Err("error_name_long".to_string());
        }
        if self.name_exists(&cleaned, except_id) {
            return Err("error_name_exists".to_string());
        }
        Ok(cleaned)
    }

    pub fn make_stored_folder(&self, name: &str) -> PathBuf {
        let mut safe = sanitize_file_name(name);
        if safe.is_empty() {
            safe = "account".to_string();
        }
        if safe.chars().count() > 40 {
            safe = safe.chars().take(40).collect::<String>().trim().to_string();
        }

        let root = paths::profiles_root();
        let mut folder = root.join(&safe);
        let mut counter = 2;
        while folder.exists()
            || self
                .profiles
                .iter()
                .any(|profile| !profile.is_default && paths::same_path(&profile.folder, &folder))
        {
            folder = root.join(format!("{safe} ({counter})"));
            counter += 1;
        }
        folder
    }

    pub fn create_profile(&self, name: &str) -> Profile {
        Profile {
            id: uuid::Uuid::new_v4().simple().to_string()[..10].to_string(),
            name: name.to_string(),
            folder: self.make_stored_folder(name),
            is_default: false,
            created_ticks: paths::now_ticks(),
            last_used_ticks: 0,
        }
    }

    pub fn profiles_dto(&self) -> Vec<ProfileDto> {
        self.profiles
            .iter()
            .map(|profile| ProfileDto {
                id: profile.id.clone(),
                name: profile.display_name(&self.language),
                is_default: profile.is_default,
                folder: profile.stored_folder().to_string_lossy().to_string(),
                last_used_ms: if profile.last_used_ticks <= 0 {
                    None
                } else {
                    Some(paths::ticks_to_ms(profile.last_used_ticks))
                },
            })
            .collect()
    }
}

pub fn default_account_name(language: &str) -> &'static str {
    if language == "tr" {
        "Orijinal hesap"
    } else {
        "Original account"
    }
}

pub fn system_language() -> String {
    let locale = sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string());
    if locale.to_lowercase().starts_with("tr") {
        "tr".to_string()
    } else {
        "en".to_string()
    }
}

fn default_profile() -> Profile {
    Profile {
        id: "default".to_string(),
        name: "Original account".to_string(),
        folder: paths::default_store(),
        is_default: true,
        created_ticks: paths::now_ticks(),
        last_used_ticks: 0,
    }
}

fn parse_profile(line: &str) -> Option<Profile> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 7 {
        return None;
    }

    let id = parts[1].trim();
    let folder = parts[3].trim();
    if id.is_empty() || folder.is_empty() {
        return None;
    }

    Some(Profile {
        id: id.to_string(),
        name: parts[2].trim().to_string(),
        folder: PathBuf::from(folder),
        is_default: parts[6].trim() == "1",
        created_ticks: parts[4].trim().parse::<i64>().unwrap_or(0),
        last_used_ticks: parts[5].trim().parse::<i64>().unwrap_or(0),
    })
}

pub fn sanitize_name(name: &str) -> String {
    name.replace(['\t', '\r', '\n'], " ").trim().to_string()
}

fn sanitize_file_name(name: &str) -> String {
    const INVALID: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut result = String::new();
    for character in name.chars() {
        if character.is_control() || INVALID.contains(&character) {
            result.push('_');
        } else {
            result.push(character);
        }
    }
    result.trim().trim_end_matches('.').to_string()
}
