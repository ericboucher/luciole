import { useState } from "react";
import { ipc, type OnboardingStatus } from "../lib/ipc";

interface Step {
  label: string;
  detail: string;
  ready: (s: OnboardingStatus) => boolean;
  stub?: boolean;
}

const STEPS: Step[] = [
  {
    label: "Modèle de transcription vocale",
    detail: "Whisper small (460 Mo)",
    ready: (s) => s.whisperReady,
    stub: true,
  },
  {
    label: "Moteur IA local",
    detail: "Ollama — installation silencieuse, données locales uniquement",
    ready: (s) => s.ollamaInstalled,
  },
  {
    label: "Modèle de langage",
    detail: "Gemma 4 E4B (5,6 Go)",
    ready: (s) => s.ollamaModelReady,
    stub: true,
  },
  {
    label: "Reconnaissance des locuteurs",
    detail: "sherpa-onnx + pyannote (75 Mo)",
    ready: (s) => s.diarizationReady,
    stub: true,
  },
];

export function Onboarding({
  initial,
  onComplete,
}: {
  initial: OnboardingStatus;
  onComplete: () => void;
}) {
  const [status, setStatus] = useState<OnboardingStatus>(initial);
  const [devBypass, setDevBypass] = useState(false);

  const allReady =
    STEPS.every((s) => s.ready(status)) &&
    status.microphoneGranted &&
    status.accessibilityGranted;
  const canContinue =
    allReady || (import.meta.env.DEV && devBypass && status.ollamaInstalled);

  const refresh = async () => {
    const next = await ipc.onboardingStatus();
    setStatus(next);
  };

  const requestAccessibility = async () => {
    await ipc.requestAccessibilityPrompt();
    setTimeout(refresh, 500);
  };

  const openMicSettings = async () => {
    await ipc.openSystemSettings("microphone");
  };

  const openAxSettings = async () => {
    await ipc.openSystemSettings("accessibility");
  };

  return (
    <div className="app">
      <div className="app-main">
        <div className="onboarding">
          <h1>Bienvenue dans Luciole</h1>
          <p>
            Votre compagnon IA local et privé. La configuration prépare les
            modèles sur votre Mac — aucune donnée n'est envoyée en ligne.
          </p>

          <div className="card" style={{ marginTop: 16 }}>
            <strong>Note (scaffold v0.1)</strong>
            <p style={{ margin: "8px 0 0" }}>
              Les étapes Whisper / Gemma / diarisation ne se téléchargent pas
              encore automatiquement dans ce scaffold. Seules la détection
              Ollama et les permissions sont actives pour l&apos;instant.
            </p>
            {import.meta.env.DEV ? (
              <label className="row" style={{ gap: 10, marginTop: 12 }}>
                <input
                  type="checkbox"
                  checked={devBypass}
                  onChange={(e) => setDevBypass(e.target.checked)}
                />
                <span style={{ fontSize: 13, color: "var(--color-text-muted)" }}>
                  Mode dev : continuer même si permissions / modèles non prêts
                  (requiert Ollama installé)
                </span>
              </label>
            ) : null}
          </div>

          <div className="stack" style={{ marginTop: 24 }}>
            {STEPS.map((step, i) => (
              <div className="step" key={step.label}>
                <div className="row between">
                  <strong>
                    {i + 1}/{STEPS.length} — {step.label}
                  </strong>
                  <span
                    className={`badge ${step.ready(status) ? "success" : "warning"}`}
                  >
                    {step.ready(status)
                      ? "Prêt"
                      : step.stub
                        ? "Non câblé"
                        : "En attente"}
                  </span>
                </div>
                <span style={{ color: "var(--color-text-muted)", fontSize: 13 }}>
                  {step.detail}
                </span>
              </div>
            ))}
          </div>

          <h2 style={{ marginTop: 32 }}>Permissions requises</h2>
          <div className="stack">
            <div className="row between card" style={{ marginBottom: 0 }}>
              <div>
                <strong>Microphone</strong>
                <p style={{ margin: 0 }}>
                  Nécessaire pour la dictée et l'enregistrement.
                </p>
              </div>
              {status.microphoneGranted ? (
                <span className="badge success">Autorisé</span>
              ) : (
                <div className="row" style={{ gap: 8 }}>
                  <button className="secondary" onClick={openMicSettings}>
                    Réglages →
                  </button>
                  <button onClick={refresh}>Vérifier</button>
                </div>
              )}
            </div>
            <div className="row between card" style={{ marginBottom: 0 }}>
              <div>
                <strong>Accessibilité</strong>
                <p style={{ margin: 0 }}>
                  Nécessaire pour lire les sélections et insérer du texte.
                </p>
              </div>
              {status.accessibilityGranted ? (
                <span className="badge success">Autorisé</span>
              ) : (
                <div className="row" style={{ gap: 8 }}>
                  <button className="secondary" onClick={openAxSettings}>
                    Réglages →
                  </button>
                  <button className="secondary" onClick={requestAccessibility}>
                    Demander…
                  </button>
                  <button onClick={refresh}>Vérifier</button>
                </div>
              )}
            </div>
          </div>

          {import.meta.env.DEV && !status.accessibilityGranted ? (
            <p style={{ marginTop: 12, color: "var(--color-text-muted)", fontSize: 13 }}>
              En dev, macOS affiche souvent le binaire en cours d&apos;exécution
              (ex. <span className="kbd">target/debug/luciole</span>) dans la
              liste Accessibilité. Ajoute-le, puis clique{" "}
              <span className="kbd">Vérifier</span>.
            </p>
          ) : null}

          <div className="row between" style={{ marginTop: 32 }}>
            <button className="ghost" onClick={refresh}>
              Actualiser
            </button>
            <button disabled={!canContinue} onClick={onComplete}>
              C'est parti
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
