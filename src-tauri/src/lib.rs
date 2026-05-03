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

use tauri::{Manager, RunEvent, WebviewUrl};
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("luciole=info")),
        )
        .with_target(false)
        .init();

    let app = tauri::Builder::default()
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
            commands::reset_onboarding,
            commands::list_notes,
            commands::read_note,
            commands::list_glossary,
            commands::add_glossary_entry,
            commands::remove_glossary_entry,
            commands::start_meeting,
            commands::stop_meeting,
            commands::run_text_action,
            commands::run_text_action_on_selection,
            commands::check_ollama_installed,
            commands::check_microphone_permission,
            commands::request_microphone_prompt,
            commands::check_accessibility_permission,
            commands::request_accessibility_prompt,
            commands::open_system_settings,
            commands::current_exe_path,
            commands::whisper_status,
            commands::transcribe_microphone_test,
        ])
        .setup(|app| {
            paths::ensure_app_dirs()?;
            settings::load_or_init(&app.handle())?;
            asr::init_bundle(app.handle());
            tray::setup_tray(app.handle())?;
            shortcuts::register_default_shortcuts(app.handle())?;

            // Overlay window (Snaply-like “+” near selection).
            // Hidden by default; shown/positioned by macOS selection monitor.
            let _overlay = tauri::WebviewWindowBuilder::new(
                app,
                "overlay",
                WebviewUrl::App("index.html#/overlay".into()),
            )
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .visible(false)
            .inner_size(56.0, 56.0)
            .build();

            system::overlay::start(app.handle().clone());

            if let Some(main) = app.get_webview_window("main") {
                // macOS UX: "close window" should behave like "hide to tray",
                // not terminate/destroy main window (so reopen works).
                let main_for_close = main.clone();
                main.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = main_for_close.hide();
                    }
                });

                // Tray-first app: only steal focus when setup required.
                let ax_ok = crate::system::permissions::accessibility_granted();
                let mic_ok = crate::system::permissions::microphone_granted();
                if ax_ok && mic_ok {
                    let _ = main.hide();
                } else {
                    let _ = main.show();
                    let _ = main.set_focus();
                }
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Luciole");

    app.run(|app, event| {
        // macOS: Dock click triggers `Reopen` even for LSUIElement apps.
        // Bring main window back when user explicitly asks.
        #[cfg(target_os = "macos")]
        if let RunEvent::Reopen { .. } = event {
            if let Some(main) = app.get_webview_window("main") {
                let _ = main.show();
                let _ = main.set_focus();
            }
        }
    });
}
