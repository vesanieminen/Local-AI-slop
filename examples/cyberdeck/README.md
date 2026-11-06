# CALC//DECK — a cyberpunk field calculator in your tab

A deck from 2077, running on the same [**Rune**](https://github.com/anomalyco/rune)
pipeline as Futhark: native Rust → WASM → one WebGPU canvas. No DOM widgets, and
no fonts in the readout — every digit is drawn as seven bevelled 7-segment
polygons, with a haze pass, a body pass and a white-hot filament core, plus
ghost segments behind them like a cold panel.

Behind the glass: a synthwave city — a banded magenta sun sunk behind the
skyline, a perspective floor grid, windows that blink on a seeded hash, drifting
neon signs, and rain. On the glass: CRT scanlines, a vignette, and a boot
sequence that scrambles the readout before it settles on `0`.

It behaves like a deck, not a widget:

- **Sparks** burst from every key (window-coordinate overlay, gravity + drag), and keys dim/bump on press.
- The **boot log** types itself down the screen while the keypad stays inert; the readout scrambles, then `READY`.
- **Hex mode**: `R10 ⇄ R16` (chip, or `h` key) slides an `A–F` row above the keypad; the mirror line shows the live `0x` / `0b` reading.
- **ICE alarm**: divide by zero and the deck flashes red, `ICE//` burns across the readout, and the tape logs the intrusion. Overflow gets the same treatment as `OUT//`.
- **TX LOG** tape (wide layouts) keeps the calculation history; phones get the last line in the footer.
- **Four channels** — tap the brand: Ghostwire / Amber / Blood / Moss, each a full palette swap with a glitch wipe.
- **1337**: type it and the deck grants `GHOST LINK` — the brand gains a `G` and the sun lingers a shade longer.
- **Synth blips** on every press (WebAudio oscillators, wasm only) — boot chime included.

## Building

The Rust source is in `src/`. The site serves the prebuilt `dist/` so Pages CI stays
fast; rebuild locally:

```bash
# one-time toolchain
rustup target add wasm32-unknown-unknown
# wasm-bindgen on PATH must match the version in Cargo.lock — release binaries:
#   https://github.com/rustwasm/wasm-bindgen/releases
# optional, for --opt: wasm-opt from https://github.com/WebAssembly/binaryen/releases

./build.sh --opt          # clones Rune at the pinned rev into ./rune, builds dist/
python3 -m http.server 8123 -d dist
# open http://localhost:8123/  (WebGPU needs localhost or HTTPS)
```

## Verifying it

Two stages, because headless Chromium here can't screenshot WebGPU canvases
(every capture path composites them as black — same blind spot as Futhark):

```bash
# 1) the live web build: zero console errors, real GPU submissions, canvas
#    sizing and a tap-survive test, desktop + phone:
node verify.mjs --url http://127.0.0.1:8123/

# 2) pixels, from Rune's native headless agent tooling (software Vulkan):
./snap.sh /tmp/cyber-snaps                  # --snap PNGs + element tree, --at on a virtual clock
node verify.mjs --png /tmp/cyber-snaps/hero-1280x800.png /tmp/cyber-snaps/calc-390x844.png ...
```

The hub thumbnail is one of those native snaps (same view code, same layout engine).

Screenshot / test hooks: `?hero` (finished calc + the 1337 easter egg) `?demo`
(1024 × 16 = 16384 mid-tape) `?ice` (divide-by-zero, frozen mid-alarm) `?beat`
`?seed=NNN`; natively `CYBER_HERO=1 CYBER_DEMO=1 CYBER_ICE=1 CYBER_BEAT=1
CYBER_SEED=NNN`.

Pinned Rune rev: `781e2b6d87cd5f32d6f4365956fad4baaf24ee50` (dev, 2026-11-06).
