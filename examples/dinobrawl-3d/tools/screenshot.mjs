/* Dino Brawl 3D — hub screenshot.   node tools/screenshot.mjs
   Opens the game like a player (pick a Dilophosaurus, press PLAY), stages a
   venom spit with the four big newcomers posed around it, and saves the
   1280x800 shot used as the hub thumbnail (examples/dinobrawl-3d/screenshot.png). */
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
  const p = pr.state().player;
  pr.placeAt(0, 0, -Math.PI / 2);                              // island middle, looking across it
  pr.setCam(Math.PI);                                          // camera due south of us, facing +z over the posed dino
  await wait(120);
  const q = pr.state().player;
  const fx = Math.cos(q.yaw), fz = -Math.sin(q.yaw);           // the way the player and camera look
  const rx = -fz, rz = fx;                                     // screen-right
  const pose = (key, fwd, side) => {
    const x = q.x + fx * fwd + rx * side, z = q.z + fz * fwd + rz * side;
    pr.placeEnemy(key, x, z, Math.atan2(-(q.z - z), x - q.x)); // face the player
  };
  pose('trike', 18, -8); pose('stego', 33, 4); pose('brachiosaurus', 42, 2);
  pose('diplodocus', 46, 18); pose('raptor', -60, 40);         // parked out of frame
  await wait(2000);                                           // chase camera settles
  return { cast: pr.state().cast,
    player: pr.state().player,
    enemies: pr.state().enemies.map(e => [e.key, e.x, e.z, e.alive, e.state, e.hp]) };
});
console.log(JSON.stringify(staged.enemies.map(e => e[0] + '@' + e[1] + ',' + e[2]).join(' ')));
await page.evaluate(() => { window.__probe.setRoar(1); window.__probe.roar(); });  // the photographed spit
await new Promise(r => setTimeout(r, 450));                    // glob mid-arc, right after the PFFFTT
await page.screenshot({ path: path.join(ROOT, 'screenshot.png') });
await browser.close(); server.close();
console.log('screenshot.png written');
