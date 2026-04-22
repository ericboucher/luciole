//! LLM integration — Gemma 4 E4B via local Ollama.
//!
//! We talk to the Ollama HTTP server at `http://127.0.0.1:11434`. Install
//! detection is a best-effort `which ollama` check; the onboarding wizard
//! runs the actual install with user consent (PRD §6).

use serde::{Deserialize, Serialize};

use crate::{commands::TextActionRequest, glossary, settings};

const OLLAMA_URL: &str = "http://127.0.0.1:11434";

pub fn ollama_installed() -> bool {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("/usr/bin/which")
            .arg("ollama")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            || std::path::Path::new("/Applications/Ollama.app").exists()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: String,
    system: String,
    stream: bool,
    options: GenerateOptions,
}

#[derive(Serialize)]
struct GenerateOptions {
    temperature: f32,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

pub async fn run_text_action(req: TextActionRequest) -> anyhow::Result<String> {
    let settings = settings::snapshot();
    let system = build_system_prompt(&req)?;
    let prompt = build_user_prompt(&req);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let body = GenerateRequest {
        model: &settings.ollama_model,
        prompt,
        system,
        stream: false,
        options: GenerateOptions { temperature: 0.2 },
    };

    let resp = client
        .post(format!("{OLLAMA_URL}/api/generate"))
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<GenerateResponse>()
        .await?;

    Ok(resp.response.trim().to_string())
}

fn build_system_prompt(req: &TextActionRequest) -> anyhow::Result<String> {
    let base = match req.action.as_str() {
        "correct" => "Tu es un correcteur de français. Corrige les fautes d'orthographe, \
            de grammaire et de ponctuation sans modifier le sens ni le style. \
            Réponds uniquement avec le texte corrigé, sans préambule.",
        "translate" => "Tu es un traducteur professionnel. Traduis le texte vers la langue \
            demandée en conservant le ton et le registre. Réponds uniquement avec la traduction.",
        "rephrase" => "Tu es un rédacteur. Reformule le texte dans le ton demandé \
            (formel ou informel) en conservant le sens. Réponds uniquement avec la reformulation.",
        "clean-dictation" => "Tu reçois une transcription vocale brute. Corrige la ponctuation \
            et les fautes manifestes sans ajouter de contenu. Réponds uniquement avec le texte nettoyé.",
        _ => anyhow::bail!("unknown action: {}", req.action),
    };

    let glossary_fragment = glossary::system_prompt_fragment()?;
    let mut out = String::from(base);
    if !glossary_fragment.is_empty() {
        out.push_str("\n\n");
        out.push_str(&glossary_fragment);
    }
    Ok(out)
}

fn build_user_prompt(req: &TextActionRequest) -> String {
    match req.action.as_str() {
        "translate" => {
            let lang = req.target_language.as_deref().unwrap_or("en");
            format!("Langue cible : {lang}\n\nTexte :\n{}", req.text)
        }
        "rephrase" => {
            let tone = req.tone.as_deref().unwrap_or("formel");
            format!("Ton : {tone}\n\nTexte :\n{}", req.text)
        }
        _ => req.text.clone(),
    }
}
