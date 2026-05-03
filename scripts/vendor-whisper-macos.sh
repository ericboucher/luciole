#!/usr/bin/env bash
# Build whisper.cpp whisper-cli and install into src-tauri/binaries/whisper-<rustc triple>.
# Requires: git, a working CMake (brew install cmake, or CMake.app), Xcode CLT (clang).
set -euo pipefail

# Évite git/curl via proxy mort (ex. Clash 127.0.0.1:7890 éteint).
unset http_proxy https_proxy HTTP_PROXY HTTPS_PROXY all_proxy ALL_PROXY no_proxy NO_PROXY

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TRIPLE="$(rustc --print host-tuple)"
DEST="$ROOT/src-tauri/binaries/whisper-${TRIPLE}"
WORKDIR="$(mktemp -d)"
cleanup() { rm -rf "$WORKDIR"; }
trap cleanup EXIT

# Le premier "cmake" du PATH est souvent le wrapper Homebrew Python (`pip install cmake`) cassé.
# On essaie d'abord le binaire du formule `cmake`, puis CMake.app, en dernier le `cmake` du PATH.
find_working_cmake() {
  local c
  for c in \
    "${CMAKE-}" \
    "/Applications/CMake.app/Contents/bin/cmake" \
    "/opt/homebrew/opt/cmake/bin/cmake" \
    "/usr/local/opt/cmake/bin/cmake" \
    "$(command -v cmake 2>/dev/null || true)"; do
    [[ -z "$c" ]] && continue
    if "$c" --version >/dev/null 2>&1; then
      printf '%s' "$c"
      return 0
    fi
  done
  return 1
}

if ! CMAKE_BIN="$(find_working_cmake)"; then
  echo "Aucun CMake exécutable trouvé (le /opt/homebrew/bin/cmake actuel est souvent un wrapper Python"
  echo "qui appelle un CMake.app manquant dans site-packages)."
  echo "→ Installe le binaire officiel : brew install cmake"
  echo "  ou installe https://cmake.org/download/ (CMake.app) et relance."
  echo "  ou : export CMAKE=/chemin/vers/cmake"
  exit 1
fi
echo "Utilise CMake: $CMAKE_BIN"

mkdir -p "$(dirname "$DEST")"
git -c http.proxy= -c https.proxy= \
  clone --depth 1 --branch v1.8.4 \
  https://github.com/ggml-org/whisper.cpp.git "$WORKDIR/w"
# BUILD_SHARED_LIBS=OFF : lien statique dans whisper-cli ; sinon libwhisper*.dylib avec @rpath vers le build /tmp (cassé après copie).
"$CMAKE_BIN" -S "$WORKDIR/w" -B "$WORKDIR/build" \
  -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_SHARED_LIBS=OFF \
  -DWHISPER_BUILD_EXAMPLES=ON
"$CMAKE_BIN" --build "$WORKDIR/build" --target whisper-cli -j"$(sysctl -n hw.ncpu 2>/dev/null || echo 4)"

cp "$WORKDIR/build/bin/whisper-cli" "$DEST"
chmod +x "$DEST"
echo "OK: $DEST"
echo "Sidecar : bundle.externalBin est déjà défini dans tauri.conf.json → le .app embarque bin/MacOS/whisper."
echo "Nom attendu par Tauri pour ce triple : whisper-${TRIPLE} (le script l’a déjà utilisé, pas de renommage)."
