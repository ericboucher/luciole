import { useEffect, useState } from "react";
import { ipc, isTauriAvailable, type OnboardingStatus } from "./lib/ipc";
import { Sidebar } from "./components/Sidebar";
import { Onboarding } from "./views/Onboarding";
import { Dashboard } from "./views/Dashboard";
import { Notes } from "./views/Notes";
import { Glossary } from "./views/Glossary";
import { Settings } from "./views/Settings";

export type Route = "dashboard" | "notes" | "glossary" | "settings";

export function App() {
  const [status, setStatus] = useState<OnboardingStatus | null>(null);
  const [tauriMissing, setTauriMissing] = useState(false);
  const [route, setRoute] = useState<Route>("dashboard");

  useEffect(() => {
    ipc
      .onboardingStatus()
      .then(setStatus)
      .catch((err) => {
        if (String(err?.message || err) === "TAURI_NOT_AVAILABLE") {
          setTauriMissing(true);
          return;
        }
        console.error(err);
      });
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

  if (!status) {
    return (
      <div className="app">
        <div className="app-main">
          <p>Chargement…</p>
        </div>
      </div>
    );
  }

  if (!status.onboardingCompleted) {
    return (
      <Onboarding
        initial={status}
        onComplete={async () => {
          await ipc.completeOnboarding();
          setStatus({ ...status, onboardingCompleted: true });
        }}
      />
    );
  }

  return (
    <div className="app">
      <div className="app-shell">
        <Sidebar route={route} onNavigate={setRoute} />
        <main className="app-main">
          {route === "dashboard" && <Dashboard />}
          {route === "notes" && <Notes />}
          {route === "glossary" && <Glossary />}
          {route === "settings" && <Settings />}
        </main>
      </div>
    </div>
  );
}
