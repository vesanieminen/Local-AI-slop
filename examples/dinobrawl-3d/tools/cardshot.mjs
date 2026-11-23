/* Dino Brawl 3D — select-card shots.   node tools/cardshot.mjs <species> [more...]
   Renders assets/select/<key>.png exactly like the Blender cardrender.py cards
   (flat Lambert, same turntable angles), from the game's own GLBs — no Blender
   needed. Run from the example folder.                                            */
import { chromium } from 'playwright';
import http from 'http';
import { readFile, writeFile } from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

const HERE = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const PORT = 8413;
const TYPES = { html: 'text/html', js: 'text/javascript', mjs: 'text/javascript', glb: 'model/gltf-binary', png: 'image/png' };
const server = http.createServer(async (req, res) => {
  let p = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
  try {
    const b = await readFile(path.join(HERE, p));
    res.writeHead(200, { 'Content-Type': TYPES[p.split('.').pop()] || 'application/octet-stream' });
    res.end(b);
  } catch { res.writeHead(404); res.end(); }
});
await new Promise(r => server.listen(PORT, r));
const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] });
const page = await browser.newPage({ viewport: { width: 360, height: 220 } });
for (const key of process.argv.slice(2)) {
  await page.goto(`http://127.0.0.1:${PORT}/tools/cardshot.html?key=${key}`, { waitUntil: 'load' });
  await page.waitForFunction(() => window.__card, null, { timeout: 60000 });
  const data = await page.evaluate(() => window.__card);
  if (data.startsWith('ERR')) { console.error(`${key}: ${data}`); continue; }
  const out = path.join(HERE, 'assets', 'select', `${key}.png`);
  await writeFile(out, Buffer.from(data.split(',')[1], 'base64'));
  console.log(`${key}: ${out}`);
}
await browser.close(); server.close();
