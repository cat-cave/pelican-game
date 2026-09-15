# Pelicans Riding Bicycles: A Feasibility Study — The Field Trials

A pelican, riding a bicycle, over five trials-style side-scrolling field
trials. Pedal, lean, flip on the canyon drops, keep the beak up. This is the
catcave composite trial: a **Bevy game compiled to wasm32** and played in the
browser, with its content flowing from the estate's planes.

**Play:** <https://pelican.catcave.dev> (Cloudflare Pages, static).

## The game

- Engine: **Bevy 0.19**, native + **wasm32-unknown-unknown** (the browser
  build is part of the proof). Fixed-timestep deterministic physics; no
  randomness in the world.
- Verification: `cargo run --example bot` (all five levels + flip
  feasibility), `--simtest` (the real app loop, render-free: state machine,
  scoring, finish — see below), and `tools/snap.sh` (headless native render:
  Xvfb + lavapipe, the software-Vulkan path).
- Five levels: The Seaside Promenade, Downhill With Wind, The Jetty Gaps,
  Klezmer Canyon, The Feasibility Summit. Score = finish + flip bonus + time
  bonus vs par. Crash on beak-first ground contact, over-rotation, or water.
- `cargo run --example bot` — the scripted playtest: a cautious bot must
  finish all five levels, and a committed-backflip bot must bank flips on
  Klezmer Canyon and still finish. Both pass.

## Content pipeline (the planes)

| Content | Source |
|---|---|
| Pelican sprite, ground weave tile | pixd plane proof artifacts (authored in-plane by jailed Aseprite Lua) |
| Finish-flag checker + strata tiles | pixd's own `driver.py` tile code path (API bearer-gated: vault sealed at build time; the probe is in `tools/pixd-extras.py`) |
| Music (132 BPM klezmer loop) | sond plane's proof artifact `sond-pelican-tune.wav`, derived to a seamless loop (`tools/make-audio.sh`; sond API bearer-gated likewise) |
| Title screen language | the design plane's deck (paper/ink/accent, serif + mono kickers, Report No. 042) |
| Strings | embedded locale bundles; ja via live model calls through the kumiki plane (`tools/localize-machine.py`, `locale-ja` feature) |
| Rider/wheel sprites | palette-faithful canvas art in the pixd 12-color palette (`tools/make-sprites.sh`) |

## Build

```sh
tools/with-env.sh "cargo run"                    # native (nix env: rust + bevy system libs)
tools/with-env.sh "bash tools/build-wasm.sh"     # web build -> web/ (serve statically)
tools/with-env.sh "cargo run --example bot"      # scripted playtest
```

Toolchain: Rust 1.96.1 stable + wasm32-unknown-unknown (`tools/env.sh`); the
native env supplies cc/alsa/x11/wayland via nix. No system Rust needed.

## Layout

- `src/` — the game (physics, levels, world render, deck-language UI)
- `assets/` — sprites, tiles, fonts, audio, locales (game content)
- `web/` — the wasm shell (index.html + loader + the sond loop)
- `tools/` — build + content-pipeline scripts
- `prototype/` — the original web-prototype (JS) used to tune the physics
  and levels; kept as the design reference
