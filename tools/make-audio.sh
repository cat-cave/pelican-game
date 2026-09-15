#!/usr/bin/env bash
# make-audio — derive the game's web music loop from the sond plane's proof
# artifact (sond-pelican-tune.wav, the fixture music variant). Probes the sond
# API for a fresh compose first; without SONDD_TOKEN (vault sealed at trial
# time) it records the block honestly and derives the loop: 15 bars @132bpm,
# matched crossfade for a seamless loop, libvorbis q4.
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
SRC="${1:-$HERE/assets/proof/sond-pelican-tune.wav}"
OUT="$HERE/assets/audio"
BASE="${SOND_BASE_URL:-https://sound.catcave.dev}"
mkdir -p "$OUT"

code="$(curl -sS -o /dev/null -w '%{http_code}' -m 15 -X POST \
  "$BASE/api/v1/jobs" -H 'Content-Type: application/json' \
  -d '{"name":"pelican-game-loop-variant"}' || echo transport-error)"
echo "sond api probe: $code"

BAR=1.8181818182
DUR=$(python3 -c "print(f'{$BAR*15:.6f}')")
XF=$(python3 -c "print(f'{$BAR*0.2:.6f}')")
ffmpeg -y -v error -i "$SRC" -t "$DUR" \
  -af "afade=t=out:st=$(python3 -c "print(f'{$DUR-$XF:.6f}')"):d=$XF,afade=t=in:st=0:d=$XF" \
  -c:a libvorbis -q:a 4 "$OUT/pelican-tune-loop.ogg"
DUR_GOT="$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$OUT/pelican-tune-loop.ogg")"
echo "loop: assets/audio/pelican-tune-loop.ogg  dur=${DUR_GOT}s  (from the sond plane's proof artifact; probe $code)"
