/* build_dinos.mjs — species definitions: skeletons, silhouettes and scale hide.
   Every mesh is authored in ROOT space (+X forward, feet at y=0, metres); joints are
   nested with parent-relative offsets and the GLB writer counter-translates each mesh,
   so a joint rotation swings the parts that hang off it. Joint names are what the game's
   poseChar animates: thighL/R shinL/R footL/R armL/R neck head jaw tail1..n (theropod),
   legFL/FR/BL/BR neck head tail1..n (quadruped), plus extras the game drives (frill, spit).
   Run: node tools/build_dinos.mjs [key...]      (default: all)                   */
import { writeFileSync } from 'fs';
import path from 'path';
import { tube, cone, ball, fin, plateRow, spikeLine, merge, empty, shade, rgb, hide,
  buildGLB, renderCard, bellyAt, checkLimbs } from './dino_factory.mjs';

const J = (name, t, parent) => ({ name, t, parent });
const M = (name, parent, part) => ({ name, parent, part });
const rel = (a, b) => [a[0] - b[0], a[1] - b[1], 0];
const p3 = p => [p[0], p[1], p[2] ?? 0];                // specs may be 2D (side view)
const mid = (a, b, lift = 0) => [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2 + lift, (a[2] ?? 0) / 2 + (b[2] ?? 0) / 2];
/* a pale eye bead — these models read best with a visible eye */
const whiteEye = (c, r = 0.05) => shade(ball(c, r, { rings: 6, segs: 3 }),
  hide(rgb('#f4ecd6'), rgb('#3a3527'), rgb('#ffffff'), null, { speck: 0.05 }));

/* a neck or tail: joints at control points, one root-space mesh per link */
function chain(name, pts, radius, paint, opts = {}) {
  const joints = [], meshes = [], n = pts.length - 1;
  for (let i = 0; i < n; i++) {
    const nm = name + (i + 1), a = pts[i], b = pts[i + 1], f = i / n, f2 = (i + 1) / n;
    joints.push(J(nm, i === 0 ? a : rel(pts[i], pts[i - 1]), i === 0 ? (opts.parent || null) : name + i));
    const r0 = radius[0] + (radius[1] - radius[0]) * f, r1 = radius[0] + (radius[1] - radius[0]) * f2;
    meshes.push(M('m' + nm, nm, shade(tube([a, mid(a, b, (opts.lift || 0) * (1 - f)), b],
      t => [r0 + (r1 - r0) * t, (r0 + (r1 - r0) * t) * (opts.flat || 1)], { rings: opts.rings || 6, segs: 4 }), paint)));
  }
  const ren = m => ({ ...m, name: m.name.replace('m' + name, name + 'm') });
  return { joints, meshes: meshes.map(ren) };
}

/* ------------------------- theropod template ---------------------------
   o = { s: size, leg, body, neckR, head, snout, tail, tailR, arms, paint,
         nail, extras(J/M pushed after the base rig) }                        */
