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
ok(roster.list.length === 14 && roster.list.includes('mosasaurus') && roster.list.includes('hunter'), 'fourteen fighters to be — the Mosasaurus and the Human included', `${roster.list.length}`);
ok(roster.cast.length === 5 && !roster.cast.includes(roster.pick),
  'the island raises five challengers that are not you', roster.cast.join(','));
ok(roster.flags.dilopho.venom && roster.flags.brachiosaurus.huge && roster.flags.trike.charge,
  'the newcomers keep their tricks', 'venom + huge + charge');
ok(roster.flags.mosasaurus.sea && roster.flags.trike.herb && roster.flags.stego.herb,
  'the Mosasaurus owns the sea, the plant-eaters are peaceful by default', 'sea + herb');
ok(roster.flags.ankylo.giant && roster.flags.trex.giant && roster.flags.spino.giant && !roster.flags.raptor.giant,
  'every heavyweight — the armoured Ankylosaurus included — can carry a raptor', 'ankylo+trex+spino giant');

// ---------- the village: people live in every world ----------
const vill = await page.evaluate(() => {
  const pr = window.__probe, h = pr.humans(), s = pr.state();
  const hs = s.enemies.filter(e => e.human && e.alive);
  return { ...h, n: hs.length, dry: hs.every(e => pr.ground(e.x, e.z) > 0),
    home: hs.filter(e => Math.hypot(e.x - h.village[0], e.z - h.village[1]) < 90).length };
});
ok(vill.total >= 7 && vill.villagers >= 3 && vill.hunters >= 4 && vill.zone,
  'a human village stands in the world — villagers and spear-hunters both', JSON.stringify(vill));
ok(vill.n > 0 && vill.dry && vill.home >= Math.ceil(vill.n * 0.5),
  'the islanders keep to dry ground around their fire', `${vill.home}/${vill.n} near home`);
const huntZone = await page.evaluate(() =>
  window.__probe.map().zones.some(z => z.tag === 'village'));
ok(huntZone, 'the village is a landmark with a name of its own');

// ---------- a bigger, flatter island with a real mountain range ----------
const shape = await page.evaluate(() => {
  const pr = window.__probe, hs = [];
  let high = 0, maxH = 0;
  for (let i = 0; i < 400; i++) {
    const a = Math.random() * 6.283, r = Math.sqrt(Math.random()) * (pr.isle() - 20);
    const h = pr.ground(Math.cos(a) * r, Math.sin(a) * r);
    if (h > 0.5) { hs.push(h); if (h > 12) high++; if (h > maxH) maxH = h; }
  }
  hs.sort((a, b) => a - b);
  const N = 96, M = 48, g = [];
  for (let i = 0; i < N; i++) { g[i] = [];
    for (let j = 0; j < M; j++) {
      const a = i / N * 6.283, r = 30 + j / M * (pr.isle() - 60);
      g[i][j] = pr.ground(Math.cos(a) * r, Math.sin(a) * r);
    } }
  let peaks = 0;
  for (let i = 0; i < N; i++) for (let j = 2; j < M - 2; j++) {
    const h = g[i][j];
    if (h < 13) continue;
    let top = true;
    for (let di = -2; di <= 2 && top; di++) for (let dj = -2; dj <= 2; dj++)
      if (g[(i + di + N) % N][j + dj] > h) { top = false; break; }
    if (top) peaks++;
  }
  return { isle: pr.isle(), shelf: pr.shelf(), highPct: high / hs.length, peaks,
    maxH: +maxH.toFixed(1), p50: +hs[hs.length >> 1].toFixed(1), p95: +hs[Math.floor(hs.length * 0.95)].toFixed(1) };
});
ok(shape.isle >= 300 && shape.shelf >= 380, 'the continent has outgrown the old island', `isle ${shape.isle}, shelf ${shape.shelf}`);
ok(shape.p50 < 11 && shape.p95 < 34 && shape.maxH < 58,
  'the land lies mostly flat — no single sky-piercing mega-mountain', `p50 ${shape.p50}, p95 ${shape.p95}, max ${shape.maxH}`);
