use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::AppHandle;

use crate::commands;
use crate::i18n;
use crate::settings::Settings;

pub fn setup(app: &AppHandle, settings: &Settings) -> tauri::Result<()> {
    let menu = build_menu(app, settings)?;

    let mut builder = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip(i18n::app_title(&settings.language))
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            if id == "show" {
                crate::claude::show_window(app);
            } else if id == "quit" {
                if !crate::commands::is_busy() {
                    app.exit(0);
                }
            } else if let Some(profile_id) = id.strip_prefix("account:") {
                commands::show_and_emit_switch(app, profile_id.to_string());
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app)?;
    Ok(())
}

pub fn rebuild(app: &AppHandle) -> tauri::Result<()> {
    let settings = Settings::load();
    let menu = build_menu(app, &settings)?;
    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu))?;
        let _ = tray.set_tooltip(Some(i18n::app_title(&settings.language)));
    }
    Ok(())
}

fn build_menu(app: &AppHandle, settings: &Settings) -> tauri::Result<Menu<tauri::Wry>> {
    let language = &settings.language;
    let menu = Menu::new(app)?;

    let show = MenuItem::with_id(app, "show", i18n::tray_show(language), true, None::<&str>)?;
    menu.append(&show)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    for profile in &settings.profiles {
        let id = format!("account:{}", profile.id);
        let item = CheckMenuItem::with_id(
            app,
            id,
            profile.display_name(language),
            true,
            profile.id == settings.active_id,
            None::<&str>,
        )?;
        menu.append(&item)?;
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    let quit = MenuItem::with_id(app, "quit", i18n::tray_quit(language), true, None::<&str>)?;
    menu.append(&quit)?;

    Ok(menu)
}
