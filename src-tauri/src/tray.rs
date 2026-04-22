//! Menu bar (tray) icon and menu.
//!
//! Luciole lives in the macOS menu bar (`LSUIElement = true` in Info.plist).
//! The dock icon is hidden; the tray menu is the primary entry point.

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

pub fn setup_tray(app: &AppHandle) -> anyhow::Result<()> {
    let open_notes = MenuItem::with_id(app, "open-notes", "Mes notes", true, None::<&str>)?;
    let open_glossary =
        MenuItem::with_id(app, "open-glossary", "Glossaire", true, None::<&str>)?;
    let open_settings =
        MenuItem::with_id(app, "open-settings", "Réglages…", true, None::<&str>)?;
    let toggle_meeting = MenuItem::with_id(
        app,
        "toggle-meeting",
        "Démarrer une réunion",
        true,
        Some("Alt+M"),
    )?;
    let separator_1 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let separator_2 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quitter Luciole", true, Some("Cmd+Q"))?;

    let menu = Menu::with_items(
        app,
        &[
            &toggle_meeting,
            &separator_1,
            &open_notes,
            &open_glossary,
            &open_settings,
            &separator_2,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id("luciole-tray")
        .menu(&menu)
        .menu_on_left_click(false)
        .tooltip("Luciole")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open-notes" => focus_main(app, "#/notes"),
            "open-glossary" => focus_main(app, "#/glossary"),
            "open-settings" => focus_main(app, "#/settings"),
            "toggle-meeting" => {
                let _ = app.emit_to("main", "shortcut:meeting-toggle", true);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn focus_main(app: &AppHandle, _hash: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        // TODO: emit a navigation event so the React router scrolls to `hash`.
    }
}

