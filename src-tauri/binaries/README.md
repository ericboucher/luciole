# Binaire Whisper (whisper.cpp)

Place ici le CLI **`whisper-cli`** compilé pour ta machine, sous le nom attendu par le dépôt :

`whisper-<triple rustc>`  
Exemple Apple Silicon : `whisper-aarch64-apple-darwin`

Génération :

```bash
chmod +x scripts/vendor-whisper-macos.sh
./scripts/vendor-whisper-macos.sh
```

Le dépôt **déclare déjà** `"externalBin": ["binaries/whisper"]` dans `tauri.conf.json`.  
Le fichier produit doit s’appeler **`whisper-<triple rustc>`** (ex. `whisper-aarch64-apple-darwin`) — c’est ce que fait le script ; **pas de renommage**.

Si `git clone` échoue sur `127.0.0.1:7890` : un proxy (Clash, etc.) pointe vers un port fermé. Le script désactive les variables `*_proxy` et force un clone sans proxy Git. Sinon : `git config --global --unset http.proxy` (et `https.proxy`) ou allume le proxy.

### CMake : `FileNotFoundError` … `CMake.app/Contents/bin/cmake`

Souvent `/opt/homebrew/bin/cmake` est un **wrapper Python** (`pip install cmake`), pas la formule Homebrew. Il appelle un CMake embarqué absent.

- Installe le vrai CMake : **`brew install cmake`** (binaire dans `/opt/homebrew/opt/cmake/bin/cmake`).
- Ou installe **CMake.app** depuis cmake.org et ajoute le PATH, ou :  
  `export CMAKE=/Applications/CMake.app/Contents/bin/cmake` avant le script.

Variable optionnelle de debug : `LUCIOLE_WHISPER_CPP` = chemin absolu vers un autre exécutable.

### `Library not loaded: @rpath/libwhisper.1.dylib`

Tu avais compilé avec des **.dylib** partagées ; les chemins `@rpath` pointaient encore vers le dossier de build temporaire. Le script vendor utilise maintenant **`-DBUILD_SHARED_LIBS=OFF`** (lien statique). Refais :

```bash
./scripts/vendor-whisper-macos.sh
```

Puis relance `pnpm tauri dev`.
