import { useState } from "react";
import { ipc, type OnboardingStatus } from "../lib/ipc";

interface Step {
  label: string;
  detail: string;
  ready: (s: OnboardingStatus) => boolean;
}

const STEPS: Step[] = [
  {
    label: "Modèle de transcription vocale",
    detail: "Whisper small (460 Mo)",
    ready: (s) => s.whisperReady,
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
  },
  {
    label: "Reconnaissance des locuteurs",
    detail: "sherpa-onnx + pyannote (75 Mo)",
    ready: (s) => s.diarizationReady,
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

  const allReady =
    STEPS.every((s) => s.ready(status)) &&
    status.microphoneGranted &&
    status.accessibilityGranted;

  const refresh = async () => {
    const next = await ipc.onboardingStatus();
    setStatus(next);
  };

  const requestAccessibility = async () => {
    await ipc.requestAccessibilityPrompt();
    setTimeout(refresh, 500);
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
                    {step.ready(status) ? "Prêt" : "En attente"}
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
                <button onClick={refresh}>Vérifier</button>
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
                <button className="secondary" onClick={requestAccessibility}>
                  Ouvrir Réglages →
                </button>
              )}
            </div>
          </div>

          <div className="row between" style={{ marginTop: 32 }}>
            <button className="ghost" onClick={refresh}>
              Actualiser
            </button>
            <button disabled={!allReady} onClick={onComplete}>
              C'est parti
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
