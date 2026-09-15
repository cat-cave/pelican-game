#!/usr/bin/env bash
# with-env — run a command inside the native Bevy build environment
# (Rust toolchain + nix system deps). Used by build-native.sh / run-native.sh.
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
source "$HERE/tools/env.sh"
# wasm builds need the host C toolchain only for build scripts; native also
# needs alsa/x11/wayland/udev system libs.
exec nix-shell -p pkg-config alsa-lib wayland libxkbcommon \
  xorg.libX11 xorg.libXcursor xorg.libXrandr xorg.libXi \
  udev.dev --run "$*"
