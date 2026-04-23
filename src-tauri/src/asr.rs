//! Speech recognition via whisper.cpp.
//!
//! The actual implementation spawns the bundled `whisper-cli` binary from
//! `src-tauri/binaries/whisper-cli-<triple>` with the requested model. For
//! the scaffold we expose the plumbing shape and leave the native invocation
//! as `todo!()` until the binary is wired in.

use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Mode {
    /// Short utterance (dictation). Optimized for latency, uses `small`.
    Dictation,
    /// Long-form batch transcription (meeting). Uses `large-v3-turbo` if
    /// available, otherwise falls back to `small`.
    Meeting,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Segment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Transcript {
    pub language: String,
    pub segments: Vec<Segment>,
}

#[allow(dead_code)]
impl Transcript {
    pub fn flat_text(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.text.trim())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[allow(dead_code)]
pub async fn transcribe(_audio: &Path, _mode: Mode, _language: &str) -> anyhow::Result<Transcript> {
    // TODO: spawn bundled whisper.cpp binary, parse JSON output.
    // Keeping the signature stable so the frontend contract is final.
    anyhow::bail!("whisper.cpp integration not yet wired in")
}
