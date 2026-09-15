#!/usr/bin/env bash
# build-wasm — the browser-playable build (wasm32-unknown-unknown).
# The wasm build is part of the trial's proof: Bevy compiled to the web.
# Output: web/ (index.html + pelican_game.js + wasm + assets) — deploy as-is.
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
source "$HERE/tools/env.sh"

cd "$HERE"
echo "== cargo build (wasm32-unknown-unknown, release) =="
# locale-ja embeds the machine-lane ja bundle (assets/locales/ja.json via
# tools/localize-machine.py) — the game ships bilingual; without the feature
# the toggle compiles out (HAS_JA=false) and the bundle is dead bytes.
cargo build --release --target wasm32-unknown-unknown --bin pelican-game --features locale-ja

echo "== wasm-bindgen =="
rm -rf web/pkg
mkdir -p web/pkg
wasm-bindgen --target web --out-dir web/pkg --out-name pelican_game \
  "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/pelican-game.wasm"
mv web/pkg/pelican_game.js web/pelican_game.js
mv web/pkg/pelican_game_bg.wasm web/pelican_game_bg.wasm
rm -rf web/pkg

echo "== wasm-opt (binaryen) =="
nix-shell -p binaryen --run \
  "wasm-opt -Oz --strip-debug -o web/pelican_game_bg.opt.wasm web/pelican_game_bg.wasm \
   && mv web/pelican_game_bg.opt.wasm web/pelican_game_bg.wasm"

echo "== stage assets =="
rm -rf web/assets
mkdir -p web/assets
cp -r assets/* web/assets/

echo "== done =="
ls -la web/ | head -8
du -sh web/pelican_game_bg.wasm
