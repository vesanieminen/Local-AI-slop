/* Dino Brawl 3D — smoke + regression checks.   node verify.mjs [url]
   Boots the game the way a player does (pick a card, press PLAY) on a desktop and
   a phone, then checks the things that have actually broken before: rigs that load
   inside-out, legs that float under the belly, a stomp that lifts a leg sideways,
   a venom spit nobody can see, and the menu growing an island without changing
   your dinosaur. Exits non-zero on any failure.                              */
import { chromium } from 'playwright';
import http from 'http';
import { readFile } from 'fs/promises';
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
ok(roster.flags.mosasaurus.sea && roster.flags.trike.herb && roster.flags.stego.herb,
  'the Mosasaurus owns the sea, the plant-eaters are peaceful by default', 'sea + herb');

// ---------- the Mosasaurus: a real animal in a real sea ----------
const mosa = await page.evaluate(async () => {
  await new Promise(r => setTimeout(r, 400));
  const m = window.__probe.mosaInfo();
  const s = window.__probe.shellCheck('mosasaurus'), l = window.__probe.limbCheck('mosasaurus');
  return { m, joints: Object.keys(window.__probe.joints('mosasaurus')).length,
    shell: s ? s.agree / (s.agree + s.opp) : 0, limbs: l || [] };
});
ok(mosa.m && mosa.m.alive && mosa.m.g < -2, 'a Mosasaurus patrols the deep water', JSON.stringify(mosa.m));
const badLim = mosa.limbs.filter(x => !x.ok);
ok(mosa.joints > 4 && mosa.shell > 0.9 && mosa.limbs.length >= 2 && !badLim.length,
  'the Mosasaurus is built right',
  `joints=${mosa.joints} shell=${(mosa.shell * 100).toFixed(0)}% limbs=${mosa.limbs.length}${badLim.length ? ' ' + JSON.stringify(badLim) : ''}`);

// ---------- bodies: rigs, shells, and legs that meet the belly ----------
for (const k of roster.list) {
  const b = await page.evaluate(async (k) => {
    window.__probe.switchTo(k);
    await new Promise(r => setTimeout(r, 260));
    const s = window.__probe.shellCheck(), l = window.__probe.limbCheck();
    return { sp: window.__probe.state().species, joints: Object.keys(window.__probe.joints('player')).length,
      shell: s ? s.agree / (s.agree + s.opp) : 0, bad: s && s.bad, limbs: l || [],
      tall: window.__probe.camInfo().tall };
  }, k);
  const badLimbs = b.limbs.filter(x => !x.ok);
  ok(b.sp === k && b.joints > 4 && b.tall > 0.5, `${k}: model loads with a working rig`,
    `joints=${b.joints} tall=${b.tall}`);
  ok(b.shell > 0.9, `${k}: skin on the outside`,
    `${(b.shell * 100).toFixed(0)}%${b.bad && b.bad.length ? ' ' + JSON.stringify(b.bad) : ''}`);
  ok(b.limbs.length >= 2 && !badLimbs.length, `${k}: legs grow into the body`,
    badLimbs.length ? JSON.stringify(badLimbs) : `${b.limbs.length} limbs`);
}

