# PRD — **Luciole**
> Companion IA local et souverain pour l'agent public  
> Open source · Offline-first · macOS  
> Status: Draft v0.2 — April 2026

---

## 1. Problem Statement

Les outils de productivité IA cloud (Snaply, Wispr Flow, Otter, Fireflies) exigent l'envoi d'enregistrements vocaux et de textes sensibles vers des serveurs tiers. Pour les utilisateurs en environnement réglementé ou sensible — gouvernement, santé, juridique, entreprise — c'est un blocage, pas un compromis.

Les alternatives open source existantes soit :
- Nécessitent un terminal et Docker (Meetily, OpenWhisper)
- Ne couvrent qu'un seul usage (dictée *ou* compte-rendu, pas les deux)
- Manquent de support vocabulaire métier (acronymes, jargon institutionnel)
- Ne produisent pas d'output structuré et portable

**Luciole** est une application macOS native qui couvre la boucle complète de productivité — dictée, correction, traduction, et comptes-rendus de réunion — entièrement hors ligne, sans aucune configuration terminal requise.

Le nom s'inspire de la luciole : un insecte qui produit sa propre lumière, sans source externe. L'IA tourne localement, génère sa propre intelligence.

---

## 2. Positionnement

**Cible principale :** Agents publics, collectivités, ministères — en particulier :
- Profils manipulant des informations sensibles ou à diffusion restreinte (DR)
- Travaillant en français et/ou en anglais
- Utilisant de nombreux acronymes institutionnels (SGPE, DINUM, ZTNA, RPG…)
- Non-techniques — ne peuvent pas utiliser Docker ou un terminal
- Ayant besoin d'outputs portables et partageables (Markdown, Obsidian, Notion)

**Cible secondaire :** Professionnels privacy-first (juridique, médical, journalisme) et contributeurs open source souhaitant une stack IA locale entièrement auditable.

**Positionnement La Suite Numérique (DINUM) :**
Luciole est conçu pour pouvoir intégrer La Suite Numérique à terme, aux côtés de Tchap, Visio, Docs. La stratégie est : **standalone d'abord**, proposition DINUM ensuite. Le design suit le **Système de Design de l'État (DSFR)** dès le départ — typographie Marianne, palette gouvernementale, composants DSFR — pour qu'une intégration future ne nécessite pas de refonte.

---

## 3. Référence marché

| | Luciole | Snaply | VoiceInk | Meetily |
|---|---|---|---|---|
| Entièrement open source | ✅ Apache 2.0 | ❌ Fermé | ✅ GPLv3 | ✅ MIT |
| Sans terminal / Docker | ✅ | ✅ | ✅ | ❌ |
| Comptes-rendus réunion | ✅ | ✅ | ❌ | ✅ |
| Diarisation locuteurs | ✅ v1 | ❌ | ❌ | ⚠️ Partiel |
| Dictionnaire acronymes | ✅ Pipeline 2 passes | ❌ | ⚠️ Basique | ❌ |
| Output Markdown + wiki-links | ✅ | ❌ | ❌ | ❌ |
| Vault Obsidian | ✅ | ❌ | ❌ | ❌ |
| Stack auditée | ✅ | ❌ | ✅ | ✅ |
| Français première classe | ✅ | ⚠️ | ⚠️ | ⚠️ |
| Modèle LLM | Gemma 4 E4B | Fine-tune Gemma 4 | Whisper only | Ollama |
| RAG local sur notes passées | Roadmap | ❌ | ❌ | ❌ |

---

## 4. Use Cases MVP (v1)

### 4.1 Dictée live
- Maintenir le raccourci global `⌥` (Option) → parler → relâcher → texte injecté au curseur
- Fonctionne dans toute application (Slack, Word, navigateur, terminal)
- Feedback visuel en temps réel : pill flottante avec waveform près du curseur
- Passe de nettoyage grammatical via LLM local après transcription
- Support multilingue : français et anglais en priorité, 99 langues via Whisper

