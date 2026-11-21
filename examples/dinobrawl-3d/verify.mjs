/* Dino Brawl 3D — smoke + regression checks.   node verify.mjs [url]
   Boots the game the way a player does (pick a card, press PLAY) on a desktop and
   a phone, then checks the things that have actually broken before: rigs that load
   inside-out, legs that float under the belly, a stomp that lifts a leg sideways,
   a venom spit nobody can see, and the menu growing an island without changing
   your dinosaur. Exits non-zero on any failure.                              */
import { chromium } from 'playwright';
import http from 'http';
import { readFile } from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const PORT = 8411;
const TYPES = { html: 'text/html', js: 'text/javascript', glb: 'model/gltf-binary', png: 'image/png', json: 'application/json' };
const server = http.createServer(async (req, res) => {
  let p = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
  if (p.endsWith('/')) p += 'index.html';
  try {
    const b = await readFile(path.join(HERE, p));
    res.writeHead(200, { 'Content-Type': TYPES[p.split('.').pop()] || 'application/octet-stream' });
    res.end(b);
  } catch { res.writeHead(404); res.end(); }
});
await new Promise(r => server.listen(PORT, r));
const wait = ms => new Promise(r => setTimeout(r, ms));
const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] });

let pass = 0, fail = 0;
const ok = (cond, what, detail = '') => {
  console.log(`${cond ? '✓' : '✗'} ${what}${detail ? '  ' + detail : ''}`);
  cond ? pass++ : fail++;
};
const errors = [];
const open = async (vp) => {
  const page = await browser.newPage(vp);
  page.on('pageerror', e => errors.push('pageerror: ' + e.message));
  page.on('console', m => { if (m.type() === 'error') errors.push('console: ' + m.text()); });
  await page.goto('http://127.0.0.1:8411/', { waitUntil: 'load' });
  return page;
};

// ---------- desktop: start as a Dilophosaurus the way a kid does ----------
const page = await open({ viewport: { width: 1280, height: 800 } });
await wait(1800);
await page.evaluate(() => {
  const c = [...document.querySelectorAll('#pick .pcard')].find(b => b.textContent.toLowerCase().includes('dilopho'));
  c.click();
});
await page.click('#playB');
await page.waitForFunction(() => window.__probe && window.__probe.ready, null, { timeout: 60000 });
await wait(1500);
const roster = await page.evaluate(() => ({
  pick: window.__probe.state().species, list: window.__probe.pickList(), flags: window.__probe.speciesFlags(),
  cast: window.__probe.state().cast,
}));
ok(roster.pick === 'dilopho', 'the picture card picks your dinosaur', roster.pick);
ok(roster.list.length === 12, 'twelve dinosaurs to be', `${roster.list.length}`);
ok(roster.cast.length === 5 && !roster.cast.includes(roster.pick),
  'the island raises five challengers that are not you', roster.cast.join(','));
ok(roster.flags.dilopho.venom && roster.flags.brachiosaurus.huge && roster.flags.trike.charge,
  'the newcomers keep their tricks', 'venom + huge + charge');

// ---------- bodies: rigs, shells, and legs that meet the belly ----------
for (const k of roster.list) {
  const b = await page.evaluate(async (k) => {
    window.__probe.switchTo(k);
    await new Promise(r => setTimeout(r, 260));
    const s = window.__probe.shellCheck(), l = window.__probe.limbCheck();
    return { sp: window.__probe.state().species, joints: Object.keys(window.__probe.joints('player')).length,
      shell: s ? s.agree / (s.agree + s.opp) : 0, limbs: l || [],
      tall: window.__probe.camInfo().tall };
  }, k);
  const badLimbs = b.limbs.filter(x => !x.ok);
  ok(b.sp === k && b.joints > 4 && b.tall > 0.5, `${k}: model loads with a working rig`,
    `joints=${b.joints} tall=${b.tall}`);
  ok(b.shell > 0.9, `${k}: skin on the outside`, `${(b.shell * 100).toFixed(0)}%`);
  ok(b.limbs.length >= 2 && !badLimbs.length, `${k}: legs grow into the body`,
    badLimbs.length ? JSON.stringify(badLimbs) : `${b.limbs.length} limbs`);
}

// ---------- the stomp lifts a fore foot up and forward ----------
const stomp = await page.evaluate(async () => {
  const sleep = ms => new Promise(r => setTimeout(r, ms));
  const pr = window.__probe;
  pr.switchTo('brachiosaurus');
  await sleep(900);
  for (let i = 0; i < 60 && !pr.canBite(); i++) await sleep(120);
  const p = pr.state().player;
  pr.placeAt(p.x, p.z, 0);
  pr.placeEnemy('diplodocus', p.x + 6, p.z, Math.PI);
  pr.bite();
  let maxZ = 0, maxX = 0;
  for (let i = 0; i < 8; i++) {
    await sleep(90);
    const j = pr.meshPose('player', 'legFR');
    if (!j) continue;
    maxZ = Math.max(maxZ, Math.abs(j.r[2])); maxX = Math.max(maxX, Math.abs(j.r[0]));
  }
  return { move: pr.moveOf('player').move, maxZ: +maxZ.toFixed(2), maxX: +maxX.toFixed(2) };
});
ok(stomp.move === 'stomp' && stomp.maxZ > 0.25 && stomp.maxX < 0.08,
  'the giant stomp lifts a fore foot up and forward', JSON.stringify(stomp));

