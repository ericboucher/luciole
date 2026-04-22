# Luciole

> Compagnon IA local et souverain pour l'agent public.
> macOS · Offline-first · Apache 2.0

Luciole est une application macOS native qui couvre la boucle complète de
productivité — dictée, correction, traduction, et comptes-rendus de réunion —
entièrement hors ligne, sans aucune configuration terminal.

Le nom s'inspire de la luciole : un insecte qui produit sa propre lumière,
sans source externe. L'IA tourne localement, génère sa propre intelligence.

**État :** scaffold v0.1 — Avril 2026. Voir [`PRD.md`](./PRD.md) pour la
spécification complète.

---

## Ce qui est dans le dépôt aujourd'hui

- Structure Tauri v2 (Rust + React/TypeScript)
- Modules Rust : paramètres, glossaire, état, raccourcis globaux, tray menu bar,
  squelettes `asr` (whisper.cpp), `llm` (Ollama / Gemma 4 E4B), `meeting`
  (capture + diarisation), `system` (permissions, injection, sélection)
- Vues React : onboarding wizard, dashboard, notes, glossaire, réglages
- Tokens DSFR (palette et typographie Marianne) en CSS
- Entitlements macOS (micro, accessibilité, AppleEvents)
- `Info.plist` avec `LSUIElement = true` (app menu bar sans dock)
- Glossaire public-secteur par défaut (`glossaire-secteur-public.yaml`)

Pas encore câblé (pistes de contribution — cf. PRD §12) :
- Binaire `whisper.cpp` bundlé et pipeline de téléchargement des modèles
- Détection d'installation Ollama et téléchargement de `gemma4:e4b`
- Poids `sherpa-onnx` + `pyannote community-1` + `CAM++` bundlés
- Capture audio `AVFoundation` simultanée mic + système
- Injection clipboard et lecture de sélection via Accessibility API
- Hold-to-dictate via `CGEventTap` bas niveau
- Fenêtre flottante "pill" avec waveform

---

## Stack

| Couche | Choix | Pourquoi |
|---|---|---|
| Framework | Tauri v2 | Bundle léger, Rust côté système, React côté UI |
| ASR | whisper.cpp | Offline, bundlable, multilingue, rapide |
| LLM | Gemma 4 E4B via Ollama | Stock Apache 2.0, Metal sur Apple Silicon |
| Diarisation | sherpa-onnx + pyannote + CAM++ | Pas de Python, poids bundlés, pas de compte HF |
| Design | DSFR (Système de Design de l'État) | Compatible La Suite Numérique |
| Stockage | Fichiers plats + SQLite | L'utilisateur possède ses données |

---

## Développement

Prérequis :
- Rust stable (`rustup`)
- Node.js ≥ 20 et npm
- Xcode Command Line Tools (macOS)

```sh
npm install
npm run tauri:dev
```

La première compilation Rust prend plusieurs minutes. Les modèles IA ne sont
pas téléchargés automatiquement par `tauri:dev` — le wizard d'onboarding gère
cette étape au runtime.

### Layout

```
luciole/
  PRD.md                         Spécification produit
  package.json                   Frontend (React + Vite)
  vite.config.ts
  tsconfig.json
  index.html
  src/                           React / TypeScript
    main.tsx                     Entrée
    App.tsx                      Routeur minimaliste
    components/
      Sidebar.tsx
    views/
      Onboarding.tsx             Wizard premier lancement
      Dashboard.tsx
      Notes.tsx
      Glossary.tsx
      Settings.tsx
    lib/
      ipc.ts                     Wrapper typé autour de invoke()
    styles/
      tokens.css                 Design tokens DSFR
      app.css
  src-tauri/                     Backend Rust
    Cargo.toml
    tauri.conf.json              Config Tauri v2
    Info.plist                   LSUIElement, descriptions permissions
    entitlements.plist           Micro, accessibilité, réseau
    build.rs
    capabilities/default.json    Permissions du runtime Tauri
    resources/
      glossaire-secteur-public.yaml
    src/
      main.rs
      lib.rs                     Bootstrap, plugins, tray, shortcuts
      commands.rs                Surface invoke()
      settings.rs                ~/.luciole/settings.json
      glossary.rs                Glossaire YAML + prompt injection
      state.rs                   État partagé Tauri
      paths.rs                   Layout ~/.luciole/
      shortcuts.rs               Raccourcis globaux
      tray.rs                    Menu bar macOS
      asr.rs                     whisper.cpp (stub)
      llm.rs                     Ollama / Gemma 4
      meeting.rs                 Pipeline réunion
      system/
        mod.rs
        permissions.rs           Micro + accessibilité (AX)
        injection.rs             Clipboard → Cmd+V (stub)
        selection.rs             AXSelectedText (stub)
```

---

## Licence

Apache License 2.0. Voir [`LICENSE`](./LICENSE).

## Contribuer

Le projet est en phase de scaffold. Contributions bienvenues, en particulier
sur :
1. Intégration `whisper.cpp` (binaire bundlé, streaming)
2. Wizard de téléchargement Ollama + `gemma4:e4b`
3. Pipeline `sherpa-onnx` pour la diarisation
4. Hold-to-dictate via `CGEventTap`
5. Composants DSFR officiels (`@codegouvfr/react-dsfr`)

Ouvrez une issue avant de commencer un gros chantier pour aligner
l'architecture.
