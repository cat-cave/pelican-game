#!/usr/bin/env bash
# env — toolchain environment for the pelican-game Bevy build.
#
# Toolchain (taken over from card G-B1 on this workstation):
#   Rust 1.96.1 stable (pinned by toolchain) + wasm32-unknown-unknown std,
#   nix-provided C toolchain + Bevy's native Linux system deps (alsa, wayland,
#   x11, libxkbcommon, udev) via nix-shell. The wasm32 build needs none of
#   the system libs — this env is for native iteration.
#
# Usage: source tools/env.sh   (then cargo build / cargo run)
#        tools/with-env.sh CMD runs CMD inside the full native env.
export RUSTUP_HOME="$HOME/.rustup"
export CARGO_HOME="$HOME/.cargo"
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/projects/pelican-game/target}"
