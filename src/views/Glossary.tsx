import { useEffect, useState } from "react";
import { ipc, type GlossaryEntry } from "../lib/ipc";

export function Glossary() {
  const [entries, setEntries] = useState<GlossaryEntry[]>([]);
  const [short, setShort] = useState("");
  const [full, setFull] = useState("");
  const [filter, setFilter] = useState("");

  const reload = async () => {
    setEntries(await ipc.listGlossary());
  };

  useEffect(() => {
    reload().catch(console.error);
  }, []);

  const add = async () => {
    if (!short.trim() || !full.trim()) return;
    await ipc.addGlossaryEntry(short.trim(), full.trim());
    setShort("");
    setFull("");
    reload();
  };

  const remove = async (entry: GlossaryEntry) => {
    await ipc.removeGlossaryEntry(entry.short);
    reload();
  };

  const filtered = filter.trim()
    ? entries.filter(
        (e) =>
          e.short.toLowerCase().includes(filter.toLowerCase()) ||
          e.full.toLowerCase().includes(filter.toLowerCase()),
      )
    : entries;

  return (
    <div>
      <h1>Glossaire</h1>
      <p>
        Les acronymes de ce glossaire sont injectés dans le prompt Gemma 4 à
        chaque appel. Pipeline deux passes (regex + contexte LLM).
      </p>

      <div className="card">
        <h2>Ajouter un acronyme</h2>
        <div className="row" style={{ alignItems: "flex-end" }}>
          <label style={{ flex: "0 0 140px" }}>
            <div style={{ fontSize: 12, color: "var(--color-text-muted)" }}>
              Forme courte
            </div>
            <input
              type="text"
              placeholder="DINUM"
              value={short}
              onChange={(e) => setShort(e.target.value)}
            />
          </label>
          <label style={{ flex: 1 }}>
            <div style={{ fontSize: 12, color: "var(--color-text-muted)" }}>
              Forme complète
            </div>
            <input
              type="text"
              placeholder="Direction Interministérielle du Numérique"
              value={full}
              onChange={(e) => setFull(e.target.value)}
            />
          </label>
          <button onClick={add}>Ajouter</button>
        </div>
      </div>

      <div className="card">
        <div className="row between" style={{ marginBottom: 12 }}>
          <h2 style={{ margin: 0 }}>Acronymes ({entries.length})</h2>
          <input
            type="search"
            placeholder="Filtrer…"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            style={{ maxWidth: 240 }}
          />
        </div>
        <div className="stack" style={{ gap: 4 }}>
          {filtered.map((entry) => (
            <div
              key={entry.short}
              className="row between"
              style={{
                padding: "8px 12px",
                borderBottom: "1px solid var(--color-border)",
              }}
            >
              <div>
                <strong>{entry.short}</strong>
                <span style={{ color: "var(--color-text-muted)", marginLeft: 8 }}>
                  {entry.full}
                </span>
              </div>
              <button className="ghost" onClick={() => remove(entry)}>
                Retirer
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
