//! Global keyboard shortcuts.
//!
//! The `tauri-plugin-global-shortcut` plugin wraps `CGEvent` tap on macOS so
//! shortcuts fire without Luciole owning focus. Hold-to-dictate is modeled as
//! a press/release listener; the text actions fire on the keydown edge.

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::state::AppState;

pub fn register_default_shortcuts(app: &AppHandle) -> anyhow::Result<()> {
    let manager = app.global_shortcut();

    let correct = Shortcut::new(Some(Modifiers::ALT), Code::KeyP);
    let translate = Shortcut::new(Some(Modifiers::ALT), Code::KeyT);
    let rephrase = Shortcut::new(Some(Modifiers::ALT), Code::KeyR);
    let meeting = Shortcut::new(Some(Modifiers::ALT), Code::KeyM);

    let handle = app.clone();
    manager.on_shortcut(correct, move |_app, _shortcut, event| {
        if matches!(event.state(), ShortcutState::Pressed) {
            let _ = handle.emit("shortcut:text-action", "correct");
        }
    })?;

    let handle = app.clone();
    manager.on_shortcut(translate, move |_app, _shortcut, event| {
        if matches!(event.state(), ShortcutState::Pressed) {
            let _ = handle.emit("shortcut:text-action", "translate");
        }
    })?;

    let handle = app.clone();
    manager.on_shortcut(rephrase, move |_app, _shortcut, event| {
        if matches!(event.state(), ShortcutState::Pressed) {
            let _ = handle.emit("shortcut:text-action", "rephrase");
        }
    })?;

    let handle = app.clone();
    manager.on_shortcut(meeting, move |_app, _shortcut, event| {
        if matches!(event.state(), ShortcutState::Pressed) {
            let state = handle.state::<AppState>();
            let mut meeting = state.meeting.lock().unwrap();
            meeting.recording = !meeting.recording;
            let _ = handle.emit("shortcut:meeting-toggle", meeting.recording);
        }
    })?;

    // TODO (v1): wire hold-to-dictate via a low-level key tap. The global
    // shortcut plugin does not expose per-modifier hold events, so dictation
    // will use a dedicated CGEventTap implementation in `system::keytap`.

    Ok(())
}
