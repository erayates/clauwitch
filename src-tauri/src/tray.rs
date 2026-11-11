use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter};

use crate::commands;
use crate::i18n;
use crate::settings::Settings;

pub fn setup(app: &AppHandle, settings: &Settings) -> tauri::Result<()> {
    let menu = build_menu(app, settings)?;

    let mut builder = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip(i18n::app_title(&settings.language))
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::claude::show_window(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            if id == "show" {
                crate::claude::show_window(app);
            } else if id == "open_claude" {
                // Same path as the UI so the busy guard and state refresh apply.
                crate::claude::show_window(app);
                let _ = app.emit("tray-open-claude", ());
            } else if id == "settings" {
                crate::claude::show_window(app);
                let _ = app.emit("tray-settings", ());
            } else if id == "quit" {
                if !commands::is_busy() {
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
        let active = settings.active_profile().display_name(&settings.language);
        let _ = tray.set_tooltip(Some(format!(
            "{} — {}",
            i18n::app_title(&settings.language),
            active
        )));
    }
    Ok(())
}

fn build_menu(app: &AppHandle, settings: &Settings) -> tauri::Result<Menu<tauri::Wry>> {
    let language = &settings.language;
    let menu = Menu::new(app)?;

    let active_name = settings.active_profile().display_name(language);
    let header = MenuItem::with_id(
        app,
        "active",
        i18n::tray_active(language, &active_name),
        false,
        None::<&str>,
    )?;
    menu.append(&header)?;
    menu.append(&MenuItem::with_id(
        app,
        "open_claude",
        i18n::tray_open_claude(language),
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "show",
        i18n::tray_show(language),
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "settings",
        i18n::tray_settings(language),
        true,
        None::<&str>,
    )?)?;
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
