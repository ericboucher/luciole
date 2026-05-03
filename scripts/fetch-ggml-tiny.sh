#!/usr/bin/env bash
# Télécharge ggml-tiny.bin (ressource packagée avec l'app).
set -euo pipefail
unset http_proxy https_proxy HTTP_PROXY HTTPS_PROXY all_proxy ALL_PROXY
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/src-tauri/resources/models/ggml-tiny.bin"
mkdir -p "$(dirname "$OUT")"
curl --noproxy '*' -fL --retry 3 -o "$OUT" \
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
ls -la "$OUT"