function theropod(o) {
  const P = o.paint, NL = o.nail;
  const joints = [], meshes = [];
  const hip = o.hip, knee = o.knee, ank = o.ank, lz = o.legZ;
  for (const side of ['L', 'R']) {
    const z = side === 'L' ? -lz : lz;
    joints.push(J('thigh' + side, [hip[0], hip[1], z]));
    joints.push(J('shin' + side, rel(knee, hip), 'thigh' + side));
    joints.push(J('foot' + side, rel(ank, knee), 'shin' + side));
    meshes.push(M('thighm' + side, 'thigh' + side, shade(tube([[hip[0] + 0.16, hip[1] + 0.1, z],
      [hip[0] - 0.02, (hip[1] + knee[1]) / 2, z], [knee[0] + 0.04, knee[1] + 0.04, z]],
      f => [o.legR * (1 - 0.45 * f), o.legR * 1.15 * (1 - 0.45 * f)], { rings: 6, segs: 4 }), P)));
    meshes.push(M('shinm' + side, 'shin' + side, shade(tube([[knee[0] + 0.03, knee[1], z],
      [ank[0] + 0.03, (knee[1] + ank[1]) / 2, z], [ank[0], ank[1], z]],
      f => [o.legR * 0.62 * (1 - 0.4 * f), o.legR * 0.66 * (1 - 0.4 * f)], { rings: 6, segs: 4 }), P)));
    meshes.push(M('footm' + side, 'foot' + side, shade(merge(
      tube([[ank[0] - ank[1] * 0.5, ank[1] * 0.9, z], [ank[0] + ank[1] * 1.5, ank[1] * 0.75, z]],
        f => [o.legR * 0.5 * (1 - 0.25 * f), o.legR * 0.36], { rings: 5, segs: 3 }),
      tube([[ank[0] - ank[1] * 1.6, ank[1], z], [ank[0] - ank[1] * 0.4, ank[1] * 0.9, z]],
        () => [o.legR * 0.34, o.legR * 0.28], { rings: 5, segs: 2 })), P)));
    if (o.claw !== false) meshes.push(M('clawm' + side, 'foot' + side, shade(spikeLine([
      [ank[0] + ank[1] * 0.9, ank[1] * 0.75, z + 0.02], [ank[0] + ank[1] * 1.9, ank[1] * 0.1, z + 0.06]],
      o.legR * 0.3, 0.004), NL)));
    if (o.arms) {
      joints.push(J('arm' + side, [o.shoulder[0], o.shoulder[1], z * 0.85]));
      meshes.push(M('armm' + side, 'arm' + side, shade(tube([[o.shoulder[0] + 0.02, o.shoulder[1] - 0.02, z * 0.85],
        [o.shoulder[0] + o.armLen * 0.5, o.shoulder[1] - o.armLen * 0.6, z * 0.85],
        [o.shoulder[0] + o.armLen, o.shoulder[1] - o.armLen * 0.95, z * 0.85]],
        f => [o.armR * (1 - 0.45 * f), o.armR * (1 - 0.45 * f)], { rings: 5, segs: 3 }), P)));
      meshes.push(M('armclawm' + side, 'arm' + side, shade(spikeLine([[o.shoulder[0] + o.armLen, o.shoulder[1] - o.armLen * 0.95, z * 0.85],
        [o.shoulder[0] + o.armLen * 1.5, o.shoulder[1] - o.armLen * 1.5, z * 0.85]], o.armR * 0.55, 0.003), NL)));
    }
  }
  meshes.push(M('bodym', null, shade(tube(o.body.map(p => [p[0], p[1], 0]),
    f => [o.bodyW[0] + (o.bodyW[1] - o.bodyW[0]) * Math.sin(Math.min(1, f * 1.15) * Math.PI * 0.5),
    o.bodyH[0] + (o.bodyH[1] - o.bodyH[0]) * Math.sin(Math.min(1, f * 1.1) * Math.PI * 0.5)],
    { rings: 9, segs: 10 }), P)));
  // neck + head sit forward of the shoulders
  const sh = o.shoulder, nk = o.neckTip, hd = o.head, hj = o.headJoint;
  joints.push(J('neck', [sh[0], sh[1], 0]));
  meshes.push(M('neckm', 'neck', shade(tube([[sh[0] - 0.1, sh[1] + 0.02, 0], mid([sh[0], sh[1], 0], nk, 0.04), nk],
    f => [o.neckR[0] + (o.neckR[1] - o.neckR[0]) * f, o.neckR[0] + (o.neckR[1] - o.neckR[0]) * f],
    { rings: 7, segs: 4 }), P)));
  joints.push(J('head', rel(hj, sh), 'neck'));
  meshes.push(M('headm', 'head', shade(merge(
    ball([hd[0], hd[1], 0], o.headR, { sx: 1.35, sy: 0.95, sz: 0.9, rings: 8, segs: 5 }),
    cone([hd[0] + o.headR * 0.3, hd[1] - o.headR * 0.2, 0], [hd[0] + o.snout, hd[1] - o.headR * 0.45, 0],
      o.headR * 0.66, o.headR * 0.26, { rings: 7 })), P)));
  meshes.push(M('eyesm', 'head', merge(whiteEye([hd[0] + o.headR * 0.35, hd[1] + o.headR * 0.42, -o.headR * 0.72], o.headR * 0.2),
    whiteEye([hd[0] + o.headR * 0.35, hd[1] + o.headR * 0.42, o.headR * 0.72], o.headR * 0.2))));
  const jh = [hd[0] - o.headR * 0.45, hd[1] - o.headR * 0.55];      // jaw hinge
  joints.push(J('jaw', rel(jh, hj), 'head'));
  meshes.push(M('jawm', 'jaw', shade(merge(
    cone([hd[0] - o.headR * 0.1, hd[1] - o.headR * 0.72, 0], [hd[0] + o.snout * 0.9, hd[1] - o.headR * 0.8, 0],
      o.headR * 0.5, o.headR * 0.18, { rings: 6 }),
    ball([hd[0] - o.headR * 0.15, hd[1] - o.headR * 0.68, 0], o.headR * 0.62, { sx: 0.95, sy: 0.55, sz: 0.9, rings: 6, segs: 3 })), P)));
  if (o.spit) joints.push(J('spit', rel([hd[0] + o.snout * 0.95, hd[1] - o.headR * 0.45], hj), 'head'));
  const tc = chain('tail', o.tail, o.tailR, P, { lift: 0.03, rings: 7 });
  joints.push(...tc.joints); meshes.push(...tc.meshes);
  return { joints, meshes };
}

