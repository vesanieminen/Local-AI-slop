# Living Room — WebGPU real-time viewer

Real-time Three.js viewer for a Blender living-room scene, matched to the
Cycles look (AgX tone mapping, +0.2 EV exposure, same Poly Haven HDRI
`kloofendal_38d_partly_cloudy_puresky_4k.hdr`).

- WebGPURenderer (three r186), with bloom + GTAO post-processing
- GLB loaded from `public/export/livingroom.glb` (Draco-compressed, local decoder)
- Recreated 42 W pendant area light; the floor lamp comes from KHR_lights_punctual
- Sun direction/color/intensity measured from the HDRI at startup; shadow-casting
- Orbit controls + HUD sliders (exposure, env rotation/intensity, sun, bloom, AO)
- Camera presets = the 4 `Cam_*` nodes exported from Blender

## Run

```bash
npm install
npm run dev   # http://localhost:5173
```

Requires a WebGPU browser (Chrome/Edge with a recent GPU).

## Deploy

Push to `main` → GitHub Actions builds `dist/` and deploys to GitHub Pages
(`.github/workflows/deploy-pages.yml`; set Pages source to "GitHub Actions").
