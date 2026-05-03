//! User-editable acronym glossary (PRD §5.6).
//!
//! Backed by a simple YAML file at `~/.luciole/glossaire.yaml`. On first
//! launch we seed it from the bundled public-sector glossary shipped under
//! `resources/glossaire-secteur-public.yaml`.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub short: String,
    pub full: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Document {
    #[serde(default)]
    acronyms: Vec<Entry>,
}

pub fn load() -> anyhow::Result<Vec<Entry>> {
    let path = paths::glossary_path();
    if !path.exists() {
        seed_default(&path)?;
    }
    let raw = std::fs::read_to_string(&path)?;
    let doc: Document = serde_yaml::from_str(&raw).unwrap_or_default();
    Ok(doc.acronyms)
}

pub fn save(entries: &[Entry]) -> anyhow::Result<()> {
    let path = paths::glossary_path();
    let doc = Document {
        acronyms: entries.to_vec(),
    };
    std::fs::write(path, serde_yaml::to_string(&doc)?)?;
    Ok(())
}

pub fn add_entry(short: String, full: String) -> anyhow::Result<()> {
    let mut entries = load()?;
    if let Some(existing) = entries.iter_mut().find(|e| e.short == short) {
        existing.full = full;
    } else {
        entries.push(Entry { short, full });
    }
    save(&entries)
}

pub fn remove_entry(short: &str) -> anyhow::Result<()> {
    let mut entries = load()?;
    entries.retain(|e| e.short != short);
    save(&entries)
}

/// Build a system prompt fragment injecting the glossary (PRD §5.6 pass 2).
pub fn system_prompt_fragment() -> anyhow::Result<String> {
    let entries = load()?;
    if entries.is_empty() {
        return Ok(String::new());
    }
    let mut out = String::from("Glossaire (utilise ces définitions si l'acronyme apparaît) :\n");
    for e in entries {
        out.push_str(&format!("- {} = {}\n", e.short, e.full));
    }
    Ok(out)
}

fn seed_default(path: &Path) -> anyhow::Result<()> {
    // Bundled default glossary path resolves at runtime via tauri resources.
    // For the scaffold we ship the file at the repo root; the build pipeline
    // copies it into `resources/` at bundle time.
    let seed = include_str!("../resources/glossaire-secteur-public.yaml");
    std::fs::write(path, seed)?;
    Ok(())
}