/* ======================= Velociraptor (2.1 m, fast) ==================== */
function raptor() {
  const paint = hide(rgb('#9a7c46'), rgb('#3b2f1d'), rgb('#dac9a2'), rgb('#4d3f24'),
    { bellyY: 0.55, grad: 0.3, speck: 0.15, stripes: '#463518', stripeFreq: 2.0 });
  const nail = hide(rgb('#eae2ca'), rgb('#8d8467'), rgb('#fff8e6'), null, { speck: 0.05 });
  const m = theropod({
    paint, nail,
    hip: [0.05, 1.2], knee: [0.2, 0.7], ank: [0.06, 0.26], legZ: 0.17, legR: 0.15,
    shoulder: [0.85, 1.5], armLen: 0.26, armR: 0.05, arms: true,
    body: [[-1.0, 1.45], [-0.3, 1.55], [0.4, 1.55], [0.85, 1.5]], bodyW: [0.24, 0.3], bodyH: [0.3, 0.4],
    neckR: [0.16, 0.13], neckTip: [1.32, 1.72], head: [1.58, 1.8], headR: 0.21, snout: 0.55, headJoint: [1.42, 1.76],
    tail: [[-1.0, 1.42], [-1.8, 1.42], [-2.5, 1.34], [-3.1, 1.2], [-3.6, 1.02]], tailR: [0.2, 0.04],
  });
  // a feather crest down the neck and arms
  m.meshes.push(M('crestm', 'head', shade(fin([[1.3, 1.95], [1.05, 2.02], [0.78, 1.9], [1.02, 1.86]], 0.04,
    { axis: 'xy', off: 0 }), paint)));
  return m;
}

/* =========== Dilophosaurus (4 m, crested, frilled, venom spit) ========== */
function dilopho() {
  const paint = hide(rgb('#2f2e33'), rgb('#141416'), rgb('#efe9dc'), rgb('#5b6070'),
    { bellyY: 1.1, grad: 0.3, speck: 0.1, stripes: '#f1ece1', stripeFreq: 1.3 });
  const frillC = hide(rgb('#d8542a'), rgb('#7c2b13'), rgb('#f6c65c'), rgb('#f6e08a'), { bellyY: 1.6, grad: 0.16, speck: 0.12 });
  const nail = hide(rgb('#2a2a2c'), rgb('#0d0d0f'), rgb('#6a6a70'), null, { speck: 0.06 });
  const m = theropod({
    paint, nail, spit: true,
    hip: [0.1, 1.7], knee: [0.3, 1.0], ank: [0.1, 0.36], legZ: 0.24, legR: 0.2,
    shoulder: [1.15, 2.1], armLen: 0.34, armR: 0.065, arms: true,
    body: [[-1.35, 2.05], [-0.4, 2.2], [0.55, 2.2], [1.15, 2.1]], bodyW: [0.3, 0.4], bodyH: [0.38, 0.52],
    neckR: [0.22, 0.18], neckTip: [1.78, 2.42], head: [2.1, 2.52], headR: 0.27, snout: 0.72, headJoint: [1.9, 2.46],
    tail: [[-1.35, 2.02], [-2.3, 2.0], [-3.15, 1.9], [-3.9, 1.72], [-4.5, 1.5]], tailR: [0.27, 0.05],
  });
  // the display frill hangs off its own joint so the game can flare it
  m.joints.push(J('frill', rel([1.45, 2.05], [1.15, 2.1]), 'neck'));
  let F = empty();
  for (const z of [-0.19, 0.19]) {
    const prof = [];
    for (let i = 0; i <= 12; i++) {
      const a = -1.45 + i / 12 * 2.9;
      prof.push([1.42 + Math.sin(a) * 0.5, 2.0 + Math.cos(a) * 0.52]);
    }
    F = merge(F, fin(prof, 0.03, { axis: 'xy', off: z }));
  }
  m.meshes.push(M('frillm', 'frill', shade(F, frillC)));
  let C = empty();
  for (const z of [-0.08, 0.08])
    C = merge(C, fin([[1.82, 2.76], [2.3, 2.86], [2.62, 3.06], [2.1, 2.84]], 0.04, { axis: 'xy', off: z }));
  m.meshes.push(M('crestm', 'head', C));
  return m;
}

