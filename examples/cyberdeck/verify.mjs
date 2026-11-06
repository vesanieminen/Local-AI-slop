// Verification for the CALC//DECK example.
//
//   1) web checks:   NODE_PATH=<dir with playwright> node verify.mjs [--url http://127.0.0.1:PORT/]
//      headless Chromium + WebGPU: zero console/page/GPU errors, real GPU
//      submissions at desktop and phone viewports, canvas sizing, and the deck
//      stays alive after a tap. Pixels can't be captured from this Chromium's
//      WebGPU compositing (see README), so visual proof comes from step 2.
//   2) pixel checks: node verify.mjs --png snap.png [--png b.png ...]
//      decodes the PNGs produced by snap.sh (Rune's native headless --snap)
//      and asserts ink + the cyan/magenta Ghostwire accents actually rendered.
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
import { inflateSync } from 'node:zlib';

const argv = process.argv.slice(2);
const flag = (name) => argv.includes(name);
const get = (name, dflt) => { const i = argv.indexOf(name); return i >= 0 ? argv[i + 1] : dflt; };

// --- PNG decode (non-interlaced 8-bit RGB/RGBA/palette) ---
function pngPixels(buf) {
  let pos = 8, w = 0, h = 0, color = 0, depth = 8;
  const idat = [];
  let palette = null;
  while (pos < buf.length) {
    const len = buf.readUInt32BE(pos);
    const type = buf.toString('ascii', pos + 4, pos + 8);
    const data = buf.subarray(pos + 8, pos + 8 + len);
    if (type === 'IHDR') { w = data.readUInt32BE(0); h = data.readUInt32BE(4); depth = data[8]; color = data[9]; }
    else if (type === 'PLTE') palette = data;
    else if (type === 'IDAT') idat.push(data);
    else if (type === 'IEND') break;
    pos += 12 + len;
  }
  if (depth !== 8) throw new Error('only 8-bit PNG supported, got depth ' + depth);
  const ch = ({ 0: 1, 2: 3, 3: 1, 4: 2, 6: 4 })[color];
  if (!ch) throw new Error('unsupported PNG color type ' + color);
  const raw = inflateSync(Buffer.concat(idat));
  const stride = w * ch;
  const out = Buffer.alloc(h * stride);
  for (let y = 0; y < h; y++) {
    const filter = raw[y * (stride + 1)];
    const line = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
    const cur = out.subarray(y * stride, (y + 1) * stride);
    const prev = y > 0 ? out.subarray((y - 1) * stride, y * stride) : Buffer.alloc(stride);
    for (let x = 0; x < stride; x++) {
      const a = x >= ch ? cur[x - ch] : 0, b = prev[x], c = x >= ch ? prev[x - ch] : 0;
      let v = line[x];
      if (filter === 1) v += a; else if (filter === 2) v += b; else if (filter === 3) v += (a + b) >> 1;
      else if (filter === 4) {
        const p = a + b - c, pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
        v += pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
      }
      cur[x] = v & 0xff;
    }
  }
  if (color === 3) {
    const rgb = Buffer.alloc(w * h * 3);
    for (let i = 0; i < w * h; i++) {
      const p = out[i] * 3;
      rgb[i * 3] = palette[p]; rgb[i * 3 + 1] = palette[p + 1]; rgb[i * 3 + 2] = palette[p + 2];
    }
    return { w, h, bpp: 3, data: rgb };
  }
  return { w, h, bpp: ch, data: out };
}

function imgData(path) {
  const img = pngPixels(readFileSync(path));
  const out = { w: img.w, h: img.h, ink: 0, cyan: 0, magenta: 0 };
  const { bpp, data } = img;
  for (let i = 0; i < data.length; i += bpp) {
    const [r, g, b] = [data[i], data[i + 1], data[i + 2]];
    if (r + g + b > 330) out.ink++;
    // Ghostwire channel: cyan #38f2ff accents and a magenta #ff38c4 sun/signs.
    if (b > 120 && g > 120 && g > r + 40 && b > r + 40) out.cyan++;
    if (r > 140 && b > 120 && r > g + 40) out.magenta++;
  }
  return out;
}

