#!/usr/bin/env bash
# deploy — publish the wasm build to Cloudflare Pages as pelican.catcave.dev.
# Requires CLOUDFLARE_API_TOKEN (Pages edit + zone edit for catcave.dev).
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
: "${CLOUDFLARE_API_TOKEN:?export CLOUDFLARE_API_TOKEN (agent-ops sops bundle)}"
PROJECT="pelican-game"
DOMAIN="pelican.catcave.dev"
cd "$HERE"

if ! npx -y wrangler@latest pages project list 2>/dev/null | grep -q "$PROJECT"; then
  echo "== creating Pages project $PROJECT =="
  npx -y wrangler@latest pages project create "$PROJECT" --production-branch main
fi

echo "== deploying web/ =="
npx -y wrangler@latest pages deploy web --project-name "$PROJECT" --branch main

echo "== custom domain $DOMAIN =="
ACCOUNT_ID="$(curl -s "https://api.cloudflare.com/client/v4/accounts" \
  -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"][0]["id"])')"
ZONE_ID="$(curl -s "https://api.cloudflare.com/client/v4/zones?name=catcave.dev" \
  -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"][0]["id"])')"
curl -s -X POST "https://api.cloudflare.com/client/v4/accounts/$ACCOUNT_ID/pages/projects/$PROJECT/domains" \
  -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" -H "Content-Type: application/json" \
  -d "{\"name\":\"$DOMAIN\"}" | python3 -c 'import json,sys; d=json.load(sys.stdin); print("domain add:", "ok" if d.get("success") else d)'

echo "== verify =="
sleep 6
curl -s -o /dev/null -w "https://$DOMAIN -> %{http_code}\n" -m 30 "https://$DOMAIN"
curl -s -o /dev/null -w "pages.dev -> %{http_code}\n" -m 30 "https://$PROJECT.pages.dev"