/* ===================== Triceratops (5.8 m, charges) ==================== */
function trike() {
  const paint = hide(rgb('#8a7a58'), rgb('#453a28'), rgb('#d8cdae'), rgb('#6a5d42'),
    { bellyY: 1.05, grad: 0.2, speck: 0.16, spots: '#5c4f37' });
  const frillC = hide(rgb('#9a6a4a'), rgb('#4b2f20'), rgb('#e0a96a'), rgb('#c8623a'), { bellyY: 1.5, grad: 0.14, speck: 0.12 });
  const hornC = hide(rgb('#efe6cd'), rgb('#8f8364'), rgb('#fffaf0'), null, { speck: 0.05 });
  const joints = [], meshes = [];
  for (const [n, x, top, r0, r1] of [['FL', 1.25, 1.5, 0.34, 0.22], ['FR', 1.25, 1.5, 0.34, 0.22],
    ['BL', -1.25, 1.4, 0.32, 0.21], ['BR', -1.25, 1.4, 0.32, 0.21]]) {
    const z = (n[1] === 'L' ? -1 : 1) * (n[0] === 'F' ? 0.66 : 0.6);
    joints.push(J('leg' + n, [x, top, z]));
    meshes.push(M('legm' + n, 'leg' + n, shade(tube([[x, top + 0.18, z], [x, top * 0.5, z], [x, 0.16, z]],
      f => [r0 - (r0 - r1) * f, r0 - (r0 - r1) * f], { rings: 7, segs: 4 }), paint)));
    meshes.push(M('footm' + n, 'leg' + n, shade(ball([x, 0.2, z], r1 * 1.15, { sx: 1.15, sy: 0.62, sz: 1.05, rings: 6, segs: 3 }), paint)));
  }
  meshes.push(M('bodym', null, shade(tube([[-2.1, 1.5, 0], [-1.0, 1.8, 0], [0.35, 1.95, 0], [1.4, 1.85, 0]],
    f => { const d = Math.abs(f - 0.45); return [0.78 - 0.4 * d, 0.86 - 0.42 * d]; }, { rings: 9, segs: 10 }), paint)));
  joints.push(J('neck', [1.4, 1.8, 0]));
  meshes.push(M('neckm', 'neck', shade(tube([[1.3, 1.8, 0], [1.95, 1.9, 0]], f => [0.52 - 0.16 * f, 0.5 - 0.14 * f],
    { rings: 8, segs: 3 }), paint)));
  joints.push(J('head', [0.85, 0.16, 0], 'neck'));
  meshes.push(M('headm', 'head', shade(merge(
    ball([2.35, 2.0, 0], 0.44, { sx: 1.05, sy: 0.95, sz: 1.2, rings: 8, segs: 5 }),
    cone([2.5, 1.9, 0], [3.2, 1.72, 0], 0.32, 0.16, { rings: 7 })), paint)));
  meshes.push(M('eyesm', 'head', merge(whiteEye([2.6, 2.18, -0.36], 0.08), whiteEye([2.6, 2.18, 0.36], 0.08))));
  // the frill: a big standing fan behind the skull, in the XY plane
  {
    const prof = [];
    for (let i = 0; i <= 16; i++) {
      const a = -1.45 + i / 16 * 2.9;
      prof.push([1.95 - Math.cos(a) * 1.0, 2.1 + Math.sin(a) * 1.1]);
    }
    meshes.push(M('frillm', 'head', shade(fin(prof, 0.16, { axis: 'xy', off: 0 }), frillC)));
  }
  {
    let Hn = empty();
    for (const z of [-0.36, 0.36])
      Hn = merge(Hn, spikeLine([[2.62, 2.28, z], [3.2, 2.95, z * 1.5], [3.42, 3.25, z * 1.7]], 0.14, 0.006));
    Hn = merge(Hn, spikeLine([[3.02, 1.95, 0], [3.5, 2.28, 0], [3.62, 2.42, 0]], 0.12, 0.01));
    meshes.push(M('hornm', 'head', shade(Hn, hornC)));
  }
  joints.push(J('jaw', [0.28, -0.42, 0], 'head'));
  meshes.push(M('jawm', 'jaw', shade(merge(
    cone([2.62, 1.62, 0], [3.24, 1.5, 0], 0.2, 0.06, { rings: 7, flat: 1.25 }),
    ball([2.6, 1.6, 0], 0.2, { sx: 1.05, sy: 0.7, sz: 1.1, rings: 6, segs: 3 })), hornC)));
  const tc = chain('tail', [[-2.1, 1.42, 0], [-2.8, 1.3, 0], [-3.4, 1.1, 0], [-3.85, 0.9, 0]], [0.34, 0.08], paint, { rings: 6 });
  joints.push(...tc.joints); meshes.push(...tc.meshes);
  return { joints, meshes };
}

