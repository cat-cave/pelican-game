#!/usr/bin/env bash
# make-sprites — author the rider/wheel sprites:
#   canvas draw (pixd 12-color palette, same code as the design reference)
#   -> headless chromium capture on magenta -> chroma-key + trim (python).
# Output: assets/sprites/{rider,wheel}.png + sprites.json (origin offsets).
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
AUTH=/tmp/spriteauth
mkdir -p "$AUTH" "$HERE/assets/sprites"
cp "$HERE/tools/sprite-author.html" "$AUTH/author.html"
CHROMIUM="$(command -v chromium || command -v chromium-browser)"
"$CHROMIUM" --headless --disable-gpu --no-sandbox --screenshot="$AUTH/wheel.png" \
  --window-size=128,128 --hide-scrollbars "file://$AUTH/author.html?part=wheel" 2>/dev/null
"$CHROMIUM" --headless --disable-gpu --no-sandbox --screenshot="$AUTH/rider.png" \
  --window-size=512,384 --hide-scrollbars "file://$AUTH/author.html?part=rider" 2>/dev/null
"$CHROMIUM" --headless --disable-gpu --no-sandbox --screenshot="$AUTH/cloud.png" \
  --window-size=320,128 --hide-scrollbars "file://$AUTH/author.html?part=cloud" 2>/dev/null
python3 "$HERE/tools/make-sprites.py"