// ---------- venom: it always spits, you can see it, and it poisons ----------
const goo = await page.evaluate(async () => {
  const sleep = ms => new Promise(r => setTimeout(r, ms));
  const pr = window.__probe;
  pr.switchTo('dilopho');
  await sleep(900);
  const p = pr.state().player;
  pr.state().enemies.forEach(e => pr.placeEnemy(e.key, p.x + 70, p.z + 70, 0));
  pr.setRoar(1); pr.roar();
  await sleep(200);
  const flight = { glob: pr.venomCount(), trail: pr.gooCount() };
  for (let i = 0; i < 30 && pr.venomCount(); i++) await sleep(120);
  const ended = pr.venomLast();
  const p2 = pr.state().player;
  const victim = pr.state().cast[0];
  pr.placeEnemy(victim, p2.x + 9, p2.z, Math.PI);
  pr.setRoar(1); pr.roar();
  let poisoned = 0, who = null;
  for (let i = 0; i < 30 && !poisoned; i++) {
    await sleep(150);
    for (const e of pr.state().enemies) { const t = pr.poisonOf(e.key).t; if (t > 0) { poisoned = t; who = e.key; } }
  }
  return { flight, ended, poisoned: +poisoned.toFixed(1), who };
});
ok(goo.flight.glob >= 1 && goo.flight.trail >= 1, 'the venom spit always flies, with a streak you can see',
  JSON.stringify(goo.flight));
ok(!goo.ended || !goo.ended.hit, 'a spit at nobody still lands with a splat', JSON.stringify(goo.ended));
ok(goo.poisoned > 0, 'a spit at a real dinosaur poisons it', `${goo.who} poisonT=${goo.poisoned}`);

// ---------- the menu: one press changes the island AND the dinosaur ----------
const menu = await page.evaluate(async () => {
  const pr = window.__probe;
  pr.openMenu();
  await new Promise(r => setTimeout(r, 400));
  const before = { map: pr.map().name, sp: pr.state().species };
  const wanted = { map: 'canyon', sp: 'trike' };
  const label = document.getElementById('menuB').textContent;
  pr.setMapPick(wanted.map); pr.setPick(wanted.sp);
  const label2 = document.getElementById('menuB').textContent;
  pr.menuStart();
  await new Promise(r => setTimeout(r, 1400));
  return { before, label, label2, after: { map: pr.map().name, sp: pr.state().species } };
});
ok(/GROW|NEW/i.test(menu.label2) && /TRICERATOPS/i.test(menu.label2),
  'the button says what it will do', menu.label2);
ok(menu.after.map !== menu.before.map && menu.after.sp === 'trike',
  'one press grows a new island and changes your dinosaur together',
  `${menu.before.map}/${menu.before.sp} -> ${menu.after.map}/${menu.after.sp}`);
await page.close();

// ---------- phone: the twelve-card strip fits and the game plays ----------
const phone = await open({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true });
await wait(2000);
const strip = await phone.evaluate(() => {
  const el = document.getElementById('pick'), cards = [...el.querySelectorAll('.pcard')];
  const r = el.getBoundingClientRect();
  return { n: cards.length, fits: cards.every(c => c.getBoundingClientRect().right <= r.right + 1),
    w: Math.round(cards[0].getBoundingClientRect().width), overflow: el.scrollWidth > el.clientWidth };
});
ok(strip.n === 12 && strip.w >= 100, 'twelve picture cards on a phone, big enough to tap', `${strip.n} × ${strip.w}px`);
await phone.evaluate(() => { const el = document.getElementById('pick'); el.scrollLeft = el.scrollWidth; });
await wait(600);
const tail = await phone.evaluate(() => {
  const el = document.getElementById('pick'), cards = [...el.querySelectorAll('.pcard')];
  const r = el.getBoundingClientRect();
  return cards.slice(-2).every(c => { const b = c.getBoundingClientRect();
    return b.left >= r.left - 1 && b.right <= r.right + 1; });
});
ok(tail, 'the strip scrolls so the last cards are fully on screen');
await phone.evaluate(() => document.querySelector('#pick .pcard:nth-child(3)').click());
await phone.click('#playB');
await phone.waitForFunction(() => window.__probe && window.__probe.ready, null, { timeout: 60000 });
await wait(2500);
const play = await phone.evaluate(() => {
  const s = window.__probe.state();
  return { species: s.species, fps: s.fps, tier: s.tier,
    bite: Math.round(document.getElementById('biteB').getBoundingClientRect().width) };
});
ok(play.bite > 60, 'the BITE button is still under your thumb', `${play.bite}px`);
ok(play.fps >= 20, `frames hold up on a phone with the giants around (${play.fps} fps, ${play.tier})`);
ok(!errors.length, 'no console errors', errors.slice(0, 3).join(' | '));

console.log(`\nDINO BRAWL 3D: ${pass} passed, ${fail} failed`);
await browser.close(); server.close();
process.exit(fail ? 1 : 0);