/* ================= Stegosaurus (7 m, plates, thagomizer) =============== */
function stego() {
  const paint = hide(rgb('#6f7f4a'), rgb('#333a22'), rgb('#cfc7a2'), rgb('#57652f'),
    { bellyY: 1.2, grad: 0.18, speck: 0.18, spots: '#4d5a2c' });
  const plateC = hide(rgb('#b06a34'), rgb('#5a3016'), rgb('#e5b268'), rgb('#e5c07a'), { bellyY: 1.9, grad: 0.12, speck: 0.12 });
  const joints = [], meshes = [];
  for (const [n, x, top, r0, r1] of [['FL', 1.7, 1.25, 0.32, 0.2], ['FR', 1.7, 1.25, 0.32, 0.2],
    ['BL', -1.6, 1.75, 0.38, 0.24], ['BR', -1.6, 1.75, 0.38, 0.24]]) {
    const z = (n[1] === 'L' ? -1 : 1) * (n[0] === 'F' ? 0.62 : 0.72);
    joints.push(J('leg' + n, [x, top, z]));
    meshes.push(M('legm' + n, 'leg' + n, shade(tube([[x, top + 0.2, z], [x, top * 0.5, z], [x, 0.16, z]],
      f => [r0 - (r0 - r1) * f, r0 - (r0 - r1) * f], { rings: 7, segs: 4 }), paint)));
  }
  const spine = [[-2.7, 1.25, 0], [-1.8, 1.75, 0], [-0.6, 2.3, 0], [0.6, 2.4, 0], [1.6, 2.0, 0], [2.4, 1.4, 0]];
  meshes.push(M('bodym', null, shade(tube(spine, f => {
    const d = Math.abs(f - 0.5); return [0.8 - 0.5 * d, 0.95 - 0.55 * d];
  }, { rings: 9, segs: 12 }), paint)));
  joints.push(J('neck', [2.4, 1.4, 0]));
  meshes.push(M('neckm', 'neck', shade(tube([[2.3, 1.4, 0], [2.9, 1.1, 0]], f => [0.34 - 0.12 * f, 0.3 - 0.11 * f],
    { rings: 7, segs: 3 }), paint)));
  joints.push(J('head', [0.62, -0.34, 0], 'neck'));
  meshes.push(M('headm', 'head', shade(merge(
    ball([3.14, 1.0, 0], 0.24, { sx: 1.3, sy: 0.9, sz: 1.0, rings: 7, segs: 4 }),
    cone([3.2, 0.95, 0], [3.62, 0.86, 0], 0.15, 0.06, { rings: 6 })), paint)));
  meshes.push(M('eyesm', 'head', merge(whiteEye([3.16, 1.14, -0.19], 0.05), whiteEye([3.16, 1.14, 0.19], 0.05))));
  joints.push(J('jaw', [0.14, -0.2, 0], 'head'));
  meshes.push(M('jawm', 'jaw', shade(cone([3.1, 0.88, 0], [3.56, 0.8, 0], 0.11, 0.045, { rings: 6 }), paint)));
  const topSpine = [[-2.4, 1.75, 0], [-1.5, 2.3, 0], [-0.5, 2.85, 0], [0.6, 2.9, 0], [1.6, 2.45, 0]];
  meshes.push(M('platesm', null, shade(plateRow(topSpine, f => [0.55 - 0.35 * Math.abs(f - 0.45), 0.62 - 0.3 * Math.abs(f - 0.45), 0.34],
    9, { side: true }), plateC)));
  const tc = chain('tail', [[-2.7, 1.2, 0], [-3.5, 1.35, 0], [-4.3, 1.45, 0], [-4.9, 1.45, 0]], [0.34, 0.1], paint, { lift: 0.05, rings: 6 });
  joints.push(...tc.joints); meshes.push(...tc.meshes);
  {
    let S = empty();
    for (const z of [-0.16, 0.16]) {
      S = merge(S, spikeLine([[-4.35, 1.5, z], [-4.95, 2.3, z * 1.8]], 0.09, 0.006));
      S = merge(S, spikeLine([[-4.05, 1.45, z * 1.9], [-4.6, 2.05, z * 3.0]], 0.08, 0.006));
    }
    meshes.push(M('spikem', 'tail3', shade(S, plateC)));
  }
  return { joints, meshes };
}