ok(shape.peaks >= 3, 'the mountains come in ridgelines of several peaks', `${shape.peaks} peaks`);

// ---------- the Human: GRAB tames a dinosaur, WASD then drives it ----------
await page.evaluate(() => window.__probe.switchTo('hunter'));
await wait(500);
const hunterPick = await page.evaluate(() => {
  const s = window.__probe.state();
  return { sp: s.species, joints: Object.keys(window.__probe.joints('player')).length };
});
ok(hunterPick.sp === 'hunter' && hunterPick.joints >= 5,
  'the Human is playable — a tiny biped with working limbs', JSON.stringify(hunterPick));
const tame = await page.evaluate(() => window.__probe.tameCheat());
ok(tame.ok, 'a Human mounts a wild dinosaur and breaks it on the spot', tame.host || tame.why);
const c0 = await page.evaluate(() => window.__probe.rideCtrl());
ok(c0 && c0.tamed && c0.controlled && c0.riders === 1,
  'the tamed giant takes your orders instead of bucking', JSON.stringify(c0));
await page.keyboard.down('KeyW');
await wait(1400);
await page.keyboard.up('KeyW');
const c1 = await page.evaluate(() => window.__probe.rideCtrl());
const gallop = c1 ? Math.hypot(c1.x - c0.x, c1.z - c0.z) : 0;
ok(gallop > 2.5, 'WASD rides the tamed mount across the island', `${gallop.toFixed(1)}m`);
await page.evaluate(() => window.__probe.bite());
const rideBite = await page.evaluate(() => window.__probe.state().player.state === 'attack');
ok(rideBite, 'BITE makes your mount strike where you point it');
await page.keyboard.down('KeyG');
await wait(200);
await page.keyboard.up('KeyG');
await wait(400);
const hopped = await page.evaluate((host) => {
  const s = window.__probe.state();
  const h = s.enemies.find(e => e.key === host && e.tamed);
  return { riding: window.__probe.rideCtrl(), tamed: !!h, controlled: h && h.controlled };
}, tame.host);
ok(!hopped.riding && hopped.tamed && !hopped.controlled,
  'GRAB again hops down — the tamed giant stays gentle and wanders off', JSON.stringify(hopped));

// ---------- spears out, jaws out: the village is on the menu both ways ----------
let spear = null, stabbed = null;
for (let t = 0; t < 3 && !stabbed; t++) {
  spear = await page.evaluate(() => window.__probe.spearTest());
  if (!spear.ok) break;
  await wait(6500);
  const s = await page.evaluate(() => window.__probe.spearHp());
  if (s && s.hp < spear.hp) stabbed = s;
}
ok(spear.ok && stabbed && stabbed.hp < spear.hp,
  'human hunters stab dinosaurs with their spears', `${spear.hp} → ${stabbed ? stabbed.hp : 'never touched'}`);
let chomp = null, bitten = null;
for (let t = 0; t < 3 && !bitten; t++) {
  chomp = await page.evaluate(() => window.__probe.chompTest());
  if (!chomp.ok) break;
  await wait(8000);
  const b = await page.evaluate(() => window.__probe.chompHp());
  if (b && (b.hp < chomp.hp || !b.alive)) bitten = b;
}
ok(chomp.ok && bitten && (bitten.hp < chomp.hp || !bitten.alive),
  'dinosaurs hunt and eat the islanders too',
  `${chomp.hp} → ${bitten ? bitten.hp : '?'}${bitten && !bitten.alive ? ' (eaten)' : ''}`);

