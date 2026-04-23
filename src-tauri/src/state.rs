//! Shared runtime state held inside Tauri's managed state container.

use std::sync::Mutex;

#[derive(Default)]
pub struct AppState {
    pub meeting: Mutex<MeetingRuntime>,
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub listening: bool,
}