### 4.2 Actions sur sélection de texte
- Sélectionner du texte dans n'importe quelle app → déclencher le raccourci
- Actions disponibles : **Corriger**, **Traduire**, **Reformuler (formel/informel)**
- Résultat remplace la sélection ou apparaît dans un popup flottant (choix utilisateur)
- Actions extensibles via templates de prompts en JSON — les utilisateurs peuvent créer les leurs

### 4.3 Comptes-rendus de réunion
- Bascule enregistrement réunion (raccourci ou icône menu bar)
- Capture simultanée microphone + audio système
- Transcription par chunks (pas de streaming) — latence acceptable post-réunion
- **Diarisation des locuteurs** : identification de qui parle à quel moment
- Post-réunion : étape de nommage des locuteurs (interface audio snippets + champ nom)
- Reconnaissance automatique des participants récurrents (empreintes vocales locales)
- Gemma 4 génère les notes structurées (résumé, décisions, actions)
- Output : **fichier Markdown** avec liens wiki `[[...]]` pour personnes, projets, acronymes
- Sauvegarde optionnelle dans un **vault Obsidian** (configuration chemin de dossier)
- Historique indexé localement en SQLite

### 4.4 Dictionnaire d'acronymes
- Glossaire YAML éditable par l'utilisateur
- Glossaire gouvernemental par défaut fourni dans le dépôt (`glossaire-secteur-public.yaml`)
- Glossaires communautaires importables comme fichiers YAML simples
- Injection dans le system prompt Gemma 4 à chaque appel — pas de fine-tuning nécessaire
- Pipeline deux passes : correction regex sur transcript brut + contexte LLM

---

## 5. Architecture Technique

### 5.1 Framework : **Tauri v2** (Rust + React/TypeScript)

**Choix retenu et justification :**
- Bundle ~5–15 MB (vs. Electron 150+ MB)
- Backend Rust pour tous les appels système (audio, Accessibility API, hotkeys, injection texte)
- Frontend React/TypeScript — familier pour les contributeurs open source
- Chemin vers Windows plus tard sans réécriture complète
- Composants DSFR intégrables dans le frontend web

**Pourquoi pas Swift natif :** Barrière contributeur plus élevée, macOS seulement pour toujours, pas de réutilisation web.  
**Pourquoi pas un fork VoiceInk :** VoiceInk (Swift, GPLv3) n'a pas de comptes-rendus, pas d'intégration LLM, pas de pipeline acronymes. La plomberie dictée est reconstruite en Tauri/Rust ; la couche intelligence est identique à construire dans les deux cas. Apache 2.0 donne plus de liberté que GPLv3.

### 5.2 ASR (Speech-to-Text) : **whisper.cpp**

- Compilé comme binaire natif, **bundlé dans l'app** — aucune installation utilisateur
- Modèles téléchargés au premier lancement avec UI de progression
- **`whisper-small`** (~460 MB) : modèle par défaut, vitesse prioritaire pour la dictée
- **`whisper-large-v3-turbo`** : téléchargement optionnel proposé au premier enregistrement réunion (meilleure précision)
- Chunks streaming pour la dictée live ; mode batch pour les réunions
- Support français + multilingue natif

**Pourquoi pas Gemma 4 E4B audio pour la dictée :**  
L'audio input de Gemma 4 est batch-only — pas de streaming. La dictée exige un feedback quasi-temps réel. whisper.cpp streame en chunks de 1–2s. Pour les réunions (transcription post-hoc), Gemma 4 audio est une option future.

### 5.3 LLM : **Gemma 4 E4B via Ollama**

- Ollama installé silencieusement au premier lancement, avec étape de consentement explicite
- `gemma4:e4b` téléchargé automatiquement (~5.6 GB, une seule fois)
- Usages : nettoyage grammatical, correction, traduction, résumé réunion, génération wiki-links
- Tourne sur Apple Silicon GPU via Metal — rapide, faible consommation batterie
- Modèle swappable par l'utilisateur (Mistral, Qwen…) dans les réglages

