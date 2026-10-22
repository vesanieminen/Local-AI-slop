# Living Room — Three.js Handover

## Files
| File | Purpose |
|---|---|
| `livingroom.glb` | **The deliverable.** 30.3 MB, Draco-compressed (66/66 meshes), 19 materials, 14 embedded textures, 4 cameras, 1 punctual light. glTF 2.0, Y-up, meters. |
| `draco/` | Draco decoder (three r180 build) — point `DRACOLoader.setDecoderPath()` here for a turnkey setup. |
| `../livingroom.blend` | Full Cycles render scene (HDRI world, procedural detail) — NOT for web, source of truth for re-authoring. |
| `../livingroom_gltf.blend` | Export-prep scene (UVs, simplified Principled graphs) — use this if re-exporting. |
| `../assets/tex/` | Poly Haven 2K source maps (CC0). `../assets/tex/gltf/` holds baked tint/roughness variants (already embedded in the GLB). |
| `../assets/hdri/kloofendal_38d_partly_cloudy_puresky_4k.hdr` | The environment lighting — load with `RGBELoader` as `scene.environment`. Not embedded in the GLB. |

## Scene facts (verified against the GLB JSON)
- Units: **meters**, room 6 × 5 × 3 m, origin at room center, floor at y=0.
- 84/84 nodes named (`Floor`, `Sofa_Seat_L`, `Glass_Panel_2`, …) — safe to `getObjectByName`.
- Cameras (perspective, aspect 1.7778):
  - `Cam_ThreeQuarter` yfov 0.899 (24 mm hero), `Cam_ToWindow` 0.863 (22 mm),
    `Cam_Detail` 0.40 (50 mm vignette), `Cam_Plant` 0.694 (35 mm backlit corner).
- Materials (19, all glTF PBR metallic-roughness): texture-based — `M_OakFloor`,
  `M_Plaster`, `M_LinenCharcoal`, `M_Walnut`, `M_WoolRug` (baseColor + metallicRoughness + normal, UVs at real-world tiling);
  constant-based — metals, ceramics, stone, foliage, books, ceiling, soil.
- Extensions in use: `KHR_draco_mesh_compression`, `KHR_materials_pbrSpecularGlossiness` not used,
  `KHR_materials_sheen` (linen), `KHR_materials_clearcoat` (walnut),
  `KHR_materials_transmission` + `KHR_materials_ior` (window glass),
  `KHR_materials_emissive_strength` (bulbs, shade), `KHR_materials_specular`, `KHR_lights_punctual`.
- `M_Curtain`: `alphaMode BLEND`, alpha 0.6, doubleSided — set `depthWrite:false` and
  render it last for correct sorting.
- Sampler state: LINEAR / LINEAR_MIPMAP_LINEAR, sRGB on baseColor — defaults are correct in Three.

## Loader
```js
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { DRACOLoader } from 'three/addons/loaders/DRACOLoader.js';
import { RGBELoader } from 'three/addons/loaders/RGBELoader.js';

const draco = new DRACOLoader().setDecoderPath('./draco/');
const gltf = new GLTFLoader().setDRACOLoader(draco);

gltf.load('livingroom.glb', (g) => {
  scene.add(g.scene);
  // cameras come in as THREE.PerspectiveCamera nodes named Cam_*
  const cam = g.scene.getObjectByName('Cam_ThreeQuarter').children[0] ?? null; // camera nodes are node-mounted
});

// environment (required — HDRI does not travel in glTF):
new RGBELoader().load('kloofendal_38d_partly_cloudy_puresky_4k.hdr', (tx) => {
  tx.mapping = THREE.EquirectangularReflectionMapping;
  scene.environment = tx;
});

renderer.toneMapping = THREE.AgXToneMapping;   // matches the Blender look
renderer.toneMappingExposure = 0.9;
```

## What does NOT transfer (and what to do about it)
1. **HDRI world lighting** — bake it in with `scene.environment` (RGBELoader), and orient it
   with `scene.environmentRotation` so the sun enters through the +Z window wall in Y-up
   (Blender world rotation Z = 335.6°; in Three the HDRI's sun is at equirect u ≈ 0.11 — verify visually).
2. **Pendant area light + 3 GI portals** — glTF punctual lights have no area type.
   Recreate: `RectAreaLight` (~40 W, 0.16 m, color `0xFFA15C`) under `Pendant_Shade`,
   plus the exported point light (`Light_FloorLamp`) comes automatically via KHR_lights_punctual.
3. **Emissive glow/bloom** — bulbs export with emissiveStrength 25 (clamped by the extension value,
   still strong); visible glow needs a bloom pass (UnrealBloom / postprocessing).
4. **Leaf subsurface** — leaves are constant albedo (doubleSided); no translucency.
5. **Glass transmission** — needs r167+ with the transmissive rendering feature enabled,
   or swap `M_WindowGlass` to `transparent:true, opacity:0.12` for cheap fallback.
6. **Curtain softness** — alpha-blend only; no sheen shading on the sheer.
7. **Cycles-only extras** — shadow ray trick on glass, AgX grade, light portals: re-approach
   with Three tooling (contact shadows / SSAO if needed).

## Performance notes
- Draco quantization: position 14 bits (~0.1 mm over the room), normals 10 bits, UVs 10 bits.
- Total tris ≈ 45–60 k after decompression (bevels applied at export). All single-UV-set;
  UVs intentionally overlap (world-scale tiling) — do not lightmap/atlas without re-UVing.
- Textures embedded: 5 × 2K JPG sets (floor/plaster/linen/walnut/wool). If GLB size matters,
  1K downscale halves it with little visual loss on web.
- Every material is `MeshPhysicalMaterial`-compatible; the sheen/clearcoat extensions will
  activate only on `MeshPhysicalMaterial` (GLTFLoader does this automatically).
