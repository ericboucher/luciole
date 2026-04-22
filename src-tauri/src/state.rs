//! Shared runtime state held inside Tauri's managed state container.

use std::sync::Mutex;

#[derive(Default)]
pub struct AppState {
    pub meeting: Mutex<MeetingRuntime>,
    pub dictation: Mutex<DictationRuntime>,
}

#[derive(Default)]
pub struct MeetingRuntime {
    pub recording: bool,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub session_id: Option<String>,
}

#[derive(Default)]
pub struct DictationRuntime {
    pub listening: bool,
}
