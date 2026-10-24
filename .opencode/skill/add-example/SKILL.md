---
name: add-example
description: Add, update, or publish a browser example in this Local-AI-slop repo — creating examples/<name>/, generating screenshot.png thumbnails, updating the index.html hub card and README table, verifying the page headlessly (including mobile), and pushing to GitHub Pages. Use whenever an example is added or changed, when hub listings or thumbnails need updating, and when shipping work to the repo, even if the user doesn't spell out the listing/push steps.
---

# Adding / publishing an example

This repo is a GitHub Pages hub of browser experiments. Every example ships at
`https://vesanieminen.github.io/Local-AI-slop/examples/<name>/`, and the hub at
the repo root links them. The routine below is what makes an example actually
*appear* correctly on the live site — creating the demo is only the first third
of the job.

## 1. Create the example

- New folder `examples/<name>/` (lowercase, short).
- Prefer one self-contained `index.html`: no libraries, no assets, no network.
  That is the repo's house style (see OutRun, black hole, coral).
- If it must be built, include `package.json` — CI runs `npm ci && npm run build`
  and serves `dist/`. Built projects must use relative asset paths
  (`vite base: './'`, runtime URLs resolved against `document.baseURI`) or they
  break under the Pages subpath.
- **Mobile is not optional**: full-viewport canvas, `touch-action: none` for
  pointer-driven demos, `env(safe-area-inset-*)` padding on controls, and
  adaptive quality (scale internal resolution / work per frame by measured
  frame time). Verify at 390×844 with touch emulation, not just desktop.

## 2. Generate the hub thumbnail

Each example needs a `screenshot.png` (or `thumb.jpg`) **at the example folder
root** — CI copies `examples/<name>/*.png|*.jpg` next to the served example so
the same path works in both branch-serving and Pages-artifact modes.

- Match the hub card's 16:10 crop: capture at 1280×800.
- Use Playwright's chromium headless with software GL:
  `npx playwright screenshot` or a small script with
  `--use-gl=angle --use-angle=swiftshader --enable-unsafe-swiftshader`.

**Thumbnails must show the demo doing its thing.** A headless run that renders
"no errors" can still be a black screen — common with simulations and shaders
that need time or parameters that actually converge. Make the capture script
wait long enough for the scene to mature, and assert on content: expose a
state object from the page (e.g. `window.__perf`) or sample pixels, and check
the screenshot image itself before committing. Keep URLs/tests deterministic
(seed the sim, or support query overrides like `?steps=NNN`).

Note: software GL is ~100× slower than a real GPU. For shader sims, pin work
per frame via URL override so enough simulation steps happen within the wait.

## 3. Update the listings (both, always)

- `index.html` (the hub): add an `<a class="card">` block inside `.grid` —
  `img` (`src="examples/<name>/screenshot.png"`, `loading="lazy"`, real alt),
  `<h2>` title, one `<p>` of plain-language what/why, `.tags` chips with the
  tech. Cards are ordered **coolest first** (the owner ranks by wow-factor —
  currently living room, black hole, OutRun, coral); slot a new example where
  its coolness belongs rather than at the end.
- `README.md`: add a row to the examples table — `[Name](examples/<name>/)`
  plus a one-sentence summary in the same voice as the others. Keep the table
  rows in the same order as the hub cards.

If an existing card's image is missing (hub shows a broken thumbnail),
regenerate it the same way rather than removing the `<img>`.

## 4. Verify before pushing

Load every touched page headlessly and require zero `console` errors and zero
`pageerror`s — both desktop (1280×800) and mobile (390×844, `hasTouch`)
viewports. For examples that animate, read the state probe or pixels to
confirm the scene is alive at capture time.

## 5. Commit and push

Push to `main` — `.github/workflows/deploy-pages.yml` builds all
`package.json` examples, assembles the site (root `index.html` + each example
dir or its `dist/` + example images) and deploys to Pages in one artifact.

```bash
git add <paths> && git commit && git push
```

Commit style: single-line, what changed and how, e.g.
"Black hole example: per-pixel Schwarzschild photon-orbit raymarching
(single-file WebGL2, no deps)". Include the hub/README updates in the same
commit as the example so the listing and the thing listed never diverge.