// ---------- the stomp lifts a fore foot up and forward ----------
const stomp = await page.evaluate(async () => {
  const sleep = ms => new Promise(r => setTimeout(r, ms));
  const pr = window.__probe;
  pr.switchTo('brachiosaurus');
  await sleep(900);
  for (let i = 0; i < 40 && pr.state().species !== 'brachiosaurus'; i++) await sleep(120);
  for (let i = 0; i < 60 && !pr.canBite(); i++) await sleep(120);
  const p = pr.state().player;
  pr.placeAt(p.x, p.z, 0);   // yaw 0 = facing +x, where the victim is placed
  pr.state().enemies.forEach(e => pr.placeEnemy(e.key, p.x + 60, p.z + 60, 0)); // clear strays behind us
  pr.placeEnemy('diplodocus', p.x + 6, p.z, Math.PI);
  pr.bite();
  let maxZ = 0, maxX = 0, sawStomp = 0;
  for (let i = 0; i < 14; i++) {
    await sleep(60);
    if (pr.moveOf('player').move !== 'stomp') continue;   // ignore walk/idle frames
    sawStomp++;
    const j = pr.jointRot('player', 'legFR');   // the rotation the game animates, before world-frame mixing
    if (!j) continue;
    maxZ = Math.max(maxZ, Math.abs(j[2])); maxX = Math.max(maxX, Math.abs(j[0]));
  }
  return { move: sawStomp ? 'stomp' : pr.moveOf('player').move, samples: sawStomp,
    maxZ: +maxZ.toFixed(2), maxX: +maxX.toFixed(2) };
});
ok(stomp.move === 'stomp' && stomp.samples >= 3 && stomp.maxZ > 0.25 && stomp.maxX < 0.08,
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
  let victim = (pr.state().enemies.find(e => e.alive && e.state !== 'die' && e.key !== 'mosasaurus') || {}).key;
  if (!victim) {                       // every challenger died of stray causes earlier: grow a fresh island
    pr.newExpedition(null, null, true);
    for (let i = 0; i < 40 && !victim; i++) {
      await sleep(150);
      victim = (pr.state().enemies.find(e => e.alive && e.state !== 'die' && e.key !== 'mosasaurus') || {}).key;
    }
  }
  const p2 = pr.state().player;
  pr.placeEnemy(victim, p2.x + 6, p2.z, Math.PI);
  pr.setRoar(1); pr.roar();
  let poisoned = 0, who = null;
  for (let i = 0; i < 30 && !poisoned; i++) {
    await sleep(150);
    pr.setRoar(1); pr.roar();   // keep spitting: the victim closes in, and the lead gets easier
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
  const curKey = pr.map().key;
  const wanted = { map: [...document.querySelectorAll('#mrow .mcard')].map(b => b.dataset.k).find(k => k !== curKey), sp: 'trike' };
  const label = document.getElementById('menuB').textContent;
  pr.stageMap(wanted.map); pr.setPick(wanted.sp);   // like an actual menu pick: stage, then one press applies
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

// ---------- plant-eaters graze peacefully until you hurt them ----------
const calm = await page.evaluate(async () => {
  const sleep = ms => new Promise(r => setTimeout(r, ms));
  const pr = window.__probe;
  pr.setCast(['stego', 'ankylo', 'diplodocus', 'brachiosaurus', 'trike']);   // all plant-eaters
  pr.newExpedition('forest', 777, true);
  for (let i = 0; i < 40 && !pr.ready; i++) await sleep(150);
  await sleep(1200);
  const p = pr.state().player;
  const vic = (pr.state().enemies.find(e => ['stego', 'ankylo'].includes(e.key) && e.alive) || {}).key;
  if (!vic) return { vic: null };
  pr.placeEnemy(vic, p.x + 7, p.z, Math.PI);
  pr.setEnemy(vic, { cool: 0, fear: 0 });
  let hostile = 0;
  for (let i = 0; i < 24; i++) {                                  // stand right in front of it
    await sleep(150);
    const e = pr.state().enemies.find(x => x.key === vic);
    if (!e || !e.alive) break;
    if (['chase', 'attack', 'flee'].includes(e.state)) hostile++;
    if (pr.moveOf(vic).angry) hostile++;
  }
  const heartsBefore = pr.state().hearts;
  pr.hurtEnemy(vic, 1);                                          // bite it: now it is MAD
  let angry = false, attacks = 0;
  for (let i = 0; i < 30; i++) {
    await sleep(150);
    const e = pr.state().enemies.find(x => x.key === vic);
    if (!e || !e.alive) break;
    if (pr.moveOf(vic).angry) angry = true;
    if (['chase', 'attack'].includes(e.state)) attacks++;
    if (angry && attacks >= 2) break;
  }
  return { vic, hostile, heartsBefore, heartsAfter: pr.state().hearts, angry, attacks };
});
ok(calm.vic && calm.hostile === 0 && calm.heartsBefore === calm.heartsAfter,
  'a plant-eater just grazes when you leave it alone', JSON.stringify(calm));
ok(calm.vic && calm.angry && calm.attacks >= 2,
  'but hurt it once and it fights back', JSON.stringify(calm));

// ---------- the Mosasaurus eats a swimming dinosaur whole ----------
const gulp = await page.evaluate(async () => {
  const sleep = ms => new Promise(r => setTimeout(r, ms));
  const pr = window.__probe;
  const vic = (pr.state().enemies.find(e => e.alive && e.key !== 'mosasaurus'
    && ['stego', 'diplodocus', 'ankylo', 'allo', 'spino', 'raptor', 'dilopho'].includes(e.key)) || {}).key;
  // a paddling-deep but not drowning patch of sea, with real deep water right next to it
  let spot = null, mos = null;
  for (let g = 0; g < 4000 && !spot; g++) {
    const a = Math.random() * 6.283, r = 95 + Math.random() * 38;
    const x = Math.cos(a) * r, z = Math.sin(a) * r;
    if (!(pr.ground(x, z) < -0.4 && pr.ground(x, z) > -1.2)) continue;
    const n = Math.hypot(x, z) || 1;
    const dirs = [[x / n, z / n], [-x / n, -z / n], [-z / n, x / n], [z / n, -x / n]];
    for (const [dx, dz] of dirs) for (let d = 5; d <= 12 && !spot; d += 1.5) {
      const ox = x + dx * d, oz = z + dz * d;
      if (pr.ground(ox, oz) < -1.0) {
        spot = [+x.toFixed(1), +z.toFixed(1)];
        mos = [+ox.toFixed(1), +oz.toFixed(1)];
        break;
      }
    }
  }
  if (!vic || !spot) return { vic, spot, eaten: false, noSpot: true };
  pr.setEnemy('mosasaurus', { x: mos[0], z: mos[1], cool: 0, fear: 0, windup: 0 });
  let eaten = false;
  for (let i = 0; i < 80 && !eaten; i++) {
    pr.placeEnemy(vic, spot[0], spot[1], Math.PI);               // keep it paddling there
    if (i % 8 === 7) pr.setEnemy('mosasaurus', { cool: 0, fear: 0 });
    await sleep(250);
    const st = pr.state();
    eaten = st.defeated.includes(vic) || st.enemies.some(e => e.key === vic && e.swallowed);
  }
  return { vic, spot, mos, eaten, def: pr.state().defeated };
});
ok(gulp.eaten, 'the Mosasaurus swallows a swimming dinosaur whole',
  `${gulp.vic}@${gulp.spot} defeated=${JSON.stringify(gulp.def)}`);

// ---------- hold-to-grab: clamp on and drag a smaller dinosaur ----------
const grab = await page.evaluate(async () => {
  const sleep = ms => new Promise(r => setTimeout(r, ms));
  const pr = window.__probe;
  pr.setGrab(false);
  const p0 = pr.state().player;
  const vic = (pr.state().enemies.find(e => e.alive && e.key !== 'mosasaurus' && e.key !== 'brachiosaurus') || {}).key;
  if (!vic) return { vic: null };
  pr.placeAt(p0.x, p0.z, 0);                                     // face down +x
  pr.placeEnemy(vic, p0.x + 2.6, p0.z, Math.PI);
  pr.setEnemy(vic, { cool: 0, fear: 0 });
  pr.setGrab(true);
  await sleep(250);
  const g1 = pr.grabInfo();
  for (let i = 0; i < 20; i++) {                                 // walk: each step drags it along
    pr.placeAt(p0.x + i * 0.45, p0.z, 0);
    await sleep(120);
  }
  await sleep(400);
  const g2 = pr.grabInfo(), v2 = pr.enemyInfo(vic);
  pr.setGrab(false);
  await sleep(300);
  const g3 = pr.grabInfo(), v3 = pr.enemyInfo(vic);
  return { vic, g1, g2, g3, v2, v3, p0 };
});
ok(grab.vic && grab.g1.held === grab.vic && grab.g1.d < 6,
  'you can clamp onto a smaller dinosaur by holding GRAB', JSON.stringify(grab.g1));
ok(grab.g2.held === grab.vic && grab.v2 && grab.v2.x > grab.p0.x + 3,
  'holding drags it along as you walk', `victim x=${grab.v2 && grab.v2.x} from x=${grab.p0.x}`);
ok(grab.g2 && grab.g1 && grab.g2.hp < grab.g1.hp,
  'holding on by the teeth hurts', `${grab.g1.hp} -> ${grab.g2.hp}`);
ok(grab.g3.held === null && grab.v3 && !grab.v3.grabbed,
  'letting go lets it go', JSON.stringify(grab.g3));
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
