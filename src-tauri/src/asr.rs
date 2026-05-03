//! Speech recognition via bundled **whisper.cpp** (`whisper-cli` sidecar).
//!
//! Paths are resolved at startup ([`init_bundle`]):
//! - Executable: next to the app binary (`whisper`), Tauri sidecar, or `src-tauri/binaries/whisper-{TARGET}` in dev.
//! - Model: `resources/models/ggml-tiny.bin` (bundled). Optional override `LUCIOLE_WHISPER_CPP` for debugging only.

use std::path::{Path, PathBuf};
use std::process::Command;

use std::sync::OnceLock;
use serde::Deserialize;
use tauri::{AppHandle, Manager};
use tauri::path::BaseDirectory;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    /// Short utterance (dictation). Uses settings `whisper_model`.
    Dictation,
    /// Long-form batch transcription (meeting). Prefer large model if present.
    Meeting,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Transcript {
    pub language: String,
    pub segments: Vec<Segment>,
}

impl Transcript {
    pub fn flat_text(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.text.trim())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Maps settings key (`small`, `base`, …) to whisper.cpp GGML filename.
pub fn model_file_name(whisper_model_setting: &str) -> String {
    let key = whisper_model_setting.trim().to_lowercase();
    let base = match key.as_str() {
        "tiny" => "tiny",
        "base" => "base",
        "small" => "small",
        "medium" => "medium",
        "large" | "large-v3" | "large-v3-turbo" => "large-v3-turbo",
        _ => "small",
    };
    format!("ggml-{base}.bin")
}

pub fn model_path(whisper_model_setting: &str) -> PathBuf {
    crate::paths::models_dir().join(model_file_name(whisper_model_setting))
}

/// Bundled whisper.cpp locations (set once from [`init_bundle`]).
#[derive(Debug, Clone)]
pub struct BundlePaths {
    pub whisper_exe: PathBuf,
    pub model: PathBuf,
}

static BUNDLE: OnceLock<BundlePaths> = OnceLock::new();

pub fn init_bundle(app: &AppHandle) {
    let paths = BundlePaths {
        whisper_exe: resolve_whisper_executable(app),
        model: resolve_bundled_model(app),
    };
    tracing::info!(
        exe = %paths.whisper_exe.display(),
        model = %paths.model.display(),
        exe_ok = paths.whisper_exe.is_file(),
        model_ok = paths.model.is_file(),
        "whisper bundle paths"
    );
    let _ = BUNDLE.set(paths);
}

pub fn bundle() -> Option<&'static BundlePaths> {
    BUNDLE.get()
}

fn resolve_whisper_executable(app: &AppHandle) -> PathBuf {
    if let Ok(p) = std::env::var("LUCIOLE_WHISPER_CPP") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return pb;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let w = dir.join("whisper");
            if w.is_file() {
                return w;
            }
        }
    }
    if let Ok(p) = app.path().resolve("whisper", BaseDirectory::Resource) {
        if p.is_file() {
            return p;
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join(format!("whisper-{}", host_triple()))
}

fn host_triple() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "x86_64-unknown-linux-gnu"
    } else {
        "unknown-unknown-unknown"
    }
}

