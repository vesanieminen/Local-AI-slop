/* dino_factory.mjs — builds the game's dinosaurs as rigged, vertex-coloured GLB models.
   ---------------------------------------------------------------------------
   Same convention as the shipped assets, which the game animates live:
     root -> joint nodes (named exactly as poseChar expects: thighL, shinL, footL,
     tail1..n, neck, head, jaw, armL, legFL..legBR) -> mesh nodes whose translation
     is the negated accumulated joint offset, so geometry is authored in root space
     and swings about its joint pivot.
   Meshes carry POSITION, NORMAL and COLOR_0 only (the game paints them itself).
   +X is forward, feet sit at y=0, units are metres.

   Run:  node tools/dino_factory.mjs [key ...]     (writes assets/<key>.glb
                                                     and assets/select/<key>.png)
   --------------------------------------------------------------------------- */
import { writeFileSync, mkdirSync } from 'fs';
import { deflateSync } from 'zlib';
import path from 'path';

/* ------------------------------- maths ---------------------------------- */
const sub = (a, b) => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const add = (a, b) => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
const mul = (a, s) => [a[0] * s, a[1] * s, a[2] * s];
const cross = (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const len = a => Math.hypot(a[0], a[1], a[2]);
const norm = a => { const l = len(a) || 1; return [a[0] / l, a[1] / l, a[2] / l]; };
const lerp3 = (a, b, t) => [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, (a[2] ?? 0) + ((b[2] ?? 0) - (a[2] ?? 0)) * t];
const hash = (x, y, z) => { let h = Math.sin(x * 127.1 + y * 311.7 + z * 74.7) * 43758.5453; return h - Math.floor(h); };
const catmull = (p, t) => {                          // smooth spine through control points
  const n = p.length;
  if (n < 2) return p[0] || [0, 0, 0];
  const u = Math.max(0, Math.min(0.9999, t)) * (n - 1);
  const i = Math.floor(u), f = u - i;
  const p0 = p[Math.max(0, i - 1)], p1 = p[i], p2 = p[Math.min(n - 1, i + 1)], p3 = p[Math.min(n - 1, i + 2)];
  const o = [0, 1, 2].map(k => 0.5 * ((2 * p1[k]) + (-p0[k] + p2[k]) * f +
    (2 * p0[k] - 5 * p1[k] + 4 * p2[k] - p3[k]) * f * f + (-p0[k] + 3 * p1[k] - 3 * p2[k] + p3[k]) * f * f * f));
  return o;
};

/* ------------------------------ part mesh ------------------------------- */
const empty = () => ({ pos: [], nrm: [], col: [], idx: [] });
function merge(a, b) {
  const o = a.pos.length / 3;
  a.pos.push(...b.pos); a.nrm.push(...b.nrm); a.col.push(...b.col);
  a.idx.push(...b.idx.map(i => i + o));
  return a;
}
function shade(part, colFn) {                         // paint every vertex of a part
  for (let i = 0; i < part.pos.length; i += 3) {
    const c = colFn(part.pos[i], part.pos[i + 1], part.pos[i + 2], i / 3);
    part.col.push(c[0], c[1], c[2]);
  }
  return part;
}
function computeNormals(part) {                       // flat-ish: per-face, then average
  const n = Array(part.pos.length / 3).fill(null).map(() => [0, 0, 0]);
  for (let i = 0; i < part.idx.length; i += 3) {
    const [a, b, c] = [part.idx[i], part.idx[i + 1], part.idx[i + 2]];
    const A = part.pos.slice(a * 3, a * 3 + 3), B = part.pos.slice(b * 3, b * 3 + 3), C = part.pos.slice(c * 3, c * 3 + 3);
    const fn = norm(cross(sub(B, A), sub(C, A)));
    for (const v of [a, b, c]) { n[v][0] += fn[0]; n[v][1] += fn[1]; n[v][2] += fn[2]; }
  }
  part.nrm = n.flatMap(v => norm(v));
  return part;
}

/* loft an organic tube along a smooth spine: elliptical rings, parallel transport */
function tube(spineIn, radii, opts = {}) {
  const segs = opts.segs ?? 14, rings = opts.rings ?? 8, cap = opts.cap ?? true;
  const spine = spineIn.map(p => [p[0], p[1], p[2] ?? 0]);   // specs may be 2D
  const pts = [];
  for (let i = 0; i <= segs; i++) pts.push(catmull(spine, i / segs));
  // tangent per ring, stable frame (avoid twist flips)
  const frames = [];
  let up = [0, 0, 1];
  for (let i = 0; i < pts.length; i++) {
    const t = norm(sub(pts[Math.min(i + 1, pts.length - 1)], pts[Math.max(i - 1, 0)]));
    let s = cross(t, up); if (len(s) < 1e-4) s = cross(t, [0, 1, 0]);
    s = norm(s); const u = norm(cross(s, t));
    frames.push({ t, s, u }); up = frames[i].u;
  }
  const P = empty();
  const vid = (i, k) => i * rings + k;
  for (let i = 0; i < pts.length; i++) {
    const f = Math.max(0, Math.min(0.9999, i / (pts.length - 1)));
    const r = radii(f), c = pts[i];
    for (let k = 0; k < rings; k++) {
      const a = k / rings * Math.PI * 2;
      const w = Math.sin(a), h = Math.cos(a);
      const off = add(mul(frames[i].s, w * r[0]), mul(frames[i].u, h * r[1]));
      const wob = opts.wob ? opts.wob(f, k) : 0;
      P.pos.push(c[0] + off[0] * (1 + wob), c[1] + off[1] * (1 + wob), c[2] + off[2] * (1 + wob));
    }
  }
  for (let i = 0; i < pts.length - 1; i++)
    for (let k = 0; k < rings; k++) {
      const k2 = (k + 1) % rings;
      P.idx.push(vid(i, k), vid(i + 1, k), vid(i + 1, k2), vid(i, k), vid(i + 1, k2), vid(i, k2));
    }
  if (cap) {
    const endCap = (ring, flip) => {
      const c = empty(), ci = P.pos.length / 3;
      const cx = pts[ring][0], cy = pts[ring][1], cz = pts[ring][2];
      c.pos.push(cx, cy, cz);
      for (let k = 0; k < rings; k++) c.pos.push(P.pos[(ring * rings + k) * 3], P.pos[(ring * rings + k) * 3 + 1], P.pos[(ring * rings + k) * 3 + 2]);
      for (let k = 0; k < rings; k++) {
        const a = 1 + k, b = 1 + (k + 1) % rings;
        if (flip) c.idx.push(0, b, a); else c.idx.push(0, a, b);
      }
      merge(P, c);
    };
    endCap(0, true); endCap(pts.length - 1, false);
  }
  return P;
}
/* tapered cone/frustum between two points */
function cone(a, b, r0, r1, opts = {}) {
  const rings = opts.rings ?? 6, segs = opts.segs ?? 4;
  const spine = [];
  for (let i = 0; i <= segs; i++) spine.push(lerp3(a, b, i / segs));
  return tube(spine, f => [r0 + (r1 - r0) * f, (r0 + (r1 - r0) * f) * (opts.flat ?? 1)], { rings, segs: segs, cap: true });
}
/* squashed ball */
function ball(c, r, opts = {}) {
  const rings = opts.rings ?? 7, segs = opts.segs ?? 5;
  const P = empty();
  const sx = opts.sx ?? 1, sy = opts.sy ?? 1, sz = opts.sz ?? 1;
  for (let i = 0; i <= segs; i++) {
    const th = i / segs * Math.PI, rr = Math.sin(th);
    for (let k = 0; k < rings; k++) {
      const a = k / rings * Math.PI * 2;
      P.pos.push(c[0] + Math.cos(a) * rr * r * sx, c[1] + Math.cos(th) * r * sy, c[2] + Math.sin(a) * rr * r * sz);
    }
  }
  for (let i = 0; i < segs; i++) for (let k = 0; k < rings; k++) {
    const k2 = (k + 1) % rings, a = i * rings + k, b = i * rings + k2, c2 = (i + 1) * rings + k, d = (i + 1) * rings + k2;
    P.idx.push(a, c2, d, a, d, b);
  }
  return P;
}
/* flat plate with a filled outline. axis 'xy' = standing fin (profile [x,y], thickness
   along z), axis 'xz' = lying plate (profile [x,z], thickness along y). The profile is a
   closed outline: it gets a fan on both faces plus a rim, so it reads as a solid plate. */
function fin(profile, thick, opts = {}) {
  const P = empty(), n = profile.length, axis = opts.axis || 'xy', o = opts.off || 0;
  const cx = profile.reduce((s, p) => s + p[0], 0) / n;
  const cy = profile.reduce((s, p) => s + p[1], 0) / n;
  const put = (a, b, dz) => axis === 'xy' ? P.pos.push(a, b, o + dz) : P.pos.push(a, o + dz, b);
  put(cx, cy, -thick * 0.5);                             // 0: front centre
  put(cx, cy, thick * 0.5);                              // 1: back centre
  for (const [a, b] of profile) put(a, b, -thick * 0.5); // 2..n+1: front rim
  for (const [a, b] of profile) put(a, b, thick * 0.5);  // n+2..2n+1: back rim
  const fRim = i => 2 + i, bRim = i => 2 + n + i;
  for (let i = 0; i < n; i++) {
    const j = (i + 1) % n;
    P.idx.push(0, fRim(j), fRim(i));                     // front face
    P.idx.push(1, bRim(i), bRim(j));                     // back face
    P.idx.push(fRim(i), fRim(j), bRim(j), fRim(i), bRim(j), bRim(i));  // rim
  }
  return P;
}
/* a row of standing plates along a spine, optionally alternating left/right */
function plateRow(spine, sizeFn, count, opts = {}) {
  let P = empty();
  for (let i = 0; i < count; i++) {
    const f = count === 1 ? 0.5 : i / (count - 1), c = catmull(spine, f);
    const s = sizeFn(f);
    const zoff = opts.side ? (i % 2 ? s[2] : -s[2]) : 0;
    const w = s[0], hgt = s[1] * (opts.hgt ?? 1);
    const prof = [[-w * 0.5, 0], [w * 0.5, 0], [w * 0.3, hgt], [-w * 0.34, hgt * 0.78]];
    const p = fin(prof, s[2] > 0 && opts.side ? 0.09 : s[2] || 0.08, { axis: 'xy', off: 0 });
    for (let k = 0; k < p.pos.length; k += 3) {
      p.pos[k] += c[0];
      p.pos[k + 1] = c[1] + p.pos[k + 1] * (opts.up ?? 1);
      p.pos[k + 2] += zoff + c[2];
    }
    P = merge(P, p);
  }
  return P;
}
/* spikes (Thagomizer, horns): a cone from a base toward a tip */
function spikeLine(spine, r0, r1, opts = {}) {
  return tube(spine, f => [r0 * (1 - f) + r1 * f, (r0 * (1 - f) + r1 * f) * (opts.flat ?? 1)],
    { rings: opts.rings ?? 5, segs: opts.segs ?? 4, cap: true });
}

/* ---------------------------- colour model ------------------------------ */
const rgb = h => [parseInt(h.slice(1, 3), 16) / 255, parseInt(h.slice(3, 5), 16) / 255, parseInt(h.slice(5, 7), 16) / 255];
const mixc = (a, b, t) => [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
/* procedural scale hide: back-to-belly gradient + speckle + optional stripes/spots */
function hide(base, dark, belly, top, opts = {}) {
  return (x, y, z) => {
    let c = mixc(base, belly, clamp01((opts.bellyY ?? 0) + 0.22 - y * (opts.grad ?? 0.16)));
    c = mixc(c, dark, clamp01((y - (opts.backY ?? 1.5)) * 0.25));
    if (top) c = mixc(c, top, clamp01((y - (opts.topY ?? 2.2)) * 0.4));
    const s = hash(Math.round(x * 7), Math.round(y * 7), Math.round(z * 7));
    c = mixc(c, dark, s * (opts.speck ?? 0.16));
    if (opts.stripes) {
      const w = Math.sin((x + (opts.stripeTilt ?? 0) * z) * (opts.stripeFreq ?? 1.4));
      if (w > 0.72) c = mixc(c, rgb(opts.stripes), clamp01((w - 0.72) * 3));
    }
    if (opts.spots) {
      const g = hash(Math.round(x * 3), Math.round(y * 3), Math.round(z * 3));
      if (g > 0.82) c = mixc(c, rgb(opts.spots), 0.75);
    }
    return c;
  };
}
const clamp01 = v => Math.max(0, Math.min(1, v));

/* ---------------------------- GLB assembly ------------------------------ */
function buildGLB(model) {
  /* model = { joints: [{name, t:[x,y,z], parent}], meshes: [{name, parent, part}] } */
  const nodes = [], meshes = [], bufferViews = [], accessors = [], bin = [];
  let binLen = 0;
  const push = (bytes, target) => {
    while (binLen % 4) { bin.push(Buffer.alloc(1)); binLen++; }
    const bv = { buffer: 0, byteOffset: binLen, byteLength: bytes.length };
    if (target) bv.target = target;
    bufferViews.push(bv); bin.push(bytes); binLen += bytes.length;
    return bufferViews.length - 1;
  };
  const acc = (bytes, type, count, comp, target, needBounds) => {
    const bv = push(bytes, target);
    const a = { bufferView: bv, componentType: comp, count, type };
    if (needBounds) {
      const f = new Float32Array(bytes.buffer, bytes.byteOffset, count * 3);
      const mn = [1e9, 1e9, 1e9], mx = [-1e9, -1e9, -1e9];
      for (let i = 0; i < count; i++) for (let k = 0; k < 3; k++) {
        const v = f[i * 3 + k]; if (v < mn[k]) mn[k] = v; if (v > mx[k]) mx[k] = v;
      }
      a.min = mn; a.max = mx;
    }
    accessors.push(a); return accessors.length - 1;
  };
  // joint node index by name, plus accumulated world translation (parents first, robustly)
  const jointIdx = {}, worldT = {};
  const jt = j => [j.t[0], j.t[1], j.t[2] ?? 0];
  let todo = model.joints.slice();
  while (todo.length) {
    const keep = [];
    for (const j of todo) {
      if (j.parent && worldT[j.parent] === undefined) { keep.push(j); continue; }
      worldT[j.name] = add(j.parent ? worldT[j.parent] : [0, 0, 0], jt(j));
    }
    if (keep.length === todo.length) throw new Error(`joint cycle or missing parent: ${keep.map(k => k.name + ' of ' + k.parent)}`);
    todo = keep;
  }
  for (const m of model.meshes) {
    const part = computeNormals(m.part);
    if (part.pos.some(v => !Number.isFinite(v)) || part.col.some(v => !Number.isFinite(v)))
      throw new Error(`${m.name}: NaN in geometry — check the control points`);
    /* Half the loft helpers wind their shells one way and half the other, which
       leaves parts inside-out (black in game, and back-face culled). Every shell
       here is built around its own middle, so weigh each face's normal against
       the way it sits from the part's centre: a negative total means inside-out. */
    const VP = part.pos, II = part.idx.slice();
    let x0 = 1e9, y0 = 1e9, z0 = 1e9, x1 = -1e9, y1 = -1e9, z1 = -1e9;
    for (let i = 0; i < VP.length; i += 3) {
      x0 = Math.min(x0, VP[i]); y0 = Math.min(y0, VP[i + 1]); z0 = Math.min(z0, VP[i + 2]);
      x1 = Math.max(x1, VP[i]); y1 = Math.max(y1, VP[i + 1]); z1 = Math.max(z1, VP[i + 2]);
    }
    const mid = [(x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2];
    let faceOut = 0;
    for (let i = 0; i < II.length; i += 3) {
      const a = II[i] * 3, b = II[i + 1] * 3, c = II[i + 2] * 3;
      const ux = VP[b] - VP[a], uy = VP[b + 1] - VP[a + 1], uz = VP[b + 2] - VP[a + 2];
      const vx = VP[c] - VP[a], vy = VP[c + 1] - VP[a + 1], vz = VP[c + 2] - VP[a + 2];
      const nx = uy * vz - uz * vy, ny = uz * vx - ux * vz, nz = ux * vy - uy * vx;   // 2x area, by winding
      const fx = (VP[a] + VP[b] + VP[c]) / 3 - mid[0];
      const fy = (VP[a + 1] + VP[b + 1] + VP[c + 1]) / 3 - mid[1];
      const fz = (VP[a + 2] + VP[b + 2] + VP[c + 2]) / 3 - mid[2];
      faceOut += nx * fx + ny * fy + nz * fz;
    }
    let nrmA = part.nrm;
    if (faceOut < 0) {
      for (let i = 0; i < II.length; i += 3) { const t = II[i + 1]; II[i + 1] = II[i + 2]; II[i + 2] = t; }
      nrmA = part.nrm.map(v => -v);
    }
    const pos = Buffer.from(new Float32Array(part.pos).buffer);
    const nrm = Buffer.from(new Float32Array(nrmA).buffer);
    const col = Buffer.from(new Float32Array(part.col).buffer);
    const idx = Buffer.from(new Uint16Array(II).buffer);
    const P = acc(pos, 'VEC3', part.pos.length / 3, 5126, 34962, true);
    const N = acc(nrm, 'VEC3', part.nrm.length / 3, 5126, 34962, false);
    const C = acc(col, 'VEC3', part.col.length / 3, 5126, 34962, false);
    const I = acc(idx, 'SCALAR', part.idx.length, 5123, 34963, false);
    meshes.push({ name: m.name, primitives: [{ attributes: { POSITION: P, NORMAL: N, COLOR_0: C }, indices: I }] });
  }
  const rootIdx = nodes.push({ name: 'root' }) - 1;
  const kidsOf = {};
  for (const j of model.joints) {
    const parent = j.parent ? jointIdx[j.parent] : rootIdx;
    const idx = nodes.push({ name: j.name, translation: jt(j) }) - 1;
    jointIdx[j.name] = idx;
    (kidsOf[parent] ||= []).push(idx);
  }
  for (const m of model.meshes) {
    const parent = m.parent ? jointIdx[m.parent] : rootIdx;
    const off = m.parent ? mul(worldT[m.parent], -1) : [0, 0, 0];
    const idx = nodes.push({ name: m.name, mesh: meshes.findIndex(x => x.name === m.name),
      translation: off.length ? off : undefined }) - 1;
    (kidsOf[parent] ||= []).push(idx);
  }
  for (const [p, kids] of Object.entries(kidsOf)) nodes[p].children = kids;
  const json = {
    asset: { generator: 'dino_factory (Local-AI-slop)', version: '2.0' },
    scene: 0, scenes: [{ name: 'Scene', nodes: [rootIdx] }], nodes, meshes,
    accessors, bufferViews, buffers: [{ byteLength: binLen }],
  };
  const jb = Buffer.from(JSON.stringify(json), 'utf8');
  const jp = Buffer.alloc((4 - jb.length % 4) % 4, 0x20);
  const bb = Buffer.concat(bin);
  const bp = Buffer.alloc((4 - bb.length % 4) % 4, 0);
  const jsonChunk = Buffer.concat([jb, jp]), binChunk = Buffer.concat([bb, bp]);
  const total = 12 + 8 + jsonChunk.length + 8 + binChunk.length;
  const out = Buffer.alloc(total);
  out.writeUInt32LE(0x46546c67, 0); out.writeUInt32LE(2, 4); out.writeUInt32LE(total, 8);
  out.writeUInt32LE(jsonChunk.length, 12); out.writeUInt32LE(0x4e4f534a, 16);
  jsonChunk.copy(out, 20);
  let o = 20 + jsonChunk.length;
  out.writeUInt32LE(binChunk.length, o); out.writeUInt32LE(0x004e4942, o + 4);
  binChunk.copy(out, o + 8);
  return out;
}

/* ------------------------ card thumbnail (PNG) -------------------------- */
/* a tiny software rasteriser: one 3/4 view, flat shaded, transparent ground */
function renderCard(model, { w = 320, h = 180, yaw = -0.62, pitch = 0.16, fit = 0.9 } = {}) {
  const tris = [];
  for (const m of model.meshes) {
    const P = computeNormals(m.part);
    for (let i = 0; i < P.idx.length; i += 3) {
      const v = [0, 1, 2].map(k => {
        const j = P.idx[i + k] * 3;
        return { p: P.pos.slice(j, j + 3), c: P.col.slice(j, j + 3) };
      });
      const fn = norm(cross(sub(v[1].p, v[0].p), sub(v[2].p, v[0].p)));
      const cc = [(v[0].c[0] + v[1].c[0] + v[2].c[0]) / 3, (v[0].c[1] + v[1].c[1] + v[2].c[1]) / 3, (v[0].c[2] + v[1].c[2] + v[2].c[2]) / 3];
      tris.push({ v, fn, c: cc });
    }
  }
  // bounds
  let lo = [1e9, 1e9, 1e9], hi = [-1e9, -1e9, -1e9];
  for (const t of tris) for (const vv of t.v) for (let k = 0; k < 3; k++) {
    if (vv.p[k] < lo[k]) lo[k] = vv.p[k]; if (vv.p[k] > hi[k]) hi[k] = vv.p[k];
  }
  const mid = [(lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, (lo[2] + hi[2]) / 2];
  const spanX = hi[0] - lo[0], spanY = hi[1] - lo[1], spanZ = hi[2] - lo[2];
  const span = Math.max(spanX, spanY, spanZ) || 1;
  const sc = Math.min((w - 16) * fit / Math.max(spanX, spanZ), (h - 16) * fit / spanY) * 0.98;
  const cy = Math.cos(yaw), sy = Math.sin(yaw), cp = Math.cos(pitch), sp2 = Math.sin(pitch);
  const proj = p => {
    let x = p[0] - mid[0], y = p[1] - mid[1], z = p[2] - mid[2];
    const x1 = x * cy - z * sy, z1 = x * sy + z * cy;
    const y1 = y * cp - z1 * sp2;
    return [x1 * sc + w / 2, h / 2 - y1 * sc, y * sp2 + z1 * cp];
  };
  const S = 2, W = w * S, H = h * S;
  const buf = new Float32Array(W * H * 4), zb = new Float32Array(W * H).fill(-1e9);
  const light = norm([-0.35, 0.8, -0.5]);
  for (const t of tris) {
    const sh = 0.42 + 0.58 * Math.max(0, t.fn[0] * light[0] + t.fn[1] * light[1] + t.fn[2] * light[2]);
    const rim = 1 + 0.10 * Math.abs(t.fn[2]);
    const c = t.c.map(x => Math.min(1, x * sh * rim));
    const q = t.v.map(v => { const s = proj(v.p); return [s[0] * S, s[1] * S, s[2]]; });
    const x0 = Math.max(0, Math.floor(Math.min(...q.map(v => v[0])))), x1 = Math.min(W - 1, Math.ceil(Math.max(...q.map(v => v[0]))));
    const y0 = Math.max(0, Math.floor(Math.min(...q.map(v => v[1])))), y1 = Math.min(H - 1, Math.ceil(Math.max(...q.map(v => v[1]))));
    const denom = (q[1][1] - q[2][1]) * (q[0][0] - q[2][0]) + (q[2][0] - q[1][0]) * (q[0][1] - q[2][1]);
    if (Math.abs(denom) < 1e-6) continue;
    for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) {
      const px = x + 0.5, py = y + 0.5;
      const a = ((q[1][1] - q[2][1]) * (px - q[2][0]) + (q[2][0] - q[1][0]) * (py - q[2][1])) / denom;
      const b = ((q[2][1] - q[0][1]) * (px - q[2][0]) + (q[0][0] - q[2][0]) * (py - q[2][1])) / denom;
      const cc = 1 - a - b;
      if (a < 0 || b < 0 || cc < 0) continue;
      const zz = a * q[0][2] + b * q[1][2] + cc * q[2][2];
      const o = (y * W + x);
      if (zz <= zb[o]) continue;
      zb[o] = zz;
      const k = o * 4;
      buf[k] = c[0]; buf[k + 1] = c[1]; buf[k + 2] = c[2]; buf[k + 3] = 1;
    }
  }
  // downsample S×S, straight alpha, then encode PNG
  const raw = Buffer.alloc(H * W * 4);
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    let r = 0, g = 0, b = 0, a = 0;
    for (let dy = 0; dy < S; dy++) for (let dx = 0; dx < S; dx++) {
      const o = ((y * S + dy) * W + (x * S + dx)) * 4;
      r += buf[o]; g += buf[o + 1]; b += buf[o + 2]; a += buf[o + 3];
    }
    const n = S * S, o2 = (y * w + x) * 4, al = a / n;
    raw[o2] = Math.round((r / n) * 255); raw[o2 + 1] = Math.round((g / n) * 255);
    raw[o2 + 2] = Math.round((b / n) * 255); raw[o2 + 3] = Math.round(al * 255);
  }
  const stride = w * 4 + 1, filt = Buffer.alloc(stride * h);
  for (let y = 0; y < h; y++) { filt[y * stride] = 0; raw.copy(filt, y * stride + 1, y * w * 4, (y + 1) * w * 4); }
  const chunk = (type, data) => {
    const len = Buffer.alloc(4); len.writeUInt32BE(data.length);
    const td = Buffer.concat([Buffer.from(type, 'ascii'), data]);
    const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(td) >>> 0);
    return Buffer.concat([len, td, crc]);
  };
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(w, 0); ihdr.writeUInt32BE(h, 4); ihdr[8] = 8; ihdr[9] = 6;
  return Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk('IHDR', ihdr), chunk('IDAT', deflateSync(filt, { level: 9 })), chunk('IEND', Buffer.alloc(0))]);
}
let crcTable = null;
function crc32(buf2) {
  if (!crcTable) {
    crcTable = [];
    for (let n = 0; n < 256; n++) { let c = n; for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1; crcTable[n] = c >>> 0; }
  }
  let c = 0xffffffff;
  for (const b of buf2) c = crcTable[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

export { sub, add, mul, cross, norm, lerp3, catmull, hash, empty, merge, shade, tube, cone, ball, fin, plateRow, spikeLine,
  rgb, mixc, hide, buildGLB, renderCard, computeNormals };
