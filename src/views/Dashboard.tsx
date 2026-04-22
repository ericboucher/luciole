import { useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { ipc } from "../lib/ipc";

export function Dashboard() {
  const [recording, setRecording] = useState(false);
  const [lastNote, setLastNote] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

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
