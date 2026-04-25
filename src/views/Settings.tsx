import { useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { ipc, type Settings as SettingsT } from "../lib/ipc";

type OverlayState = {
  visible: boolean;
  anchor?: { x: number; y: number; width: number; height: number } | null;
};

export function Settings() {
  const [settings, setSettings] = useState<SettingsT | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [axGranted, setAxGranted] = useState<boolean | null>(null);
  const [micGranted, setMicGranted] = useState<boolean | null>(null);
  const [ollamaInstalled, setOllamaInstalled] = useState<boolean | null>(null);
  const [overlayState, setOverlayState] = useState<OverlayState | null>(null);
  const [exePath, setExePath] = useState<string | null>(null);

  useEffect(() => {
    ipc.getSettings().then(setSettings).catch(console.error);
  }, []);

  useEffect(() => {
    let mounted = true;
    const tick = async () => {
      try {
        const [axOk, micOk, ollamaOk] = await Promise.all([
          ipc.checkAccessibilityPermission(),
          ipc.checkMicrophonePermission(),
          ipc.checkOllamaInstalled(),
        ]);
        if (!mounted) return;
        setAxGranted(axOk);
        setMicGranted(micOk);
        setOllamaInstalled(ollamaOk);
      } catch {
        // ignore
      }
    };
    tick();
    const id = window.setInterval(tick, 2000);
    return () => {
      mounted = false;
      window.clearInterval(id);
    };
  }, []);

  useEffect(() => {
    ipc.currentExePath().then(setExePath).catch(() => {});
  }, []);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    (async () => {
      unlisten = await listen<OverlayState>("overlay:state", (event) => {
        setOverlayState(event.payload ?? null);
      });
    })();
    return () => unlisten?.();
  }, []);

  if (!settings) {
    return <p>Chargement…</p>;
  }

  const patch = (next: Partial<SettingsT>) =>
    setSettings({ ...settings, ...next });

  const save = async () => {
    setSaving(true);
    try {
      await ipc.updateSettings(settings);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } finally {
      setSaving(false);
    }
  };

  const pickVault = async () => {
    const picked = await open({ directory: true, multiple: false });
    if (typeof picked === "string") {
      patch({ obsidianVaultPath: picked });
    }
  };

  return (
    <div>
      <h1>Réglages</h1>

      <div className="card">
        <h2>Setup</h2>
        <div style={{ fontSize: 13, color: "var(--color-text-muted)" }}>
          Permissions et dépendances système.
        </div>
        <div style={{ marginTop: 10, fontSize: 13 }}>
          <div>
            <strong>Accessibilité</strong>:{" "}
            {axGranted === null ? "…" : axGranted ? "OK" : "NON"}
          </div>
          <div style={{ marginTop: 6 }}>
            <strong>Microphone</strong>:{" "}
            {micGranted === null ? "…" : micGranted ? "OK" : "NON"}
          </div>
          <div style={{ marginTop: 6 }}>
            <strong>Ollama</strong>:{" "}
            {ollamaInstalled === null ? "…" : ollamaInstalled ? "OK" : "NON"}
          </div>
        </div>
        <div className="row" style={{ gap: 8, marginTop: 12, flexWrap: "wrap" }}>
          {!axGranted ? (
            <>
              <button
                className="secondary"
                onClick={() => ipc.openSystemSettings("accessibility")}
              >
                Réglages →
              </button>
              <button
                className="secondary"
                onClick={() => ipc.requestAccessibilityPrompt().catch(() => {})}
              >
                Demander…
              </button>
            </>
          ) : null}
          {micGranted === false ? (
            <>
              <button
                className="secondary"
                onClick={() => ipc.openSystemSettings("microphone")}
              >
                Micro →
              </button>
              <button
                className="secondary"
                onClick={() => ipc.requestMicrophonePrompt().catch(() => {})}
              >
                Demander micro…
              </button>
            </>
          ) : null}
          {ollamaInstalled === false ? (
            <button
              className="secondary"
              onClick={() => window.open("https://ollama.com/download", "_blank")}
            >
              Installer Ollama →
            </button>
          ) : null}
        </div>
      </div>

      <div className="card">
        <h2>Diagnostic overlay</h2>
        <div style={{ fontSize: 13, color: "var(--color-text-muted)" }}>
          Pour debug bouton “+”.
        </div>
        <div style={{ marginTop: 10, fontSize: 13 }}>
          <div>
            <strong>Accessibilité</strong>:{" "}
            {axGranted === null ? "…" : axGranted ? "OK" : "NON"}
          </div>
          {exePath ? (
            <div style={{ marginTop: 6 }}>
              <strong>Binaire</strong>: <code>{exePath}</code>
            </div>
          ) : null}
          <div style={{ marginTop: 6 }}>
            <strong>overlay:state</strong>:{" "}
            <code>{overlayState ? JSON.stringify(overlayState) : "—"}</code>
          </div>
        </div>
        {!axGranted ? (
          <div className="row" style={{ gap: 8, marginTop: 12 }}>
            <button
              className="secondary"
              onClick={() => ipc.openSystemSettings("accessibility")}
            >
              Réglages →
            </button>
            <button
              className="secondary"
              onClick={() => ipc.requestAccessibilityPrompt().catch(() => {})}
            >
              Demander…
            </button>
          </div>
        ) : null}
      </div>

      <div className="card">
        <h2>Langue</h2>
        <label>
          <div style={{ fontSize: 12, color: "var(--color-text-muted)" }}>
            Langue principale pour la transcription et les réponses
          </div>
          <select
            value={settings.language}
            onChange={(e) => patch({ language: e.target.value })}
          >
            <option value="fr">Français</option>
            <option value="en">English</option>
          </select>
        </label>
      </div>

      <div className="card">
        <h2>Modèles</h2>
        <div className="stack">
          <label>
            <div style={{ fontSize: 12, color: "var(--color-text-muted)" }}>
              Modèle Whisper
            </div>
            <select
              value={settings.whisperModel}
              onChange={(e) => patch({ whisperModel: e.target.value })}
            >
              <option value="small">small — 460 Mo, rapide</option>
              <option value="large-v3-turbo">
                large-v3-turbo — meilleure précision
              </option>
            </select>
          </label>
          <label>
            <div style={{ fontSize: 12, color: "var(--color-text-muted)" }}>
              Modèle LLM (Ollama)
            </div>
            <input
              type="text"
              value={settings.ollamaModel}
              onChange={(e) => patch({ ollamaModel: e.target.value })}
              placeholder="gemma4:e4b"
            />
          </label>
        </div>
      </div>

      <div className="card">
        <h2>Vault Obsidian</h2>
        <p>
          Les notes sont écrites dans ce dossier. Les wiki-links
          <span className="kbd">[[…]]</span> sont immédiatement résolubles.
        </p>
        <div className="row">
          <input
            type="text"
            value={settings.obsidianVaultPath ?? ""}
            onChange={(e) =>
              patch({ obsidianVaultPath: e.target.value || null })
            }
            placeholder="/Users/moi/Documents/Obsidian Vault"
          />
          <button className="secondary" onClick={pickVault}>
            Parcourir…
          </button>
        </div>
      </div>

      <div className="card">
        <h2>Raccourcis</h2>
        <div className="stack">
          <ShortcutField
            label="Dictée"
            value={settings.shortcutDictate}
            onChange={(v) => patch({ shortcutDictate: v })}
          />
          <ShortcutField
            label="Corriger"
            value={settings.shortcutCorrect}
            onChange={(v) => patch({ shortcutCorrect: v })}
          />
          <ShortcutField
            label="Traduire"
            value={settings.shortcutTranslate}
            onChange={(v) => patch({ shortcutTranslate: v })}
          />
          <ShortcutField
            label="Reformuler"
            value={settings.shortcutRephrase}
            onChange={(v) => patch({ shortcutRephrase: v })}
          />
          <ShortcutField
            label="Démarrer / arrêter une réunion"
            value={settings.shortcutMeeting}
            onChange={(v) => patch({ shortcutMeeting: v })}
          />
        </div>
      </div>

      <div className="row between">
        <span style={{ color: "var(--color-text-muted)", fontSize: 13 }}>
          {saved ? "Enregistré." : ""}
        </span>
        <button disabled={saving} onClick={save}>
          {saving ? "Enregistrement…" : "Enregistrer"}
        </button>
      </div>
    </div>
  );
}

function ShortcutField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <label className="row between" style={{ alignItems: "center" }}>
      <span style={{ flex: 1 }}>{label}</span>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        style={{ maxWidth: 200 }}
      />
    </label>
  );
}
