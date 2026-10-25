# Local AI slop 🧪

Browser experiments, served as GitHub Pages from one repo:

**▶ Hub: https://vesanieminen.github.io/Local-AI-slop/**

| Example | What it is |
|---|---|
| [Living room — WebGPU](examples/livingroom/) | Blender-exported living room at 160 fps with WebGPURenderer: AgX matched to the Cycles look, same HDRI, GTAO + bloom. Needs a WebGPU browser; ~50 MB. |
| [RollerCoaster — Scream Machine Edition](examples/coaster/) | A coaster that designs itself — lift, drop, airtime hills, helix — gets validated by a physics sim (stalls and 4-G layouts get re-rolled), then you ride the survivor from the front seat: 240 Hz point-mass physics, seat-measured G-forces, hand-projected rails, synthesized clanks, wind and screams. One file, no assets. |
| [Orrery — the whole solar system, per pixel](examples/orrery/) | Real Keplerian orbits (J2000 elements) raytraced in one fullscreen shader: sun corona, Jupiter's Great Red Spot, Earth's clouds and night lights, Saturn's ring shadows. Compressed orbits to play, true scale to feel the void, time warp 1 hour → 20 years/s, tap a world to fly there. One file, no assets. |
| [Black hole](examples/blackhole/) | Every pixel raymarches an actual photon orbit around a Schwarzschild black hole: gravitational lensing, Doppler-beamed accretion disk, photon ring. One file, no libraries, no assets. |
| [Ink — a fluid you can stir](examples/ink/) | A real-time incompressible fluid in WebGL2: advect the velocity field, solve pressure with Jacobi iterations, project it divergence-free, and carry glowing dye along. Drag to pour ink into the vortex; four palettes, stir, clear and pause. One file, no assets. |
| [Coral — reaction-diffusion](examples/coral/) | Gray-Scott reaction-diffusion on ping-ponged half-float textures: coral, mitosis, worms, fingerprints and mazes from six (f, k) presets. Drag to paint life; adaptive quality keeps phones at 60 fps. One file, no libraries, no assets. |
| [Monkey Island — 3D intro](examples/monkeyisland/) | The Secret of Monkey Island Special Edition opening rebuilt live: moonlit island, ships, glowing logo, and the Main Title Theme synthesized from inline score data. Click to start, drag to orbit. One file, no assets. |
| [Commander Keen — self-playing demo](examples/commanderkeen/) | The box art alive at 320×200 in the 16-color Keen palette: a bot runs, jumps and lie-flat-shoots through two looping levels — Vorticons, Vortiguns, ick, 1UPs, the grinning Vorticomander — with AdLib-style FM audio. Press any key to take over. One file, no assets. |
| [Deluxe Ski Jump — Gore Edition](examples/skijump/) | The 1997 Finnish cult classic rebuilt as a two-button phone game: time the takeoff, lean through the flight, stick the telemark on the K line — gusts, FIS judges, nine hills from K8 to K190. The spicy detail: every landing sprays a little blood and crashes get gory, but the flying is the game. One file, no assets. |
| [Deluxe Ski Jump — Eyeball Edition](examples/skijumpfp/) | The Gore Edition's twin: side view through the inrun, but from the kick onward the camera lives in the jumper's eyeball — your own V-skis in frame, your own depth perception on the K line, blood on your own lens when you crash. Same physics and hills, hand-rolled 3D projection. One file, no assets. |
| [Dungeon Master — Torchlight Edition](examples/dungeonmaster/) | The 1986 classic rebuilt as a playable real-time crawler: torch-lit WebGL2 dungeons with fuel that burns down, a party of three, real-time grid combat, and spell casting by chaining magic runes. Descend forever. One file, no assets. |
| [DOOM — Terminated Edition](examples/terminatordoom/) | Doom rebuilt inside a Skynet sector: raycast steel-and-rust corridors, chrome endoskeletons hunting you in waves, red keycard, coolant barrels, a plasma carbine and a synthesized pulse score. One file, no assets. |
| [OutRun 3D](examples/outrun/) | The SEGA arcade poster as a live Three.js scene with the soundtrack synthesized from the arcade FM data. One file, zero assets. |

## Layout

```
index.html            ← example hub (served at the repo root)
examples/<name>/      ← one folder per example
  static              ← plain HTML/JS → served as-is
  + package.json      ← CI runs npm ci && npm run build and serves its dist/
.github/workflows/deploy-pages.yml   ← builds all examples, deploys one artifact
```

Adding an example = new folder under `examples/` + a card in `index.html`.
Built projects must use relative asset paths (`vite base: './'`, and resolve
runtime URLs against `document.baseURI`) so they work under the Pages subpath.

## Living room example — local dev

```bash
cd examples/livingroom
npm install
npm run dev   # http://localhost:5173
```
