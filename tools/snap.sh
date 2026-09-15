#!/usr/bin/env bash
# snap — headless native render: Xvfb + lavapipe (software Vulkan) + llvmpipe,
# screenshot at T seconds, auto-exit. This is the G-B1 software-render path:
# no GPU required. Usage: tools/snap.sh out.png [seconds]
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${1:?out.png}"; AT="${2:-6}"
mkdir -p "$HERE/shots"

# runtime libs from the nix store (this machine has no system cc/libs);
# only 64-bit copies (ELF class byte == 0x02)
LIBS=""
find64() {
  find /nix/store -maxdepth 3 -name "$1" 2>/dev/null | while read -r f; do
    [ "$(head -c 5 "$f" | tail -c 1 | od -An -tx1 | tr -d ' ')" = "02" ] && { echo "$f"; return; }
  done | head -1
}
for soname in libwayland-client.so.0 libudev.so.1 libxkbcommon-x11.so libxkbcommon.so.0 libxcb-xkb.so.1 libX11.so.6 libX11-xcb.so.1 libxcb.so.1; do
  p="$(find64 "$soname")"
  [ -n "$p" ] && LIBS="$(dirname "$p"):$LIBS"
done
MESA="$(dirname "$(find64 libvulkan_lvp.so)")"
VKL="$(dirname "$(find64 libvulkan.so.1)")"

Xvfb :140 -screen 0 1280x760x24 &>/dev/null &
XVFB=$!
trap 'kill $XVFB 2>/dev/null || true' EXIT
sleep 1

cd "$HERE"
DISPLAY=:140 \
  VK_ICD_FILENAMES=/run/opengl-driver/share/vulkan/icd.d/lvp_icd.x86_64.json \
  LD_LIBRARY_PATH="$VKL:$MESA:$LIBS" \
  "$HERE/target/release/pelican-game" --snap "$HERE/shots/$OUT"
ls -la "$HERE/shots/$OUT"