/* ============ Sauropods: Diplodocus (whip tail) + Brachiosaurus ======== */
function sauropod(o) {
  const paint = hide(rgb(o.base), rgb(o.dark), rgb(o.belly), rgb(o.dark),
    { bellyY: o.bellyY, grad: 0.15, speck: 0.15, stripes: o.stripes, stripeFreq: 1.05, spots: o.spots });
  const joints = [], meshes = [];
  /* the body first, so the hips can be set from where the shell actually is:
     a joint below the belly leaves a leg you can see floating in mid air */
  const bodyP = tube(o.body, f => {
    const d = Math.abs(f - 0.45); return [o.bodyR * (1 - d * 1.15), o.bodyR * o.tall * (1 - d * 1.25)];
  }, { rings: 9, segs: 12 });
  for (const [n, x, top0, r0, r1] of o.legs) {
    const z = (n[1] === 'L' ? -1 : 1) * o.legZ(n[0] === 'F');
    const bel = bellyAt(bodyP, x, z);
    const top = bel == null ? top0 : Math.max(top0, bel + 0.55);   // the hip sits inside the body
    joints.push(J('leg' + n, [x, top, z]));
    meshes.push(M('legm' + n, 'leg' + n, shade(tube([[x, top + 0.3, z], [x, top * 0.5, z], [x, 0.2, z]],
      f => [r0 - (r0 - r1) * f, r0 - (r0 - r1) * f], { rings: 7, segs: 4 }), paint)));
    meshes.push(M('footm' + n, 'leg' + n, shade(ball([x, 0.22, z], r1 * 1.2, { sx: 1.2, sy: 0.6, sz: 1.1, rings: 6, segs: 3 }), paint)));
  }
  meshes.push(M('bodym', null, shade(bodyP, paint)));
  const nc = chain('neck', o.neck, o.neckR, paint, { rings: 8, lift: 0.05 });
  joints.push(...nc.joints); meshes.push(...nc.meshes);
  const last = 'neck' + (o.neck.length - 1), hd = o.head;
  joints.push(J('head', rel([hd[0], hd[1]], o.neck[o.neck.length - 1]), last));
  meshes.push(M('headm', 'head', shade(merge(
    ball([hd[0], hd[1], 0], o.headR, { sx: 1.35, sy: o.crest ? 1.25 : 0.95, sz: 0.98, rings: 7, segs: 4 }),
    cone([hd[0] + o.headR * 0.3, hd[1] - o.headR * 0.25, 0], [hd[0] + o.headR * 2.2, hd[1] - o.headR * 0.5, 0],
      o.headR * 0.62, o.headR * 0.26, { rings: 6 })), paint)));
  meshes.push(M('eyesm', 'head', merge(whiteEye([hd[0] + o.headR * 0.4, hd[1] + o.headR * 0.4, -o.headR * 0.72], o.headR * 0.2),
    whiteEye([hd[0] + o.headR * 0.4, hd[1] + o.headR * 0.4, o.headR * 0.72], o.headR * 0.2))));
  if (o.crest) meshes.push(M('crestm', 'head', shade(fin([[hd[0] - o.headR * 0.5, hd[1] + o.headR * 0.85],
    [hd[0] + o.headR * 0.8, hd[1] + o.headR * 1.5], [hd[0] + o.headR * 1.5, hd[1] + o.headR * 1.15],
    [hd[0] + o.headR * 0.3, hd[1] + o.headR * 0.85]], 0.06, { axis: 'xy', off: 0 }), paint)));
  joints.push(J('jaw', [o.headR * 0.2, -o.headR * 0.7, 0], 'head'));
  meshes.push(M('jawm', 'jaw', shade(cone([hd[0] - o.headR * 0.2, hd[1] - o.headR * 0.7, 0],
    [hd[0] + o.headR * 1.8, hd[1] - o.headR * 0.85, 0], o.headR * 0.5, o.headR * 0.2, { rings: 6 }), paint)));
  const tch = chain('tail', o.tail, o.tailR, paint, { rings: 7, lift: 0.05 });
  joints.push(...tch.joints); meshes.push(...tch.meshes);
  return { joints, meshes };
}
const diplodocus = () => sauropod({
  base: '#6c86a0', dark: '#33424f', belly: '#c9d6dd', stripes: '#3f5262', bellyY: 1.9,
  body: [[-3.2, 2.5, 0], [-1.6, 2.95, 0], [0.2, 3.1, 0], [1.7, 2.8, 0]], bodyR: 1.0, tall: 1.1,
  neck: [[1.7, 2.75, 0], [3.1, 3.3, 0], [4.4, 3.55, 0], [5.4, 3.5, 0]], neckR: [0.46, 0.26],
  head: [5.85, 3.45, 0], headR: 0.28,
  tail: [[-3.2, 2.45, 0], [-4.5, 2.4, 0], [-5.8, 2.3, 0], [-7.0, 2.1, 0], [-8.1, 1.85, 0], [-9.0, 1.5, 0], [-9.7, 1.1, 0]],
  tailR: [0.46, 0.07],
  legs: [['FL', 1.5, 2.0, 0.36, 0.23], ['FR', 1.5, 2.0, 0.36, 0.23], ['BL', -1.75, 2.25, 0.4, 0.25], ['BR', -1.75, 2.25, 0.4, 0.25]],
  legZ: f => f ? 0.74 : 0.68,
});
const brachiosaurus = () => sauropod({
  base: '#8d8f6a', dark: '#3d4030', belly: '#ded6b4', spots: '#6a6c4a', bellyY: 2.6, crest: true,
  body: [[-3.1, 3.4, 0], [-1.6, 4.05, 0], [0.3, 4.25, 0], [1.9, 4.15, 0]], bodyR: 1.3, tall: 1.15,
  neck: [[1.9, 4.1, 0], [2.9, 5.1, 0], [3.7, 6.0, 0], [4.3, 6.6, 0]], neckR: [0.7, 0.42],
  head: [4.7, 6.85, 0], headR: 0.38,
  tail: [[-3.1, 3.35, 0], [-4.3, 3.2, 0], [-5.4, 2.95, 0], [-6.3, 2.6, 0], [-7.1, 2.2, 0]],
  tailR: [0.6, 0.16],
  legs: [['FL', 1.6, 2.8, 0.52, 0.32], ['FR', 1.6, 2.8, 0.52, 0.32], ['BL', -1.8, 3.0, 0.55, 0.34], ['BR', -1.8, 3.0, 0.55, 0.34]],
  legZ: f => f ? 0.92 : 0.86,
});

/* ---------------------------------------------------------------------- */
const BUILDERS = { raptor, dilopho, trike, stego, diplodocus, brachiosaurus };
const OUT = path.resolve(import.meta.dirname, '..');
const keys = process.argv.slice(2).filter(k => BUILDERS[k]);
for (const k of (keys.length ? keys : Object.keys(BUILDERS))) {
  const model = BUILDERS[k]();
  checkLimbs(model, k);                              // refuses to ship a dino with floating legs
  const glb = buildGLB(model);
  writeFileSync(path.join(OUT, 'assets', k + '.glb'), glb);
  writeFileSync(path.join(OUT, 'assets', 'select', k + '.png'),
    renderCard(model, ['brachiosaurus', 'diplodocus'].includes(k) ? { yaw: -0.5, pitch: 0.1, fit: 0.94 } : {}));
  const tris = model.meshes.reduce((n, m) => n + m.part.idx.length / 3, 0);
  console.log(`${k}: ${(glb.length / 1024) | 0} KB, ${model.meshes.length} parts, ${tris} tris, ${model.joints.length} joints`);
}
