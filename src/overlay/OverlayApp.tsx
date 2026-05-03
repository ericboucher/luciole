import { useEffect, useMemo, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";
import { ipc } from "../lib/ipc";

type AnchorRect = { x: number; y: number; width: number; height: number };
type OverlayState = { visible: boolean; anchor?: AnchorRect | null };

type Mode = "instant" | "chat";

const LANGS: { id: string; label: string }[] = [
  { id: "en", label: "English" },
  { id: "fr", label: "Français" },
  { id: "es", label: "Español" },
  { id: "de", label: "Deutsch" },
];

export function OverlayApp() {
  const win = useMemo(() => getCurrentWindow(), []);
  const [overlay, setOverlay] = useState<OverlayState>({ visible: false });
  const [open, setOpen] = useState(false);
  const [mode, setMode] = useState<Mode>("instant");
  const [busy, setBusy] = useState(false);
  const [toast, setToast] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    (async () => {
      unlisten = await listen<OverlayState>("overlay:state", async (event) => {
        const next = event.payload ?? { visible: false };
        setOverlay(next);

        if (!next.visible || !next.anchor) {
          setOpen(false);
          await win.hide();
          return;
        }

        // Position right next to selection (end).
        // Note: On macOS, Tauri window positions match screen coordinates,
        // so we do not invert Y here.
        const padX = +200;
        const padY = +200;
        const x = Math.round(next.anchor.x + padX);
        // Align vertically near selection center.
        const y = Math.round(next.anchor.y + padY);
        await win.setPosition(new PhysicalPosition(x, y));
        await win.show();
      });
    })();
    return () => unlisten?.();
  }, [win]);

  const run = async (
    action:
      | "correct"
      | "translate"
      | "rephrase"
      | "clean-dictation",
    opts?: { targetLanguage?: string; tone?: "formel" | "informel" },
  ) => {
    setBusy(true);
    setToast(null);
    try {
      await ipc.runTextActionOnSelection(action, opts);
      setOpen(false);
      await win.hide();
    } catch (e) {
      setToast(String(e));
      setTimeout(() => setToast(null), 3500);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="overlay-app">
      {/* plus button */}
      <button
        onClick={() => setOpen((v) => !v)}
        disabled={busy || !overlay.visible}
        style={{
          width: 24,
          height: 24,
          borderRadius: 999,
          border: "1px solid rgba(0,0,0,0.18)",
          background: "#000091",
          color: "white",
          fontSize: 16,
          lineHeight: "24px",
          padding: 0,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          position: "absolute",
          right: 8,
          top: 8,
          boxShadow: "none",
          cursor: "pointer",
        }}
        aria-label="Luciole actions"
      >
        +
      </button>

      {/* popover */}
      {open ? (
        <div
          style={{
            position: "absolute",
            right: 8,
            top: 32,
            width: 260,
            borderRadius: 14,
            background: "white",
            border: "1px solid rgba(0,0,0,0.12)",
            boxShadow: "0 18px 50px rgba(0,0,0,0.25)",
            overflow: "hidden",
          }}
        >
          <div style={{ display: "flex", gap: 0 }}>
            <TabButton
              active={mode === "instant"}
              onClick={() => setMode("instant")}
              label="Instant"
            />
            <TabButton
              active={mode === "chat"}
              onClick={() => setMode("chat")}
              label="Chat"
            />
          </div>

          {mode === "instant" ? (
            <div style={{ padding: 8 }}>
              <MenuButton
                label="Grammar"
                onClick={() => run("correct")}
                disabled={busy}
              />
              <MenuButton
                label="Polish"
                onClick={() => run("rephrase", { tone: "formel" })}
                disabled={busy}
              />
              <MenuButton
                label="Custom Request"
                onClick={() => {
                  setToast("Custom request: next step (not wired yet)");
                }}
                disabled={busy}
              />
              <div style={{ marginTop: 6 }}>
                <div
                  style={{
                    fontSize: 12,
                    color: "rgba(0,0,0,0.55)",
                    padding: "6px 8px",
                  }}
                >
                  Translate
                </div>
                <div style={{ display: "flex", flexWrap: "wrap", gap: 6, padding: "0 8px 6px" }}>
                  {LANGS.map((l) => (
                    <button
                      key={l.id}
                      onClick={() => run("translate", { targetLanguage: l.id })}
                      disabled={busy}
                      style={{
                        borderRadius: 999,
                        border: "1px solid rgba(0,0,0,0.14)",
                        background: "white",
                        padding: "6px 10px",
                        fontSize: 12,
                        cursor: "pointer",
                      }}
                    >
                      {l.label}
                    </button>
                  ))}
                </div>
              </div>

              {toast ? (
                <div
                  style={{
                    marginTop: 8,
                    fontSize: 12,
                    color: "rgba(0,0,0,0.7)",
                    padding: "8px 10px",
                    background: "rgba(0,0,0,0.04)",
                    borderRadius: 10,
                  }}
                >
                  {toast}
                </div>
              ) : null}
            </div>
          ) : (
            <div style={{ padding: 10 }}>
              <div style={{ fontSize: 13, color: "rgba(0,0,0,0.7)" }}>
                Chat MVP: opens Luciole main window later.
              </div>
              <button
                onClick={() => setToast("Chat: next step (not wired yet)")}
                disabled={busy}
                style={{
                  marginTop: 10,
                  width: "100%",
                  borderRadius: 10,
                  border: "1px solid rgba(0,0,0,0.14)",
                  background: "white",
                  padding: "10px 12px",
                  cursor: "pointer",
                }}
              >
                Open chat
              </button>
              {toast ? (
                <div style={{ marginTop: 10, fontSize: 12, color: "rgba(0,0,0,0.7)" }}>
                  {toast}
                </div>
              ) : null}
            </div>
          )}
        </div>
      ) : null}
    </div>
  );
}

function TabButton({
  active,
  onClick,
  label,
}: {
  active: boolean;
  onClick: () => void;
  label: string;
}) {
  return (
    <button
      onClick={onClick}
      style={{
        flex: 1,
        padding: "10px 12px",
        fontWeight: 700,
        fontSize: 13,
        border: "none",
        background: active ? "rgba(0,0,0,0.06)" : "white",
        cursor: "pointer",
      }}
    >
      {label}
    </button>
  );
}

function MenuButton({
  label,
  onClick,
  disabled,
}: {
  label: string;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        width: "100%",
        textAlign: "left",
        padding: "10px 10px",
        borderRadius: 10,
        border: "none",
        background: "white",
        cursor: "pointer",
      }}
      onMouseEnter={(e) => {
        (e.currentTarget as HTMLButtonElement).style.background = "rgba(0,0,0,0.04)";
      }}
      onMouseLeave={(e) => {
        (e.currentTarget as HTMLButtonElement).style.background = "white";
      }}
    >
      {label}
    </button>
  );
}

