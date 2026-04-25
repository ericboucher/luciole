//! Command surface exposed to the React frontend via `invoke()`.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::{glossary, llm, meeting, settings, state::AppState, system};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub path: String,
    pub title: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingStatus {
    pub onboarding_completed: bool,
    pub whisper_ready: bool,
    pub ollama_installed: bool,
    pub ollama_model_ready: bool,
    pub diarization_ready: bool,
    pub microphone_granted: bool,
    pub accessibility_granted: bool,
}

#[tauri::command]
pub fn get_settings() -> settings::Settings {
    settings::snapshot()
}

#[tauri::command]
pub fn update_settings(next: settings::Settings) -> Result<(), String> {
    settings::update(next).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn complete_onboarding() -> Result<(), String> {
    settings::mark_onboarding_complete().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_onboarding() -> Result<(), String> {
    settings::mark_onboarding_incomplete().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn onboarding_status() -> OnboardingStatus {
    let settings = settings::snapshot();
    OnboardingStatus {
        onboarding_completed: settings.onboarding_completed,
        // TODO: actual readiness probes — hash checks against bundled models.
        whisper_ready: false,
        ollama_installed: llm::ollama_installed(),
        ollama_model_ready: false,
        diarization_ready: false,
        microphone_granted: system::permissions::microphone_granted(),
        accessibility_granted: system::permissions::accessibility_granted(),
    }
}

#[tauri::command]
pub fn list_notes() -> Result<Vec<Note>, String> {
    meeting::list_notes().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_note(path: String) -> Result<String, String> {
    meeting::read_note(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_glossary() -> Result<Vec<glossary::Entry>, String> {
    glossary::load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_glossary_entry(short: String, full: String) -> Result<(), String> {
    glossary::add_entry(short, full).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_glossary_entry(short: String) -> Result<(), String> {
    glossary::remove_entry(&short).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_meeting(state: State<'_, AppState>) -> Result<String, String> {
    meeting::start(&state).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_meeting(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<meeting::MeetingResult, String> {
    meeting::stop(&app, &state).await.map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextActionRequest {
    pub action: String,
    pub text: String,
    pub target_language: Option<String>,
    pub tone: Option<String>,
}

#[tauri::command]
pub async fn run_text_action(req: TextActionRequest) -> Result<String, String> {
    llm::run_text_action(req).await.map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextActionOnSelectionRequest {
    pub action: String,
    pub target_language: Option<String>,
    pub tone: Option<String>,
}

#[tauri::command]
pub async fn run_text_action_on_selection(req: TextActionOnSelectionRequest) -> Result<String, String> {
    let text = system::selection::selected_text().map_err(|e| e.to_string())?;
    let out = llm::run_text_action(TextActionRequest {
        action: req.action,
        text,
        target_language: req.target_language,
        tone: req.tone,
    })
    .await
    .map_err(|e| e.to_string())?;

    system::injection::paste_into_focused_app(&out).map_err(|e| e.to_string())?;
    Ok(out)
}

#[tauri::command]
pub fn check_ollama_installed() -> bool {
    llm::ollama_installed()
}

#[tauri::command]
pub fn check_microphone_permission() -> bool {
    system::permissions::microphone_granted()
}

#[tauri::command]
pub fn request_microphone_prompt() -> Result<(), String> {
    system::permissions::prompt_microphone().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_accessibility_permission() -> bool {
    system::permissions::accessibility_granted()
}

#[tauri::command]
pub fn request_accessibility_prompt() -> Result<(), String> {
    system::permissions::prompt_accessibility().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_system_settings(panel: String) -> Result<(), String> {
    // Use macOS `open` so we don't depend on shell-plugin URL allowlisting.
    #[cfg(target_os = "macos")]
    {
        let url = match panel.as_str() {
            "microphone" => "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone",
            "accessibility" => "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
            _ => return Err("unknown panel".into()),
        };

        std::process::Command::new("/usr/bin/open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = panel;
        Err("only available on macOS".into())
    }
}

#[tauri::command]
pub fn current_exe_path() -> Result<String, String> {
    std::env::current_exe()
        .map(|p| p.display().to_string())
        .map_err(|e| e.to_string())
}
