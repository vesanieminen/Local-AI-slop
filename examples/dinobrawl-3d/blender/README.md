# How these dinosaurs are made

**Standing rule (owner decision, 2026-11-21): all 3D models for this repo are
built through Blender MCP on the owner's live Blender** (`execute_blender_code`
etc. — see the "Blender — always" section of the global AGENTS.md). Headless or
standalone model factories are what produced anatomically broken dinos before
the rework; they are not used. This folder is the record of the pipeline so any
future model (or rebuild) can reuse it.

## Files

| File | What it is |
|---|---|
| `new_dinos.blend` | The working scene + Text blocks (`dt`, `build_*`, `cardrender`). Provenance for every GLB in `../assets/`. |
| `dt.py` | The `dt` helper toolkit: jointed-rig builders (`joint`, `tube`, `blob`, `plate`, `spike_line`, `eye_pair`), vertex-colour painting (`paint`), GLB export (`export_glb`) and select-card renderers. Reconstructed from the live session and **verified instruction-for-instruction against the live module that built the shipped models** — running it reproduces the exact toolkit. Ends by registering itself as `sys.modules["dt"]`. |
| `build_<species>.py` | One declarative recipe per species (Tyrannosaurus-era twelve + Mosasaurus): joints, tubes/blobs, plates, teeth, eyes, colours. Same content as the Text blocks in the .blend. Every tail (and the sauropod necks) end with `dt.flare(...)` loops — sleeves that slide each segment over the one ahead so the chain reads as one continuous limb (see the "patch FIX @tail-continuity" block in `dt.py`). |
| `fix_tails.py` | The tail-continuity repair for the six models that predate build scripts (`trex, allo, spino, ankylo, dr, im`): slopes the flat rear wall into the tail root, telescopes the segments together and adds a body-parented hip collar, run live on the imported GLBs and re-exported. |
| `cardrender.py` | Script that renders the in-game select-screen cards (`../assets/select/*.png`) with `dt.render_card2`. |

> **No Blender running?** `../tools/cardshot.mjs` renders the same 320×180
> Lambert cards headlessly from each `assets/<species>.glb` using the game's
> own three.js + GLTFLoader (no Blender needed): `node tools/cardshot.mjs
> mosasaurus` writes `assets/select/mosasaurus.png`. It mirrors `cardrender.py`'s
> turntable (yaw −0.62, pitch 0.16, fit 0.9) and flat shading.

## Coordinate system

x = forward, y = lateral, z = up, metres. Model units are metres; the game
scales with `SPECIES[...].scale`.

## Making a new model (or rebuilding one)

1. Connect to the live Blender through the Blender MCP tools.
2. Run `dt.py` once (paste as a Text block or
   `exec(open(".../blender/dt.py").read())`) — it registers `sys.modules["dt"]`.
3. Run a `build_*.py` (Text block or exec) — it clears the scene and rebuilds
   the rig + meshes. Calibrate `tube` radii/positions against the viewport
   (`look`) as you go; the whole point of live Blender is that you *see* it.
4. `dt.export_glb("root", ".../assets/<species>.glb")`.
5. Add the species to the game (`SPECIES`, pick cards, cast rolls), then run
   `node ../verify.mjs` — its rig/shell/limb checks (e.g. `limbCheck` for
   flipper/leg attachment to the body) guard against detached limbs.
6. Commit the `.blend` (with updated Text blocks), the build script, and the
   GLB together.
