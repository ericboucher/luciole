import { useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { ipc, isTauriAvailable } from "./lib/ipc";
import { Sidebar } from "./components/Sidebar";
import { Dashboard } from "./views/Dashboard";
import { Notes } from "./views/Notes";
import { Glossary } from "./views/Glossary";
import { Settings } from "./views/Settings";

export type Route = "dashboard" | "notes" | "glossary" | "settings";

export function App() {
  const [tauriMissing, setTauriMissing] = useState(false);
  const [route, setRoute] = useState<Route>("dashboard");
  const [toast, setToast] = useState<string | null>(null);
  const [axGranted, setAxGranted] = useState<boolean | null>(null);
  const [micGranted, setMicGranted] = useState<boolean | null>(null);
  const [ollamaInstalled, setOllamaInstalled] = useState<boolean | null>(null);
  const [exePath, setExePath] = useState<string | null>(null);

  useEffect(() => {
    // minimal boot ping: if this fails, we are in browser (non-tauri)
    ipc.currentExePath().catch((err) => {
      if (String(err?.message || err) === "TAURI_NOT_AVAILABLE") {
        setTauriMissing(true);
        return;
      }
      console.error(err);
    });
  }, []);

  useEffect(() => {
    ipc.currentExePath().then(setExePath).catch(() => {});
  }, []);

  useEffect(() => {
    let mounted = true;
    const tick = async () => {
      try {
        const [axOk, micOk] = await Promise.all([
          ipc.checkAccessibilityPermission(),
          ipc.checkMicrophonePermission(),
        ]);
        if (!mounted) return;
        setAxGranted(axOk);
        setMicGranted(micOk);
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
    let mounted = true;
    const tick = async () => {
      try {
        const ok = await ipc.checkOllamaInstalled();
        if (mounted) setOllamaInstalled(ok);
      } catch {
        // ignore
      }
    };
    tick();
    const id = window.setInterval(tick, 5000);
    return () => {
      mounted = false;
      window.clearInterval(id);
    };
  }, []);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    (async () => {
      unlisten = await listen<string>("shortcut:text-action", async (event) => {
        try {
          const out = await ipc.runTextActionOnSelection(event.payload as any);
          setToast(`OK: ${out.slice(0, 80)}${out.length > 80 ? "…" : ""}`);
          setTimeout(() => setToast(null), 2500);
        } catch (e) {
          setToast(`Erreur: ${String(e)}`);
          setTimeout(() => setToast(null), 4000);
        }
      });
    })();
    return () => {
      unlisten?.();
    };
  }, []);

  if (tauriMissing || !isTauriAvailable()) {
    return (
      <div className="app">
        <div className="app-main">
          <h2>Luciole doit être lancée via Tauri</h2>
          <p>
            Tu as probablement ouvert <code>http://localhost:1420</code> dans un
            navigateur. Lance plutôt l&apos;app avec <code>npm run tauri:dev</code>{" "}
            et ouvre la fenêtre via l&apos;icône de la barre de menu macOS.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="app">
      <div className="app-shell">
        <Sidebar route={route} onNavigate={setRoute} />
        <main className="app-main">
          {axGranted === false || micGranted === false || ollamaInstalled === false ? (
            <div className="card" style={{ marginBottom: 16 }}>
              <strong>Configuration requise</strong>
              <div style={{ marginTop: 6, fontSize: 13, color: "var(--color-text-muted)" }}>
                Active permissions pour utiliser overlay (“+”), dictée, et actions.
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
                  {ollamaInstalled === null
                    ? "…"
                    : ollamaInstalled
                      ? "OK"
                      : "NON"}
                </div>
              </div>
              {exePath ? (
                <div style={{ marginTop: 10, fontSize: 12 }}>
                  Binaire en cours:
                  <div style={{ marginTop: 6 }}>
                    <code>{exePath}</code>
                  </div>
                </div>
              ) : null}
              <div className="row" style={{ gap: 8, marginTop: 10 }}>
                {axGranted === false ? (
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
              </div>
              <div style={{ marginTop: 8, fontSize: 12, color: "var(--color-text-muted)" }}>
                Après activation, <strong>quitte Luciole</strong> puis relance
                (macOS applique parfois permission au redémarrage).
              </div>
            </div>
          ) : null}
          {toast ? (
            <div className="card" style={{ marginBottom: 16 }}>
              <strong>Action</strong>
              <div style={{ marginTop: 6, fontSize: 13 }}>{toast}</div>
            </div>
          ) : null}
          {route === "dashboard" && <Dashboard />}
          {route === "notes" && <Notes />}
          {route === "glossary" && <Glossary />}
          {route === "settings" && <Settings />}
        </main>
      </div>
    </div>
  );
}
