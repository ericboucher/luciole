//! Luciole data directory layout.
//!
//! All user data lives under `~/.luciole/`. The directory is created lazily
//! on first launch. See PRD §5.8.

use std::path::PathBuf;

use directories::BaseDirs;
use once_cell::sync::Lazy;

static ROOT: Lazy<PathBuf> = Lazy::new(|| {
    BaseDirs::new()
        .map(|d| d.home_dir().join(".luciole"))
        .expect("no home directory found")
});

pub fn root() -> &'static PathBuf {
    &ROOT
}

pub fn notes_dir() -> PathBuf {
    root().join("notes")
}

pub fn transcripts_dir() -> PathBuf {
    root().join("transcripts")
}

pub fn models_dir() -> PathBuf {
    root().join("models")
}

pub fn settings_path() -> PathBuf {
    root().join("settings.json")
}

pub fn glossary_path() -> PathBuf {
    root().join("glossaire.yaml")
}

pub fn db_path() -> PathBuf {
    root().join("db.sqlite")
}

pub fn speakers_db_path() -> PathBuf {
    root().join("speakers.db")
}

pub fn ensure_app_dirs() -> anyhow::Result<()> {
    for dir in [root().clone(), notes_dir(), transcripts_dir(), models_dir()] {
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
    }
    Ok(())
}
