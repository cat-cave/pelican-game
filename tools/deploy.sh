#!/usr/bin/env bash
# deploy — publish the wasm build as the pelican-game Worker (static assets,
# wrangler.jsonc) at pelican.catcave.dev. Requires CLOUDFLARE_API_TOKEN with
# Workers scripts edit for the account (the sops-bundle token works; the Pages
# path was retired — the Pages API 403s under it and the live serve path has
# been the Worker since 2026-09-15).
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
: "${CLOUDFLARE_API_TOKEN:?export CLOUDFLARE_API_TOKEN}"
: "${CLOUDFLARE_ACCOUNT_ID:?export CLOUDFLARE_ACCOUNT_ID}"
cd "$HERE"

echo "== deploying web/ via Worker static assets =="
npx -y wrangler@latest deploy

echo "== verify =="
sleep 6
curl -s -o /dev/null -w "https://pelican.catcave.dev -> %{http_code}\n" -m 30 "https://pelican.catcave.dev"
curl -s -o /dev/null -w "wasm -> %{http_code} (%{size_download} bytes)\n" -m 60 "https://pelican.catcave.dev/pelican_game_bg.wasm"
curl -s -m 30 "https://pelican.catcave.dev/assets/locales/ja.json" | python3 -c 'import json,sys; print("ja.json live:", json.load(sys.stdin)["game.title"])'