fn resolve_bundled_model(app: &AppHandle) -> PathBuf {
    if let Ok(p) = app.path().resolve("models/ggml-tiny.bin", BaseDirectory::Resource) {
        if p.is_file() {
            return p;
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/models/ggml-tiny.bin")
}

pub fn transcribe(audio: &Path, mode: Mode, whisper_model: &str, language: &str) -> anyhow::Result<Transcript> {
    let audio = audio
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("fichier audio introuvable: {e}"))?;

    let meta = std::fs::metadata(&audio)?;
    if meta.len() > 500 * 1024 * 1024 {
        anyhow::bail!("fichier trop volumineux (>500 Mo)");
    }

    let bundle = BUNDLE
        .get()
        .ok_or_else(|| anyhow::anyhow!("moteur STT non initialisé (bundle manquant)"))?;

    if !bundle.whisper_exe.is_file() {
        anyhow::bail!(
            "binaire whisper absent: {}\n\
             Exécute scripts/vendor-whisper-macos.sh puis place le résultat sous src-tauri/binaries/",
            bundle.whisper_exe.display()
        );
    }

    let model_file = match mode {
        Mode::Dictation => bundle.model.clone(),
        Mode::Meeting => {
            let large_path = model_path("large-v3-turbo");
            if large_path.is_file() {
                large_path
            } else {
                let alt = model_path(whisper_model);
                if alt.is_file() {
                    alt
                } else {
                    bundle.model.clone()
                }
            }
        }
    };

    if !model_file.is_file() {
        anyhow::bail!(
            "modèle Whisper absent: {}\n\
             Attendu: resources/models/ggml-tiny.bin (voir scripts/fetch-assets.sh).",
            model_file.display()
        );
    }

    let whisper_bin = bundle.whisper_exe.clone();
    // whisper `-of BASE` writes BASE.txt (not `<input>.wav.txt`).
    let out_base = audio.with_extension("");
    let txt_from_of = out_base.with_extension("txt");
    let _ = std::fs::remove_file(&txt_from_of);

    let lang = language.trim();
    let mut cmd = Command::new(&whisper_bin);
    cmd.arg("-m").arg(&model_file);
    cmd.arg("-f").arg(&audio);
    cmd.arg("-otxt");
    cmd.arg("-of").arg(&out_base);
    if !lang.is_empty() && lang != "auto" {
        cmd.arg("-l").arg(lang);
    }
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let out = cmd.output().map_err(|e| {
        anyhow::anyhow!(
            "échec exécution whisper.cpp ({e}). Vérifie que le binaire est exécutable."
        )
    })?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("whisper.cpp a échoué: {}", err.trim());
    }

    let text = if txt_from_of.is_file() {
        std::fs::read_to_string(&txt_from_of)?
    } else {
        // Some builds still emit next to the audio file.
        let legacy = PathBuf::from(format!("{}.txt", audio.display()));
        if legacy.is_file() {
            std::fs::read_to_string(&legacy)?
        } else {
            anyhow::bail!(
                "sortie texte absente après whisper (attendu: {}).",
                txt_from_of.display()
            )
        }
    };

    let text = text.trim().to_string();

    // Optional JSON (if generated with `-oj`); same base as `-of`.
    let json_path = out_base.with_extension("json");
    let segments = if json_path.is_file() {
        parse_whisper_json_file(&json_path).unwrap_or_else(|_| vec![segment_from_flat(&text)])
    } else {
        vec![segment_from_flat(&text)]
    };

    Ok(Transcript {
        language: language.to_string(),
        segments,
    })
}

fn segment_from_flat(text: &str) -> Segment {
    Segment {
        start_ms: 0,
        end_ms: 0,
        text: text.to_string(),
    }
}

#[derive(Deserialize)]
struct WhisperJson {
    #[serde(default)]
    transcription: Vec<WhisperSeg>,
    #[serde(default)]
    result: Option<WhisperResult>,
}

#[derive(Deserialize)]
struct WhisperResult {
    #[serde(default)]
    transcription: Vec<WhisperSeg>,
}

#[derive(Deserialize)]
struct WhisperSeg {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    timestamps: Option<WhisperTimestamps>,
}

#[derive(Deserialize)]
struct WhisperTimestamps {
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
}

fn parse_whisper_json_file(path: &Path) -> anyhow::Result<Vec<Segment>> {
    let raw = std::fs::read_to_string(path)?;
    let j: WhisperJson = serde_json::from_str(&raw)?;
    let rows = if !j.transcription.is_empty() {
        j.transcription
    } else if let Some(r) = j.result {
        r.transcription
    } else {
        vec![]
    };

    let mut out = Vec::new();
    for row in rows {
        let text = row.text.unwrap_or_default().trim().to_string();
        if text.is_empty() {
            continue;
        }
        let (start_ms, end_ms) = row
            .timestamps
            .map(|t| {
                (
                    parse_ts_ms(t.from.as_deref().unwrap_or("0")),
                    parse_ts_ms(t.to.as_deref().unwrap_or("0")),
                )
            })
            .unwrap_or((0, 0));
        out.push(Segment {
            start_ms,
            end_ms,
            text,
        });
    }
    if out.is_empty() {
        anyhow::bail!("no segments");
    }
    Ok(out)
}

fn parse_ts_ms(s: &str) -> u64 {
    // Formats: "00:00:00.000" or plain ms
    let s = s.trim();
    if let Ok(n) = s.parse::<u64>() {
        return n;
    }
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() == 3 {
        let h: f64 = parts[0].parse().unwrap_or(0.0);
        let m: f64 = parts[1].parse().unwrap_or(0.0);
        let sec: f64 = parts[2].parse().unwrap_or(0.0);
        return ((h * 3600.0 + m * 60.0 + sec) * 1000.0) as u64;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_file_name_maps() {
        assert!(model_file_name("small").contains("small"));
        assert!(model_file_name("large-v3-turbo").contains("large-v3-turbo"));
    }
}
