import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { ipc, type Settings as SettingsT } from "../lib/ipc";

export function Settings() {
  const [settings, setSettings] = useState<SettingsT | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    ipc.getSettings().then(setSettings).catch(console.error);
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
