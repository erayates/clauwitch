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
