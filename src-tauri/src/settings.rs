//! Persistent user settings.
//!
//! Written as JSON under `~/.luciole/settings.json` so users can inspect and
//! edit them with a plain text editor.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub language: String,
    pub whisper_model: String,
    pub ollama_model: String,
    pub obsidian_vault_path: Option<String>,
    pub shortcut_dictate: String,
    pub shortcut_correct: String,
    pub shortcut_translate: String,
    pub shortcut_rephrase: String,
    pub shortcut_meeting: String,
    pub onboarding_completed: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "fr".into(),
            whisper_model: "small".into(),
            ollama_model: "gemma4:e4b".into(),
            obsidian_vault_path: None,
            // Hold-to-dictate uses Option alone; the string form documents intent.
            shortcut_dictate: "Option".into(),
            shortcut_correct: "Option+P".into(),
            shortcut_translate: "Option+T".into(),
            shortcut_rephrase: "Option+R".into(),
            shortcut_meeting: "Option+M".into(),
            onboarding_completed: false,
        }
    }
}

pub static SETTINGS: once_cell::sync::Lazy<Mutex<Settings>> =
    once_cell::sync::Lazy::new(|| Mutex::new(Settings::default()));

pub fn load_or_init(_app: &AppHandle) -> anyhow::Result<()> {
    let path = paths::settings_path();
    let loaded = if path.exists() {
        let raw = std::fs::read_to_string(&path)?;
        serde_json::from_str::<Settings>(&raw).unwrap_or_default()
    } else {
        let defaults = Settings::default();
        std::fs::write(&path, serde_json::to_string_pretty(&defaults)?)?;
        defaults
    };
    *SETTINGS.lock().unwrap() = loaded;
    Ok(())
}

pub fn snapshot() -> Settings {
    SETTINGS.lock().unwrap().clone()
}

pub fn update(next: Settings) -> anyhow::Result<()> {
    let path = paths::settings_path();
    std::fs::write(&path, serde_json::to_string_pretty(&next)?)?;
    *SETTINGS.lock().unwrap() = next;
    Ok(())
}

pub fn mark_onboarding_complete() -> anyhow::Result<()> {
    let mut current = snapshot();
    current.onboarding_completed = true;
    update(current)
}

pub fn mark_onboarding_incomplete() -> anyhow::Result<()> {
    let mut current = snapshot();
    current.onboarding_completed = false;
    update(current)
}
