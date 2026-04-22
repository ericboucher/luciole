import type { Route } from "../App";

const LINKS: { key: Route; label: string }[] = [
  { key: "dashboard", label: "Accueil" },
  { key: "notes", label: "Mes notes" },
  { key: "glossary", label: "Glossaire" },
  { key: "settings", label: "Réglages" },
];

export function Sidebar({
  route,
  onNavigate,
}: {
  route: Route;
  onNavigate: (r: Route) => void;
}) {
  return (
    <aside className="app-sidebar">
      <div className="sidebar-brand">
        <FireflyMark />
        <span>Luciole</span>
      </div>
      {LINKS.map((link) => (
        <button
          key={link.key}
          className={`sidebar-link ${route === link.key ? "active" : ""}`}
          onClick={() => onNavigate(link.key)}
        >
          {link.label}
        </button>
      ))}
    </aside>
  );
}

function FireflyMark() {
  return (
    <svg
      width="22"
      height="22"
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden
    >
      <circle cx="12" cy="14" r="5" fill="currentColor" />
      <path
        d="M7 9 L4 6 M17 9 L20 6"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
      />
    </svg>
  );
}
