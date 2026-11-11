pub fn app_title(_language: &str) -> &'static str {
    "Clauwitch"
}

pub fn tray_show(language: &str) -> &'static str {
    if language == "tr" {
        "Pencereyi göster"
    } else {
        "Show window"
    }
}

pub fn tray_quit(language: &str) -> &'static str {
    if language == "tr" {
        "Çıkış"
    } else {
        "Quit"
    }
}

pub fn tray_active(language: &str, name: &str) -> String {
    if language == "tr" {
        format!("Aktif: {name}")
    } else {
        format!("Active: {name}")
    }
}

pub fn tray_open_claude(language: &str) -> &'static str {
    if language == "tr" {
        "Claude'u aç"
    } else {
        "Open Claude"
    }
}

pub fn tray_settings(language: &str) -> &'static str {
    if language == "tr" {
        "Ayarlar"
    } else {
        "Settings"
    }
}
