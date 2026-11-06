# Futhark — a native Rust app in your tab

The hub's first native app: [**Rune**](https://github.com/anomalyco/rune), a GPU-rendered
Rust UI framework, compiled to WASM and drawn into one WebGPU canvas. No JS UI libraries,
no DOM widgets — Rune's own renderer, text, layout and springs.

It's dressed as a dwarven forge hall: every panel is a chiselled stone slab (stone
gradient, bevelled edges, inset lips), letters are engraved three-deep (ember rim,
dark recess, pale face), the 24 Elder Futhark runes are cut as dark grooves with a
**molten core and a spark at the chisel tip**, magma rivers flow across the floor, and
embers drift up the hall on a torch-flicker clock. The Cast tab draws three runes —
past, present, future — on glowing slabs. And **THE LEDGER** is the todo list the
hall demands: carve a task and leave it untouched, and its heat bar fills as the
slab slides toward the magma line — then it cracks, glows and crumbles into the
floor. Strike tasks down to seal them in gold first.

Every panel's face is **procedural basalt**: at startup the app grows a seamless
fractal height field, derives surface normals from it, Lambert-shades them from
a fixed top-left light and rasterizes the result once into a `Pixels` image — a
bump/displacement map baked to RGB, shared by every slab (`Pixels` +
`SurfaceFit::Fill`). The floor adds real relief: each flagstone is drawn with a
dark sunk copy under its face, a lit top edge and chips, courses overlapping
into depth. And the hall's lower half is a **flowing magma landscape**: river
polylines drawn as six nested strokes — far heat bleed down to a thin white-hot
seam — so every channel is a pure gradient of heat like the wall cracks, with
hot dashes walking downstream by arc length (`sub_polyline`), quadratic-Bezier
flame tongues licking from the banks and vents, a flowing magma lake at the
bottom edge and a 58-ember field on a torch-flicker clock. One gotcha worth
knowing: an element that sizes one axis from content takes the *image's* aspect
ratio once it carries pixels — fixed sizes or `aspect_ratio(1e6)` keep content
in charge.

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
(every capture path composites them as black — Rune's own `web-hello` has the
same blind spot, and its proof suite passes anyway):

```bash
# 1) the live web build: zero console errors, real GPU submissions while a
#    cast animates, and a slow ambient beat at rest (the forge never fully
#    stills — rivers flow, embers climb), desktop + mobile:
node verify.mjs --url http://127.0.0.1:8123/

# 2) pixels, from Rune's native headless agent tooling (software Vulkan):
./snap.sh /tmp/snaps                     # --snap PNGs + element tree, --at on a virtual clock
node verify.mjs --png /tmp/snaps/cast-1280x800.png /tmp/snaps/alphabet-390x844.png ...
```

The hub thumbnail is one of those native snaps (same view code, same layout engine).

Screenshot / test hooks: `?mode=cast` `?seed=NNN` `?auto` (staggered cast animation)
`?mode=tasks` (pre-heated ledger) `?rune=NN` `?beat`; natively `FUTHARK_CAST=1
FUTHARK_SEED=7 FUTHARK_RUNE=NN FUTHARK_TASKS=1`.

Pinned Rune rev: `781e2b6d87cd5f32d6f4365956fad4baaf24ee50` (dev, 2026-11-06).
