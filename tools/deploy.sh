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
# The Access app (lockdown-pelican.catcave.dev) fronts the custom domain:
# anonymous requests 302 to the Access login. That redirect IS the "live and
# locked" signal. (200 would mean Access came off; anything else is a problem.)
loc=$(curl -s -o /dev/null -w "%{http_code} %{redirect_url}" -m 30 "https://pelican.catcave.dev")
echo "https://pelican.catcave.dev -> $loc"
case "$loc" in
  *"cloudflareaccess.com"*) echo "  live + Access-locked (intended)" ;;
  "200 "*) echo "  WARNING: live but NOT behind Access" ;;
  *) echo "  UNEXPECTED response" ;;
esac
# deployed bytes are content-hashed by wrangler; compare against web/ locally:
sha256sum web/pelican_game_bg.wasm web/pelican_game.js web/index.html | sed 's/^/  /'