// ---------- every world has people ----------
const worldPacks = await page.evaluate(async () => {
  const sleep = ms => new Promise(r => setTimeout(r, ms));
  const out = {};
  for (const k of ['canyon', 'beach']) {
    window.__probe.newExpedition(k, 1234 + k.length);
    await sleep(1800);
    out[k] = window.__probe.humans();
  }
  return out;
});
await wait(2000);
for (const k of ['canyon', 'beach']) {
  const w = worldPacks[k];
  ok(w && w.total >= 7 && w.zone,
    `the ${k} world has its own village of people`, JSON.stringify(w));
}
// back to the forest, back to the Dilophosaurus: the tests that follow expect the hero we started as
await page.evaluate(() => { window.__probe.setCast(null); window.__probe.newExpedition('forest', 777); });
await wait(2000);
await page.evaluate(() => window.__probe.switchTo('dilopho'));
await wait(500);

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
  pr.state().enemies.forEach(e => { if (!e.human) pr.placeEnemy(e.key, p.x + 70, p.z + 70, 0); });
  pr.setRoar(1); pr.roar();
  await sleep(200);
  const flight = { glob: pr.venomCount(), trail: pr.gooCount() };
  for (let i = 0; i < 30 && pr.venomCount(); i++) await sleep(120);
  const ended = pr.venomLast();
  let victim = (pr.state().enemies.find(e => e.alive && e.state !== 'die' && !e.human && e.key !== 'mosasaurus') || {}).key;
  if (!victim) {                       // every challenger died of stray causes earlier: grow a fresh island
    pr.newExpedition(null, null, true);
    for (let i = 0; i < 40 && !victim; i++) {
      await sleep(150);
      victim = (pr.state().enemies.find(e => e.alive && e.state !== 'die' && e.key !== 'mosasaurus') || {}).key;
    }
  }
  const p2 = pr.state().player;
  pr.clearBaits();   // a beach snack would lure the victim away mid-spit
  pr.placeEnemy(victim, p2.x + 6, p2.z, Math.PI);
  pr.setRoar(1); pr.roar();
  let poisoned = 0, who = null;
  for (let i = 0; i < 30 && !poisoned; i++) {
    await sleep(150);
    const now = pr.state().player, vic = pr.state().enemies.find(e => e.key === victim && e.alive);
    if (vic && Math.hypot(vic.x - now.x, vic.z - now.z) > 14) pr.placeEnemy(victim, now.x + 6, now.z, Math.PI);
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
  const vic = (pr.state().enemies.find(e => e.alive && !e.counted && e.key !== 'mosasaurus'
    && ['stego', 'diplodocus', 'ankylo', 'allo', 'spino', 'raptor', 'dilopho'].includes(e.key)) || {}).key;
  // a paddling-deep but not drowning patch of sea, with real deep water right next to it
  let spot = null, mos = null;
  for (let g = 0; g < 4000 && !spot; g++) {
    const a = Math.random() * 6.283, r = 330 + Math.random() * 80;   // the wide sea shelf
    const x = Math.cos(a) * r, z = Math.sin(a) * r;
    if (!(pr.ground(x, z) < -0.4 && pr.ground(x, z) > -1.2)) continue;
    const n = Math.hypot(x, z) || 1;
    const dirs = [[x / n, z / n], [-x / n, -z / n], [-z / n, x / n], [z / n, -x / n]];
    for (const [dx, dz] of dirs) for (let d = 4; d <= 18 && !spot; d += 1.5) {
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

// ---------- raptor winter: a world where nothing hunts but raptors ----------
await page.evaluate(() => { window.__probe.setCast(null); window.__probe.newExpedition('winter', 4242); });
await wait(2000);
const winter = await page.evaluate(() => {
  const pr = window.__probe, s = pr.state();
  const beasts = s.enemies.filter(e => e.alive && !e.human);
  return { m: pr.map(), keys: [...new Set(beasts.map(e => e.key))],
    sea: beasts.some(e => e.key === 'mosasaurus'), hum: pr.humans() };
});
ok(winter.m.winter && winter.m.noSea, 'the Raptor Winter loads: frozen seas, no open water', JSON.stringify(winter.m));
ok(winter.keys.length === 1 && winter.keys[0] === 'raptor' && !winter.sea,
  'in Raptor Winter nothing hunts but raptors', JSON.stringify(winter.keys));
ok(winter.hum.total >= 6 && winter.hum.zone, 'even the frozen world has a human village', JSON.stringify(winter.hum));
const wKill = await page.evaluate(() => {
  const pr = window.__probe; let n = 0;
  while (pr.kill('raptor') && n < 30) n++;
  return { n, left: pr.state().goalLeft };
});
await wait(2800);
const wonNow = await page.evaluate(() => window.__probe.state().won);
ok(wKill.n >= 5 && wKill.left === 0, `the whole raptor pack falls (${wKill.n} down)`, `${wKill.left} to go`);
ok(wonNow, 'beat every raptor and the frozen world is yours');
await page.evaluate(() => window.__probe.closeWin && window.__probe.closeWin());
await page.close();

// ---------- raptors ride the giants: leap on, peck, and get shaken off ----------
const ridePage = await open({ viewport: { width: 1280, height: 800 } });
await wait(1500);
await ridePage.evaluate(() => { window.__probe.setPick('raptor'); });
await ridePage.click('#playB');
await ridePage.waitForFunction(() => window.__probe && window.__probe.ready, null, { timeout: 60000 });
await wait(1500);
const ride = await ridePage.evaluate(() => {
  const pr = window.__probe;
  pr.spawnDino('trike');
  const m = pr.mount();
  return { m, after: pr.rideInfo(), flags: pr.speciesFlags() };
});
ok(ride.flags.trike.giant && ride.flags.raptor.rider, 'raptors can ride and giants can be ridden',
  JSON.stringify([ride.flags.trike.giant, ride.flags.raptor.rider]));
ok(ride.m.ok && ride.after.riding === ride.m.host, 'a raptor can leap onto a giant\'s back',
  JSON.stringify(ride.m) + ' → riding:' + ride.after.riding);
// saddle check: while the giant bucks, the raptor stays planted on the back —
// no rubber-banding off the tail, no hovering above the spine
const saddle = await ridePage.evaluate(async () => {
  const sleep = ms => new Promise(r => setTimeout(r, ms));
  let latMax = 0, fwdMax = 0, dUpMax = 0, sad = 0, samples = 0;
  for (let i = 0; i < 26; i++) {
    await sleep(120);
    const g = window.__probe.rideGeom();
    if (!g) break;
    const dx = g.rider.x - g.host.x, dz = g.rider.z - g.host.z, y = g.host.yaw;
    const fwd = Math.abs(dx * Math.cos(y) - dz * Math.sin(y)), lat = Math.abs(dx * Math.sin(y) + dz * Math.cos(y));
    sad = g.host.sad;
    dUpMax = Math.max(dUpMax, Math.abs(g.rider.y - g.host.y - g.host.sad));
    latMax = Math.max(latMax, lat); fwdMax = Math.max(fwdMax, fwd); samples++;
  }
  return { latMax: +latMax.toFixed(2), fwdMax: +fwdMax.toFixed(2), dUpMax: +dUpMax.toFixed(2), sad, samples };
});
ok(saddle.samples > 15 && saddle.latMax < 0.8 && saddle.fwdMax < 1.5 && saddle.dUpMax < 0.9,
  'the rider stays saddled while the giant bucks', JSON.stringify(saddle));
ok(ride.after.host && ride.after.host.state === 'shake' && ride.after.host.riders.includes('you'),
  'the bucking giant thunders around trying to shake you off', JSON.stringify(ride.after.host));
const ride2 = await ridePage.evaluate(() => {
  const pr = window.__probe;
  const before = pr.rideInfo().host.hp;
  pr.bite();
  const r = pr.rideInfo();
  return { before, after: r.host.hp, riding: r.riding };
});
ok(ride2.after < ride2.before && ride2.riding, 'pecking the giant from up top hurts it',
  `${ride2.before} → ${ride2.after}`);
let off = await ridePage.evaluate(() => window.__probe.rideInfo().riding);
for (let t = 0; t < 3 && off !== null; t++) {   // swiftshader frames can swallow a short grab edge
  await ridePage.evaluate(() => window.__probe.setGrab(true));
  await wait(550);
  await ridePage.evaluate(() => window.__probe.setGrab(false));
  await wait(400);
  off = await ridePage.evaluate(() => window.__probe.rideInfo().riding);
}
ok(off === null, 'GRAB again hops you down off the giant', String(off));
// the armoured tank is saddleable too — nearest giant logic + armour must not block it
const anky = await ridePage.evaluate(() => {
  const pr = window.__probe;
  const a = pr.spawnDino('ankylo');
  pr.teleport(a.x + 2, a.z);                       // stand right beside it
  return { m: pr.mount(), hp: pr.rideInfo().host && pr.rideInfo().host.hp };
});
ok(anky.m.ok && anky.m.host === 'ankylo', 'a raptor can even saddle an Ankylosaurus', JSON.stringify(anky.m));
const anky2 = await ridePage.evaluate(async (hp0) => {   // peck until one lands; attack windows come round
  const sleep = ms => new Promise(r => setTimeout(r, ms));
  const pr = window.__probe;
  let r = null;
  for (let i = 0; i < 12; i++) {
    pr.bite();
    await sleep(400);
    r = pr.rideInfo();
    if (r.host && r.host.hp < hp0) return r;
  }
  return r;
}, anky.hp);
ok(anky2.host && anky2.host.hp < anky.hp && anky2.host.riders.includes('you'),
  'and pecks land despite all that armour', `${anky.hp} → ${anky2.host && anky2.host.hp}`);

// ---------- the Mosasaurus: selectable, and gloriously at home in the deep ----------
await ridePage.evaluate(() => window.__probe.newExpedition('beach', 999));
await wait(2200);
const deep = await ridePage.evaluate(() => {
  const pr = window.__probe, W = pr.water;
  for (let g = 0; g < 6000; g++) {
    const a = Math.random() * 6.283, r = 330 + Math.random() * 80;   // the wide sea shelf
    const x = Math.cos(a) * r, z = Math.sin(a) * r;
    if (pr.ground(x, z) < W - 3) return { x, z };
  }
  return null;
});
ok(deep, 'the beach island has a proper deep ocean to rule', deep ? `deep at ${Math.round(deep.x)},${Math.round(deep.z)}` : 'none found');
const mm = await ridePage.evaluate((d) => {
  const pr = window.__probe;
  pr.switchTo('mosasaurus');
  pr.teleport(d.x, d.z);
  return { sp: pr.state().species };
}, deep);
await wait(700);
const float = await ridePage.evaluate(() => {
  const s = window.__probe.state();
  return { y: s.player.y, W: window.__probe.water, state: s.player.state };
});
ok(mm.sp === 'mosasaurus' && float.y > float.W - 2.2 && float.y < float.W + 0.6,
  'a playable Mosasaurus floats in the deep sea', `y=${float.y} vs water=${float.W}`);
const gulp2 = await ridePage.evaluate(() => {
  const pr = window.__probe, s = pr.state();
  const vic = s.enemies.find(e => e.alive && !e.swallowed && ['raptor', 'dilopho', 'allo', 'spino'].includes(e.key));
  if (!vic) return null;
  pr.placeEnemy(vic.key, s.player.x + 1.6, s.player.z, 0);
  pr.face(s.player.x + 4, s.player.z);
  pr.bite();
  const after = pr.state().enemies.find(e => e.key === vic.key);
  return { vic: vic.key, swallowed: after ? after.swallowed : 'gone' };
});
ok(gulp2 && gulp2.swallowed, 'and it swallows small dinosaurs whole, as is its right', JSON.stringify(gulp2));
await ridePage.close();

// ---------- split-screen: up to four heroes share one island ----------
const mp = await open({ viewport: { width: 1280, height: 800 } });
await wait(1800);
await mp.evaluate(() => { window.__probe.setPlayers(2); window.__probe.setPick('raptor'); });
await mp.click('#playB');
await mp.waitForFunction(() => window.__probe && window.__probe.ready, null, { timeout: 60000 });
await wait(1500);
const two = await mp.evaluate(() => {
  const pr = window.__probe;
  return { n: pr.pcount(), pl: pr.playersInfo(), panes: document.querySelectorAll('.vp').length,
    heartsHidden: getComputedStyle(document.getElementById('hearts')).display };
});
ok(two.n === 2 && two.panes === 2, 'two players, two panes, one island', `${two.n} heroes × ${two.panes} panes`);
ok(two.pl[0].vp.w === 640 && two.pl[1].vp.x === 640 && two.pl[0].key !== two.pl[1].key,
  'the screen splits in half and no two heroes share a species',
  JSON.stringify(two.pl.map(p => `${p.key}@x${p.vp.x}`)));
ok(two.heartsHidden === 'none' && two.pl.every(p => p.hearts === 5),
  'each hero keeps their own hearts and little HUD', `display:${two.heartsHidden} hearts:${two.pl.map(p => p.hearts)}`);
const p2a0 = await mp.evaluate(() => window.__probe.playersInfo()[1]);
// the new continent may park P2 with a hill, the sea or a bad nap in the way: try each way, wake, retry
let p2b = p2a0, moved2 = 0;
for (let round = 0; round < 2 && moved2 <= 1.5; round++) {
  for (const key of ['KeyI', 'KeyJ', 'KeyL', 'KeyK']) {
    await mp.keyboard.down(key); await wait(900); await mp.keyboard.up(key);
    const now = await mp.evaluate(() => window.__probe.playersInfo()[1]);
    moved2 = Math.hypot(now.x - p2a0.x, now.z - p2a0.z);
    if (moved2 > 1.5) { p2b = now; break; }
    p2b = now;
  }
  if (moved2 <= 1.5) await mp.evaluate(() => window.__probe.wakeP(1));   // in case it naps
}
ok(moved2 > 1.5, 'P2 roams the island with IJKL',
  `moved ${moved2.toFixed(1)}${p2b.ko ? ', napping' : ''}`);
const p1a = await mp.evaluate(() => window.__probe.playersInfo()[0]);
await mp.keyboard.down('KeyW'); await wait(900); await mp.keyboard.up('KeyW');
const p1b = await mp.evaluate(() => window.__probe.playersInfo()[0]);
ok(Math.hypot(p1b.x - p1a.x, p1b.z - p1a.z) > 1.5, 'while P1 still roams with WASD',
  `moved ${Math.hypot(p1b.x - p1a.x, p1b.z - p1a.z).toFixed(1)}`);
await mp.close();
const mp4 = await open({ viewport: { width: 1280, height: 800 } });
await wait(1800);
await mp4.evaluate(() => window.__probe.setPlayers(4));
await mp4.click('#playB');
await mp4.waitForFunction(() => window.__probe && window.__probe.ready, null, { timeout: 60000 });
await wait(1200);
const four = await mp4.evaluate(() => {
  const pr = window.__probe, pl = pr.playersInfo();
  return { n: pr.pcount(), panes: document.querySelectorAll('.vp').length,
    corners: [...new Set(pl.map(p => p.vp.x + ',' + p.vp.y))].length, w: pl[0].vp.w,
    kinds: new Set(pl.map(p => p.key)).size };
});
ok(four.n === 4 && four.panes === 4 && four.corners === 4 && four.w <= 640 && four.kinds >= 3,
  'all four heroes get their own corner of the island',
  `${four.n} heroes, ${four.corners} corners, ${four.kinds} species`);
await mp4.close();

// ---------- bait: the Mosasaurus lures the whole island to the beach ----------
const baitp = await open({ viewport: { width: 1280, height: 800 } });
await wait(1800);
await baitp.evaluate(() => { window.__probe.setPick('raptor'); });
await baitp.click('#playB');
await baitp.waitForFunction(() => window.__probe && window.__probe.ready, null, { timeout: 60000 });
await baitp.evaluate(() => window.__probe.newExpedition('beach', 31337));
await wait(1500);
const drop = await baitp.evaluate(async () => {
  const sleep = ms => new Promise(r => setTimeout(r, ms));
  const pr = window.__probe;
  const s = pr.mosaShore();
  for (let i = 0; i < 20 && !pr.baits().length; i++) await sleep(500);
  return { s, b: pr.baits()[0] || null };
});
ok(drop.s.ok && drop.b && drop.b.hp === 7 &&
  Math.hypot(drop.b.x - drop.s.shore[0], drop.b.z - drop.s.shore[1]) < 3,
  'the Mosasaurus hauls a fat fish up onto the beach as a gift for everyone', JSON.stringify(drop));
const lure = await baitp.evaluate(async () => {
  const sleep = ms => new Promise(r => setTimeout(r, ms));
  const pr = window.__probe;
  await sleep(400);                                 // let last run's fish settle
  const b = pr.baits()[0];
  if (!b) return { fail: 'bait vanished' };
  pr.exileNear(b.x, b.z, 130);               // one animal's snack, no ambient free-for-all
  pr.setInvuln(150);                         // a hero nap would freeze the whole island
  // pick a challenger kind that exists exactly once, plant it 50–80 m off across dry land
  const cnt = {};
  for (const e of pr.state().enemies)
    if (e.alive && !e.counted && e.key !== 'mosasaurus') cnt[e.key] = (cnt[e.key] || 0) + 1;
  const key = Object.keys(cnt).find(k => cnt[k] === 1);
  if (!key) return { fail: 'no lone challenger' };
  // scan all directions for one with unbroken dry land between animal and fish
  let px = null, pz = null;
  for (let k = 0; k < 16 && px === null; k++) {
    const ang = k * Math.PI / 8;
    for (let d = 80; d >= 50 && px === null; d -= 5) {
      let land = true;
      for (let m = 20; m <= d; m += 10) {
        const tx = b.x + Math.cos(ang) * m, tz = b.z + Math.sin(ang) * m;
        if (pr.ground(tx, tz) < 0.4) { land = false; break; }
      }
      if (land) { px = b.x + Math.cos(ang) * d; pz = b.z + Math.sin(ang) * d; }
    }
  }
  if (px === null) return { fail: 'no land that far out' };
  pr.placeEnemy(key, px, pz, 0);
  const d0 = Math.hypot(px - b.x, pz - b.z);
  let dmin = d0, eaten = false, bites = 0;
  for (let i = 0; i < 55 && !eaten; i++) {
    pr.exileNear(b.x, b.z, 25, key);          // keep late arrivals off the fish, every second
    await sleep(1000);
    const s = pr.state();
    if (s.down) pr.wakeP();                   // hero napping stops the whole island
    const e = s.enemies.find(x => x.key === key && x.alive);
    if (e) { dmin = Math.min(dmin, Math.hypot(e.x - b.x, e.z - b.z)); bites = Math.max(bites, e.bites); }
    const gone = !pr.baits().some(bb => Math.hypot(bb.x - b.x, bb.z - b.z) < 1);
    if (bites >= 1 || (gone && dmin < 8)) eaten = true;   // its OWN chomp — the fish itself can't tell them apart
  }
  return { key, d0: +d0.toFixed(0), dmin: +dmin.toFixed(1), bites, eaten };
});
ok(!lure.fail && (lure.eaten || lure.dmin < lure.d0 * 0.5),
  'a dinosaur smells that fish from a huge way off and trots the whole way in', JSON.stringify(lure));
ok(lure.eaten, 'and when it gets there, it eats the Mosasaurus\'s bait', JSON.stringify(lure));
await baitp.close();

// ---------- phone: the twelve-card strip fits and the game plays ----------
const phone = await open({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true });
await wait(2000);
const strip = await phone.evaluate(() => {
  const el = document.getElementById('pick'), cards = [...el.querySelectorAll('.pcard')];
  const r = el.getBoundingClientRect();
  return { n: cards.length, fits: cards.every(c => c.getBoundingClientRect().right <= r.right + 1),
    w: Math.round(cards[0].getBoundingClientRect().width), overflow: el.scrollWidth > el.clientWidth };
});
ok(strip.n === 14 && strip.w >= 100, 'fourteen picture cards on a phone, big enough to tap', `${strip.n} × ${strip.w}px`);
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