**Référence marché :** Snaply utilise un Gemma 4 fine-tuné (~5.25 GB observé dans leurs réglages). Luciole utilise le modèle stock Apache 2.0 — auditable, reproductible, open. Le fine-tuning est un axe de contribution communautaire post-v1.

**Performances français :** Gemma 4 E4B validé pour la correction et le résumé en français sans adaptation supplémentaire.

### 5.4 Diarisation des locuteurs : **sherpa-onnx + pyannote community-1 + CAM++**

**Stack retenue (inspirée de l'implémentation OpenWhispr, avril 2026) :**
- **pyannote-segmentation-3.0** : détection et segmentation des locuteurs
- **CAM++ (3D-Speaker)** : embeddings d'empreintes vocales
- **sherpa-onnx** : runtime ONNX natif — pas de Python, pas de PyTorch, pas de GPU requis

**Poids bundlés directement dans l'app** (~50–80 MB) — aucun compte Hugging Face requis, cohérent avec la promesse "sans terminal, sans compte".

**Pipeline diarisation :**
```
Audio capturé (mic + système)
        ↓
whisper.cpp → transcript brut + timestamps
        ↓
sherpa-onnx / pyannote community-1
→ "Locuteur 0 : 00:00–00:45"
→ "Locuteur 1 : 00:46–01:20"
        ↓
Alignement timestamps ↔ segments transcript
        ↓
Interface de nommage post-réunion
→ snippets audio par locuteur + champ nom
→ empreintes sauvegardées dans ~/.luciole/speakers.db
→ reconnaissance automatique lors des réunions suivantes
        ↓
Passe 1 : Correction regex acronymes
        ↓
Gemma 4 : résumé avec attribution locuteurs
```

### 5.5 Intégration système (couche Rust)

```
Accessibility API    → lecture texte sélectionné dans toute app
CGEvent / hotkeys    → capture raccourcis globaux (sans focus app)
AVFoundation         → capture microphone + audio système simultanés
Clipboard API        → injection texte (sauvegarder → remplacer → restaurer)
NSStatusBar          → icône menu bar + contrôles
```

Nécessite l'entitlement `com.apple.security.accessibility` → **pas d'App Store** (attendu).  
Distribution via **DMG signé (Apple Developer ID)** sur GitHub Releases.

### 5.6 Pipeline acronymes (2 passes)

```
Passe 1 — Correction regex
  Transcript brut Whisper
  → scan des variantes phonétiques des acronymes connus
  → normalisation vers la forme canonique
    (ex. "S.G.P.E", "esgeupeu" → "SGPE")

Passe 2 — Injection contexte LLM
  System prompt inclut le glossaire complet :
  "SGPE = Secrétariat Général à la Planification Écologique
   DINUM = Direction Interministérielle du Numérique
   ZTNA = Zero Trust Network Access
   ..."
  → Gemma 4 utilise ce contexte pendant le résumé et le nettoyage
  → Acronymes correctement développés dans les comptes-rendus
```

**Format glossaire (éditable par l'utilisateur) :**
```yaml
# ~/.luciole/glossaire.yaml
acronyms:
  - short: SGPE
    full: Secrétariat Général à la Planification Écologique
  - short: DINUM
    full: Direction Interministérielle du Numérique
  - short: ZTNA
    full: Zero Trust Network Access
  - short: RPG
    full: Registre Parcellaire Graphique
  - short: HFDS
    full: Haut Fonctionnaire de Défense et de Sécurité
```

Le dépôt publie un `glossaire-secteur-public.yaml` maintenu par la communauté. Les utilisateurs peuvent importer des glossaires tiers ou créer les leurs.

### 5.7 Output comptes-rendus

```markdown
# Réunion — 2026-04-22

## Résumé
Discussion autour de la stratégie données eau et de la coordination
avec la [[DINUM]] sur le déploiement [[ZTNA]].

## Décisions
- Envoi d'une note à la [[HFDS]] avant fin de semaine
- Validation du référentiel [[RNASP]] reportée à mai

## Actions
- [ ] [[Eric]] — Note [[HFDS]] avant vendredi *(00:04)*
- [ ] [[Livio]] — Coordination [[DINUM]] sur [[ZTNA]] *(00:23)*

## Échanges
**[[Eric]]** *(00:00)* — On devrait contacter la [[DINUM]] avant
la fin du mois sur le point [[ZTNA]].

**[[Livio]]** *(00:43)* — D'accord, je prends en charge.
Le dossier [[RNASP]] peut attendre mai.

## Participants
[[Eric]], [[Livio]]

## Tags
#[[SGPE]] #[[DINUM]] #[[planification-écologique]]
```

- Stocké localement : `~/.luciole/notes/YYYY-MM-DD-titre.md`
- Compatible Obsidian, Logseq, Notion import, tout éditeur Markdown
- **Vault Obsidian** : chemin de dossier configurable dans les réglages — les notes sont écrites directement dans le vault, `[[wiki-links]]` immédiatement résolubles
- Transcript brut sauvegardé en parallèle : `YYYY-MM-DD-titre_transcript.txt`

### 5.8 Stockage local

```
~/.luciole/
  notes/              # Comptes-rendus Markdown
  transcripts/        # Transcripts bruts
  glossaire.yaml      # Dictionnaire acronymes utilisateur
  settings.json       # Configuration app
  db.sqlite           # Index réunions, recherche
  speakers.db         # Empreintes vocales locales
  models/             # Poids bundlés (whisper, diarisation)
```

SQLite pour l'historique et la recherche. Pas de sync cloud. L'utilisateur possède toutes ses données en fichiers plats.

### 5.9 Roadmap post-v1 : RAG local sur notes passées

Embedding de toutes les notes passées via un modèle local (`nomic-embed-text` via Ollama). Vecteurs stockés dans SQLite avec sqlite-vec. Au démarrage d'une réunion, Luciole remonte les notes passées sémantiquement proches comme contexte pour Gemma 4 — comportement "wiki LLM" où les nouvelles notes se connectent automatiquement à l'historique pertinent.

---

## 6. Expérience premier lancement (la promesse "sans terminal")

```
Bienvenue dans Luciole
Votre compagnon IA local et privé

Configuration de votre assistant...

[1/4] Modèle de transcription vocale — Whisper small (460 Mo)
      ████████████████████░░░░  80%

[2/4] Moteur IA local — Ollama
      Installation en cours sur votre Mac.
      Vos données ne quitteront jamais votre appareil.  ✓

[3/4] Modèle de langage — Gemma 4 (5,6 Go)
      ████░░░░░░░░░░░░░░░░░░░░  18%  ~6 min restantes

[4/4] Reconnaissance des locuteurs (75 Mo)             ✓

────────────────────────────────────────────────────
Permissions requises (une seule fois) :

  🎤  Microphone              [Autoriser]
  ♿  Accessibilité            [Ouvrir Réglages →]

────────────────────────────────────────────────────
Vous êtes prêt.

  Maintenez ⌥ pour dicter
  Sélectionnez du texte + ⌥P pour corriger
  ⌥M pour démarrer / arrêter un enregistrement
```

Aucun terminal. Aucun Docker. Aucun compte. Aucun Python.

---

## 7. Modèle d'interaction

| Action | Déclencheur | Résultat |
|---|---|---|
| Dicter | Maintenir `⌥` | Texte injecté au curseur au relâchement |
| Corriger la sélection | Sélection + `⌥P` | Texte corrigé remplace la sélection |
| Traduire la sélection | Sélection + `⌥T` | Texte traduit remplace la sélection |
| Reformuler | Sélection + `⌥R` | Réécriture dans le ton choisi |
| Démarrer/stopper réunion | `⌥M` ou menu bar | Bascule enregistrement + indicateur waveform |
| Nommer les locuteurs | Auto après réunion | Panel snippets audio + champs nom |
| Historique notes | Menu bar → Notes | Panel notes passées en Markdown |
| Éditer glossaire | Menu bar → Glossaire | Éditeur clé-valeur simple |

Tous les raccourcis sont configurables dans les Réglages.

---

## 8. Design & Style

- **Langue UI :** Français par défaut, anglais disponible
- **Design system :** DSFR (Système de Design de l'État) — typographie Marianne, palette gouvernementale, composants DSFR — pour compatibilité La Suite Numérique
- **Ton :** Sobre, institutionnel, rassurant — pas de jargon tech, pas d'anthropomorphisme excessif
- **Icône menu bar :** Petite luciole stylisée, pulse doucement lors de l'écoute ou du traitement

---

## 9. Distribution

- **GitHub Releases** — `.dmg` signé (Apple Developer ID)
- **Homebrew Cask** — `brew install --cask luciole` (post-v1)
- **Pas d'App Store** — entitlement Accessibility incompatible avec le sandboxing

**Licence :** Apache 2.0  
**Poids modèles fine-tunés** (quand publiés) : Apache 2.0, hébergés sur Hugging Face

---

## 10. Périmètre MVP v1

### Inclus ✅
- Raccourcis globaux + injection texte dans toute app
- Dictée live avec nettoyage grammatical LLM
- Actions sur sélection (corriger, traduire, reformuler)
- Enregistrement réunion + notes Markdown structurées
- **Diarisation des locuteurs** (sherpa-onnx, bundlé)
- Interface de nommage des locuteurs post-réunion
- Reconnaissance automatique participants récurrents
- Dictionnaire acronymes 2 passes (YAML, glossaire gov par défaut)
- Premier lancement sans terminal (wizard progressif)
- App menu bar avec panneau historique
- Index SQLite des notes
- Export Markdown + configuration vault Obsidian
- Design DSFR

### Hors périmètre v1 ❌
- Support Windows
- App iOS companion
- Sync cloud ou partage équipe
- Fine-tuning du modèle (stock Gemma 4 E4B en v1)
- RAG local / recherche sémantique sur notes passées
- Intégration calendrier (rejoindre réunions automatiquement)
- Export PDF
- Diarisation en temps réel (post-réunion uniquement en v1)

---

## 11. Questions ouvertes résolues

| Question | Décision |
|---|---|
| Nom | **Luciole** |
| Framework | **Tauri v2** (Rust + React) |
| Fork VoiceInk ? | **Non** — build from scratch |
| ASR | **whisper.cpp** bundlé (small par défaut, large-v3-turbo optionnel) |
| LLM | **Gemma 4 E4B via Ollama** |
| Diarisation | **sherpa-onnx + pyannote community-1 + CAM++, poids bundlés** |
| Compte HF requis ? | **Non** — poids diarisation bundlés directement |
| Étape Ollama visible ? | **Oui** — étape explicite, cadrage privacy |
| Glossaire | **YAML simple, glossaire gov par défaut dans le dépôt** |
| Output notes | **Markdown + vault Obsidian (chemin configurable)** |
| Français | **Gemma 4 E4B suffisant, validé** |
| Stratégie DINUM | **Standalone d'abord, design DSFR dès v1** |
| Licence | **Apache 2.0** |
| Distribution | **DMG signé GitHub Releases, Homebrew post-v1** |
| Conflit nom fichier Obsidian | **Suffixe timestamp automatique** |

---

## 12. Prochaines étapes

1. **Scaffold projet Tauri v2** — structure Rust + React, entitlements macOS
2. **Prototype hotkey + injection texte** — valider la boucle système fondamentale
3. **Pipeline whisper.cpp** — binaire bundlé, téléchargement modèle au premier lancement
4. **Intégration Ollama + Gemma 4** — actions texte de base (corriger, traduire)
5. **Pipeline diarisation** — sherpa-onnx intégré, UI nommage locuteurs
6. **Pipeline réunion complet** — audio → transcript → diarisation → notes Markdown
7. **Wizard premier lancement** — UX sans terminal, permissions, progression
8. **Glossaire acronymes** — pipeline 2 passes + éditeur UI
9. **Intégration Obsidian** — chemin vault configurable, wiki-links

---

*Draft v0.2 — Avril 2026*  
*Prochaine étape : scaffold Tauri + prototype hotkey/injection*