let failed = 0;
function report(name, ok, detail) {
  console.log(`${name}: ${ok ? 'OK' : 'FAIL'} — ${detail}`);
  if (!ok) failed++;
}

if (flag('--png')) {
  for (const p of argv.slice(argv.indexOf('--png') + 1).filter((a) => a.endsWith('.png'))) {
    const s = imgData(p);
    report(`png ${p}`, s.ink > 3000 && s.cyan > 800 && s.magenta > 40, `${s.w}x${s.h} ink=${s.ink} cyan=${s.cyan} magenta=${s.magenta}`);
  }
  process.exit(failed ? 1 : 0);
}

// --- web checks ---
const { chromium } = createRequire(import.meta.url)('playwright');
const BASE = get('--url', 'http://127.0.0.1:8123/');
const browser = await chromium.launch({ headless: true, args: ['--enable-unsafe-webgpu'] });

async function probe(name, url, { width, height, hasTouch = false, mobile = false, wantSubmits, settle = 1500, tap = false }) {
  const ctx = await browser.newContext({ viewport: { width, height }, hasTouch, isMobile: mobile, colorScheme: 'dark' });
  await ctx.addInitScript(() => {
    window.__probe = { submits: 0, gpuErrors: [] };
    const submit = GPUQueue.prototype.submit;
    GPUQueue.prototype.submit = function (...a) { window.__probe.submits++; return submit.apply(this, a); };
    const request = GPUAdapter.prototype.requestDevice;
    GPUAdapter.prototype.requestDevice = async function (...a) {
      const d = await request.apply(this, a);
      d.addEventListener('uncapturederror', (e) => window.__probe.gpuErrors.push(e.error?.message || String(e)));
      return d;
    };
  });
  const page = await ctx.newPage();
  const errors = [];
  page.on('console', (m) => { if (m.type() === 'error') errors.push('console: ' + m.text()); });
  page.on('pageerror', (e) => errors.push('pageerror: ' + e.message));
  await page.goto(BASE + url, { waitUntil: 'load' });
  await page.waitForFunction(() => window.__probe.submits > 0, null, { timeout: 15000 });
  const size = await page.locator('canvas').evaluate((c) => [c.width, c.height]);
  const early = await page.evaluate(() => window.__probe.submits);
  await page.waitForTimeout(settle);
  if (tap) {
    // punch a keypad key roughly where the "7" sits on both layouts, then
    // make sure the deck survived the hit.
    await page.mouse.click(width * 0.14, height * (mobile ? 0.52 : 0.52));
    await page.waitForTimeout(400);
  }
  const now = await page.evaluate(() => window.__probe.submits);
  const gpuErrors = await page.evaluate(() => window.__probe.gpuErrors);
  const moved = now - early;
  // The city never sleeps (rain, blink, rain ticker on a ~16-25 ms beat), so
  // "at rest" is a live ambient frame rate, never zero.
  const submitsOk = wantSubmits === 'ambient' ? moved >= 15 : moved >= wantSubmits;
  const sizeOk = size[0] >= width * 0.9 && size[1] >= height * 0.9;
  const ok = errors.length === 0 && gpuErrors.length === 0 && submitsOk && sizeOk;
  report(name, ok, `canvas=${size.join('x')} submits=${now} (Δ${moved}, want ${wantSubmits}) errors=${errors.length + gpuErrors.length}`);
  for (const e of errors) console.log('  ' + e);
  for (const e of gpuErrors) console.log('  gpu: ' + e);
  await ctx.close();
}

await probe('web-desktop-load', '', { width: 1280, height: 800, wantSubmits: 'ambient' });
await probe('web-desktop-demo', '?demo', { width: 1280, height: 800, wantSubmits: 'ambient' });
await probe('web-desktop-hero', '?hero', { width: 1280, height: 800, wantSubmits: 'ambient' });
await probe('web-mobile-load', '', { width: 390, height: 844, hasTouch: true, mobile: true, wantSubmits: 'ambient', tap: true });
await probe('web-mobile-hero', '?hero', { width: 390, height: 844, hasTouch: true, mobile: true, wantSubmits: 'ambient' });

await browser.close();
process.exit(failed ? 1 : 0);
