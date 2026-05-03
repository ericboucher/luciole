import { useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { ipc } from "../lib/ipc";

export function Dashboard() {
  const [recording, setRecording] = useState(false);
  const [lastNote, setLastNote] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [whisper, setWhisper] = useState<{
    binaryPath: string | null;
    modelPath: string;
    modelExists: boolean;
  } | null>(null);
  const [transcriptTest, setTranscriptTest] = useState<string | null>(null);
  const [transcriptError, setTranscriptError] = useState<string | null>(null);
  const [transcribing, setTranscribing] = useState(false);
  const [micLevel, setMicLevel] = useState(0);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    (async () => {
      unlisten = await listen<boolean>("shortcut:meeting-toggle", (event) => {
        setRecording(event.payload);
      });
    })();
    return () => {
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    (async () => {
      unlisten = await listen<{ level: number }>(
        "transcription-test-level",
        (event) => {
          setMicLevel(typeof event.payload.level === "number" ? event.payload.level : 0);
        },
      );
    })();
    return () => {
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    let mounted = true;
    const load = () => {
      ipc
        .whisperStatus()
        .then((s) => {
          if (mounted) setWhisper(s);
        })
        .catch(() => {});
    };
    load();
    const id = window.setInterval(load, 8000);
    return () => {
      mounted = false;
      window.clearInterval(id);
    };
  }, []);

  const runTranscriptionTest = async () => {
    setTranscriptError(null);
    setTranscriptTest(null);
    setMicLevel(0);
    try {
      setTranscribing(true);
      const text = await ipc.transcribeMicrophoneTest(8);
      setTranscriptTest(text);
    } catch (e) {
      setTranscriptError(String(e));
    } finally {
      setMicLevel(0);
      setTranscribing(false);
    }
  };

  const toggleMeeting = async () => {
    setError(null);
    try {
      if (!recording) {
        await ipc.startMeeting();
        setRecording(true);
      } else {
        const result = await ipc.stopMeeting();
        setRecording(false);
        setLastNote(result.notePath);
      }
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div>
      <h1>Bonjour</h1>
      <p>Voici les raccourcis principaux de Luciole.</p>

      <div className="card">
        <h2>Raccourcis</h2>
        <div className="stack">
          <ShortcutRow
            keys={["⌥"]}
            label="Maintenez pour dicter — le texte est injecté au curseur."
          />
          <ShortcutRow
            keys={["⌥", "P"]}
            label="Corriger la sélection de texte."
          />
          <ShortcutRow
            keys={["⌥", "T"]}
            label="Traduire la sélection."
          />
          <ShortcutRow
            keys={["⌥", "R"]}
            label="Reformuler la sélection."
          />
          <ShortcutRow
            keys={["⌥", "M"]}
            label="Démarrer / arrêter un enregistrement de réunion."
          />
        </div>
      </div>

      <div className="card">
        <h2 style={{ marginBottom: 8 }}>Transcription</h2>
        <p style={{ marginTop: 0, color: "var(--color-text-muted)", fontSize: 14 }}>
          Enregistre <strong>8 secondes</strong> via le microphone puis transcription locale (Whisper
          packagé dans l&apos;app). Modèle : <code>ggml-tiny.bin</code> dans les ressources.
        </p>
        {whisper ? (
          <div style={{ fontSize: 13, marginBottom: 12 }}>
            <div>
              <strong>Binaire whisper</strong> :{" "}
              {whisper.binaryPath ? (
                <code style={{ fontSize: 12 }}>{whisper.binaryPath}</code>
              ) : (
                <span style={{ color: "var(--color-danger)" }}>
                  introuvable — exécute <code>scripts/vendor-whisper-macos.sh</code> puis relance
                  (voir <code>src-tauri/binaries/README.md</code>)
                </span>
              )}
            </div>
            <div style={{ marginTop: 6 }}>
              <strong>Modèle</strong> : <code style={{ fontSize: 12 }}>{whisper.modelPath}</code>{" "}
              {whisper.modelExists ? (
                <span style={{ color: "#18753c" }}> OK</span>
              ) : (
                <span style={{ color: "var(--color-danger)" }}> absent</span>
              )}
            </div>
          </div>
        ) : null}
        <TranscriptionWave level={micLevel} active={transcribing} />
        <button type="button" onClick={runTranscriptionTest} disabled={transcribing}>
          {transcribing ? "Écoute + transcription…" : "Tester la transcription (micro, 8 s)"}
        </button>
        {transcriptError ? (
          <p style={{ marginTop: 12, color: "var(--color-danger)", whiteSpace: "pre-wrap" }}>
            {transcriptError}
          </p>
        ) : null}
        {transcriptTest !== null ? (
          <div style={{ marginTop: 14 }}>
            <div style={{ fontSize: 13, marginBottom: 6 }}>
              <strong>Résultat</strong>
            </div>
            <textarea
              readOnly
              value={transcriptTest}
              rows={6}
              style={{
                width: "100%",
                boxSizing: "border-box",
                fontFamily: "inherit",
                fontSize: 14,
                padding: 10,
                borderRadius: 8,
                border: "1px solid rgba(0,0,0,0.12)",
              }}
            />
          </div>
        ) : null}
      </div>

      <div className="card">
        <div className="row between">
          <div>
            <h2 style={{ marginBottom: 4 }}>Réunion</h2>
            <p style={{ margin: 0 }}>
              {recording
                ? "Enregistrement en cours — parlez normalement."
                : "Prêt à enregistrer. L'audio reste sur votre Mac."}
            </p>
          </div>
          <button onClick={toggleMeeting}>
            {recording ? "Arrêter" : "Démarrer"}
          </button>
        </div>
        {lastNote && (
          <p style={{ marginTop: 16, fontSize: 13 }}>
            Dernière note : <code>{lastNote}</code>
          </p>
        )}
        {error && (
          <p style={{ marginTop: 16, color: "var(--color-danger)" }}>
            {error}
          </p>
        )}
      </div>
    </div>
  );
}

/** Bandeau onde — barres animées par niveau RMS envoyé pendant l’enregistrement test. */
function TranscriptionWave({ level, active }: { level: number; active: boolean }) {
  const bars = 28;
  const safe = Number.isFinite(level) ? Math.min(1, Math.max(0, level)) : 0;
  return (
    <div
      role="img"
      aria-label={active ? "Niveau du microphone" : "Visualisation microphone au repos"}
      style={{
        display: "flex",
        alignItems: "flex-end",
        justifyContent: "center",
        gap: 3,
        height: 52,
        marginBottom: 14,
        padding: "10px 12px",
        borderRadius: 8,
        background: "var(--color-bg-alt)",
        border: "1px solid var(--color-border)",
        opacity: active ? 1 : 0.65,
        transition: "opacity 0.2s ease",
      }}
    >
      {Array.from({ length: bars }, (_, i) => {
        const phase = (i / bars) * Math.PI * 2;
        const envelope = 0.38 + 0.62 * (0.5 + 0.5 * Math.sin(phase + 0.7));
        const h =
          active && safe > 0.005
            ? 6 + safe * 38 * envelope
            : active
              ? 5 + safe * 18 * envelope
              : 4;
        return (
          <span
            key={i}
            style={{
              display: "block",
              width: 5,
              height: Math.max(3, h),
              borderRadius: 2,
              background:
                active && safe > 0.02 ? "var(--color-accent)" : "var(--luciole-grey-300)",
              transition: "height 0.06s ease-out, background 0.12s ease",
            }}
          />
        );
      })}
    </div>
  );
}

function ShortcutRow({ keys, label }: { keys: string[]; label: string }) {
  return (
    <div className="row">
      <div className="row" style={{ gap: 4, minWidth: 96 }}>
        {keys.map((k, i) => (
          <span className="kbd" key={i}>
            {k}
          </span>
        ))}
      </div>
      <span style={{ color: "var(--color-text-muted)" }}>{label}</span>
    </div>
  );
}
