/* Dino Brawl 3D — hub screenshot.   node tools/screenshot.mjs
   Opens the game like a player (pick a Dilophosaurus, press PLAY), stages a
   venom spit with the big newcomers posed on a flat beach, the Mosasaurus
   lurking offshore and one of its bait fish washed up at the tide line, and
   saves the 1280x800 shot used as the hub thumbnail
   (examples/dinobrawl-3d/screenshot.png).                                        */
import { chromium } from 'playwright';
import http from 'http';
import { readFile } from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const PORT = 8421;
const TYPES = { html: 'text/html', js: 'text/javascript', glb: 'model/gltf-binary', png: 'image/png', json: 'application/json' };
const server = http.createServer(async (req, res) => {
  let p = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
  if (p.endsWith('/')) p += 'index.html';
  try {
    const b = await readFile(path.join(ROOT, p));
    res.writeHead(200, { 'Content-Type': TYPES[p.split('.').pop()] || 'application/octet-stream' });
    res.end(b);
  } catch { res.writeHead(404); res.end(); }
});
await new Promise(r => server.listen(PORT, r));
const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'], viewport: { width: 1280, height: 800 } });
const page = await browser.newPage();
await page.goto(`http://127.0.0.1:${PORT}/`, { waitUntil: 'load' });
await page.waitForSelector('#pick .pcard', { timeout: 30000 });
await page.evaluate(() => {
  [...document.querySelectorAll('#pick .pcard')].find(b => b.textContent.toLowerCase().includes('dilopho')).click();
});
await page.click('#playB');
await page.waitForFunction(() => window.__probe && window.__probe.ready, null, { timeout: 60000 });
await page.waitForTimeout(1200);

const staged = await page.evaluate(async () => {
  const pr = window.__probe;
  const wait = ms => new Promise(r => setTimeout(r, ms));
  pr.setCast(['trike', 'stego', 'brachiosaurus', 'diplodocus', 'raptor']);
  pr.newExpedition('forest', 20261121, true);
  for (let i = 0; i < 40 && pr.state().enemies.filter(e => e.alive).length < 5; i++) await wait(200);
  // deterministic: scan fixed angles for a flat beach with open water straight out.
  // trees never grow below h=1.4, so a corridor kept under that is guaranteed tree-free
  const openCorridor = (x, z, nx, nz) => {
    for (let d = -5; d <= 24; d += 2) for (let s = -7; s <= 7; s += 2)
      if (pr.ground(x + nx * d + nz * s, z + nz * d - nx * s) >= 1.45) return false;
    return true;
  };
  let shore = null;
  for (let i = 0; i < 360 && !shore; i++) {
    const a = (i * 3.7 % 360) * Math.PI / 180;               // golden-ish angle walk
    for (let r = 160; r <= 248 && !shore; r += 3) {          // the coastline of the big world
      const x = Math.cos(a) * r, z = Math.sin(a) * r, h = pr.ground(x, z);
      if (!(h > 0.45 && h < 1.35)) continue;
      const nx = Math.cos(a), nz = Math.sin(a);
      let sea = 0;
      for (let d = 15; d <= 26; d += 1.5) if (pr.ground(x + nx * d, z + nz * d) < -1.5) { sea = d; break; }
      if (!sea || !openCorridor(x, z, nx, nz)) continue;
      shore = { x, z, nx, nz, sea };
    }
  }
  if (!shore) return { cast: null, mos: null, fail: 'no sea in frame' };
  const yaw = Math.atan2(-shore.nz, shore.nx);               // look straight out to sea
  pr.placeAt(shore.x, shore.z, yaw);
  pr.setCam(yaw - Math.PI / 2);                              // camera behind us, over the shoulder
  await wait(250);
  const q = pr.state().player;
  const fx = Math.cos(q.yaw), fz = -Math.sin(q.yaw);         // the way the player and camera look
  const rx = -fz, rz = fx;                                   // screen-right
  const pose = (key, fwd, side) => {                        // on the sand or wading, never drowning
    const deep = key === 'brachiosaurus' || key === 'diplodocus';   // tall long-necks wade deeper
    for (const [f, s] of [[fwd, side], [fwd - 4, side], [fwd, side + 3], [fwd, side - 3], [fwd + 3, side], [fwd - 8, side]]) {
      const x = q.x + fx * f + rx * s, z = q.z + fz * f + rz * s;
      if (pr.ground(x, z) < (deep ? -2.4 : -0.8)) continue;
      pr.placeEnemy(key, x, z, Math.atan2(-(q.z - z), x - q.x));  // face the player
      return true;
    }
    return false;
  };
  // the sea monster: the first deep water straight ahead, nudged left of centre
  let mx = null, mz = null;
  for (const s of [3, 6, 0, -3]) {
    let found = false;
    for (let d = shore.sea + 3; d >= 12 && !found; d -= 1.5) {
      const x = q.x + fx * d - rx * s, z = q.z + fz * d - rz * s;
      if (pr.ground(x, z) < -1.2) { mx = x; mz = z; found = true; }
    }
    if (found) break;
  }
  const parkRaptor = () => pose('raptor', -60, 40);          // parked out of frame
  const rePose = () => {
    const ok = [pose('trike', 8, -7), pose('stego', 12, 6), pose('diplodocus', 15, 12),
      pose('brachiosaurus', 19, -1), parkRaptor()];
    if (mx !== null) pr.setEnemy('mosasaurus', { x: mx, z: mz, cool: 99, fear: 0, state: 'idle',
      baitT: 9e9,                                  // no sudden urge to waddle ashore mid-photo
      yaw: Math.atan2(-(q.z - mz), mx - q.x) });            // jaws pointed at the beach
    return ok;
  };
  let okPosed = rePose();
  await wait(2000);                                          // chase camera settles
  okPosed = rePose();                                        // snap them back before the shutter
  // the Mosasaurus's bait: a fat fish washed up at the tide line, straight ahead.
  // dropped last so the posed dinos are still mid-turn toward the camera, not the snack
  let bait = null;
  outer: for (const s of [-2.5, 0, 2.5, -4.5, 4.5]) {
    for (let d = shore.sea + 2; d >= 4; d -= 0.8) {        // find the tide line wherever it runs
      const x = q.x + fx * d - rx * s, z = q.z + fz * d - rz * s, h = pr.ground(x, z);
      if (h > -0.4 && h < 0.8 && pr.dropBait(x, z).ok) { bait = [+x.toFixed(0), +z.toFixed(0), +h.toFixed(2)]; break outer; }
    }
  }
  return { okPosed, sp: pr.state().species, cast: pr.state().cast, mos: pr.mosaInfo(), shore, bait,
    enemies: pr.state().enemies.map(e => [e.key, e.x, e.z, e.alive, e.state]) };
});
console.log(staged.sp, staged.cast ? staged.cast.join(',') : 'STAGING FAIL ' + staged.fail, JSON.stringify(staged.mos),
  'bait:' + JSON.stringify(staged.bait),
  JSON.stringify((staged.enemies || []).map(e => e[0] + '@' + e[1] + ',' + e[2])));
await page.evaluate(() => { window.__probe.setRoar(1); window.__probe.roar(); });  // the photographed spit
await new Promise(r => setTimeout(r, 450));                      // glob mid-arc, right after the PFFFTT
await page.screenshot({ path: path.join(ROOT, 'screenshot.png') });
await browser.close(); server.close();
console.log('screenshot.png written');
