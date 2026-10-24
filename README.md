# Local AI slop 🧪

Browser experiments, served as GitHub Pages from one repo:

**▶ Hub: https://vesanieminen.github.io/Local-AI-slop/**

| Example | What it is |
|---|---|
| [OutRun 3D](examples/outrun/) | The SEGA arcade poster as a live Three.js scene with the soundtrack synthesized from the arcade FM data. One file, zero assets. |
| [Black hole](examples/blackhole/) | Every pixel raymarches an actual photon orbit around a Schwarzschild black hole: gravitational lensing, Doppler-beamed accretion disk, photon ring. One file, no libraries, no assets. |
| [Coral — reaction-diffusion](examples/coral/) | Gray-Scott reaction-diffusion on ping-ponged half-float textures: coral, mitosis, worms, fingerprints and mazes from six (f, k) presets. Drag to paint life; adaptive quality keeps phones at 60 fps. One file, no libraries, no assets. |
| [Living room — WebGPU](examples/livingroom/) | Blender-exported living room at 160 fps with WebGPURenderer: AgX matched to the Cycles look, same HDRI, GTAO + bloom. Needs a WebGPU browser; ~50 MB. |

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
