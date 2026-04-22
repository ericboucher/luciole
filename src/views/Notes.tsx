import { useEffect, useState } from "react";
import { ipc, type Note } from "../lib/ipc";

export function Notes() {
  const [notes, setNotes] = useState<Note[]>([]);
  const [selected, setSelected] = useState<Note | null>(null);
  const [content, setContent] = useState<string>("");

  useEffect(() => {
    ipc.listNotes().then(setNotes).catch(console.error);
  }, []);

  useEffect(() => {
    if (!selected) {
      setContent("");
      return;
    }
    ipc.readNote(selected.path).then(setContent).catch(console.error);
  }, [selected]);

  return (
    <div>
      <h1>Mes notes</h1>
      <p>Notes générées localement. Stockées sous <code>~/.luciole/notes/</code>.</p>

      <div style={{ display: "grid", gridTemplateColumns: "240px 1fr", gap: 16 }}>
        <div className="stack" style={{ gap: 4 }}>
          {notes.length === 0 && (
            <p style={{ fontSize: 13 }}>Aucune note pour l'instant.</p>
          )}
          {notes.map((note) => (
            <button
              key={note.path}
              className={`sidebar-link ${selected?.path === note.path ? "active" : ""}`}
              onClick={() => setSelected(note)}
            >
              <div style={{ fontWeight: 600, fontSize: 13 }}>{note.title}</div>
              <div style={{ fontSize: 11, opacity: 0.7 }}>
                {new Date(note.createdAt).toLocaleString("fr-FR")}
              </div>
            </button>
          ))}
        </div>
        <div className="card" style={{ minHeight: 400 }}>
          {selected ? (
            <pre style={{ whiteSpace: "pre-wrap" }}>{content}</pre>
          ) : (
            <p>Sélectionnez une note pour la lire.</p>
          )}
        </div>
      </div>
    </div>
  );
}
