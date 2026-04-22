//! Meeting recording, transcription, diarization and note generation.
//!
//! Pipeline (PRD §5.4):
//!   capture (mic + system) → whisper.cpp → sherpa-onnx diarization
//!   → speaker alignment → LLM summary → Markdown note
//!
//! This module owns the high-level orchestration; each step is implemented
//! in its own sibling module (some still stubbed — see TODO markers).

use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::{commands::Note, paths, state::AppState};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeetingResult {
    pub note_path: String,
    pub transcript_path: String,
    pub speakers_pending_naming: Vec<String>,
}

pub async fn start(state: &State<'_, AppState>) -> anyhow::Result<String> {
    let mut meeting = state.meeting.lock().unwrap();
    if meeting.recording {
        anyhow::bail!("meeting already in progress");
    }
    let session_id = format!("mtg-{}", Utc::now().format("%Y%m%d-%H%M%S"));
    meeting.recording = true;
    meeting.started_at = Some(Utc::now());
    meeting.session_id = Some(session_id.clone());
    // TODO: start AVFoundation capture on a background task and write a WAV
    // under `~/.luciole/transcripts/<session_id>.wav`.
    tracing::info!(%session_id, "meeting capture started (stub)");
    Ok(session_id)
}

pub async fn stop(_app: &AppHandle, state: &State<'_, AppState>) -> anyhow::Result<MeetingResult> {
    let (session_id, started_at) = {
        let mut meeting = state.meeting.lock().unwrap();
        if !meeting.recording {
            anyhow::bail!("no meeting in progress");
        }
        meeting.recording = false;
        let id = meeting
            .session_id
            .take()
            .ok_or_else(|| anyhow::anyhow!("missing session id"))?;
        let started = meeting.started_at.take();
        (id, started)
    };

    tracing::info!(%session_id, "meeting capture stopped (stub)");
    let _ = started_at; // silence dead_code for scaffold

    // TODO: run the full pipeline — transcription, diarization, summarization.
    // For the scaffold we write a placeholder note so the downstream UI has
    // something to display end-to-end.
    let date = Utc::now().format("%Y-%m-%d").to_string();
    let title = format!("{date}-{session_id}");
    let note_path = paths::notes_dir().join(format!("{title}.md"));
    let transcript_path = paths::transcripts_dir().join(format!("{title}_transcript.txt"));

    std::fs::write(
        &note_path,
        format!(
            "# Réunion — {date}\n\n*Note générée par Luciole. Le pipeline de diarisation et de résumé \
             n'est pas encore câblé dans cette version.*\n\n## Participants\n\n## Résumé\n\n## Décisions\n\n## Actions\n"
        ),
    )?;
    std::fs::write(&transcript_path, "")?;

    Ok(MeetingResult {
        note_path: note_path.to_string_lossy().into_owned(),
        transcript_path: transcript_path.to_string_lossy().into_owned(),
        speakers_pending_naming: vec![],
    })
}

pub fn list_notes() -> anyhow::Result<Vec<Note>> {
    let dir = paths::notes_dir();
    let mut out = vec![];
    if !dir.exists() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let metadata = entry.metadata()?;
        let created_at = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| {
                chrono::DateTime::<Utc>::from_timestamp(d.as_secs() as i64, 0)
                    .unwrap_or_else(Utc::now)
                    .to_rfc3339()
            })
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("note")
            .to_string();
        out.push(Note {
            path: path.to_string_lossy().into_owned(),
            title,
            created_at,
        });
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(out)
}

pub fn read_note(path: &str) -> anyhow::Result<String> {
    // Security: only allow reads under the notes directory.
    let canonical = std::fs::canonicalize(path)?;
    let notes_root = std::fs::canonicalize(paths::notes_dir())?;
    if !canonical.starts_with(&notes_root) {
        anyhow::bail!("note path outside notes directory");
    }
    Ok(std::fs::read_to_string(canonical)?)
}
