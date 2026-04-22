//! Luciole — local-first AI companion.
//!
//! The Rust backend owns all system-level plumbing (hotkeys, audio capture,
//! text injection, file I/O) and exposes a small command surface to the React
//! frontend.

mod asr;
mod commands;
mod glossary;
mod llm;
mod meeting;
mod paths;
mod settings;
mod shortcuts;
mod state;
mod system;
mod tray;

use tauri::Manager;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("luciole=info")),
        )
        .with_target(false)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::update_settings,
            commands::onboarding_status,
            commands::complete_onboarding,
            commands::list_notes,
            commands::read_note,
            commands::list_glossary,
            commands::add_glossary_entry,
            commands::remove_glossary_entry,
            commands::start_meeting,
            commands::stop_meeting,
            commands::run_text_action,
            commands::check_ollama_installed,
            commands::check_microphone_permission,
            commands::check_accessibility_permission,
            commands::request_accessibility_prompt,
        ])
        .setup(|app| {
            paths::ensure_app_dirs()?;
            settings::load_or_init(&app.handle())?;
            tray::setup_tray(app.handle())?;
            shortcuts::register_default_shortcuts(app.handle())?;

            // Hide the main window on first boot; the menu bar is the anchor.
            if let Some(main) = app.get_webview_window("main") {
                let _ = main.hide();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Luciole");
}
