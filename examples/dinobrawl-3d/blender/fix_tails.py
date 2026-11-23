"""fix_tails.py — tail-continuity repair for the scriptless six.

The first twelve models were built live before build scripts were saved, so
trex, allo, spino, ankylo, dr and im have NO build recipe — their GLB is the
truth. Those bodies ended in a tall flat rear wall with the tail pegged below
it, and their tail segments met in visible steps. This module repairs the
imported GLBs in-place in the live Blender (per the standing Blender-MCP
rule), for re-export:

1. slope_rump  — sinks the body's rear wall into the tail root along a ramp,
2. telescope   — pulls each tail segment's front ring inside the segment
                 ahead so the silhouette flows (used alone for the
                 Ankylosaurus, whose single tail plate covers its own seam),
3. collar      — a vertex-coloured "hipm" skirt parented to the BODY over
                 the rump seam, so tail sway can never open the joint again.

repair() runs all three with auto-derived parameters. In a live session with
the species GLB imported:

    exec(open(".../blender/fix_tails.py").read())
    repair()   # then export the root like dt.export_glb would

For every model that HAS a build script, continuity is baked at the source
instead with dt.flare (see dt.py "patch FIX @tail-continuity").
"""

import bpy, bmesh, numpy as np
from mathutils import Vector

# ── Dino Brawl 3D: tail/rump continuity repair ────────────────────────────
# The Tyrannosaurus-era models (trex, allo, spino, ankylo, dr, im) end the
# body in a tall flat slab with the tail dangling below it, and the tail
# segments meet in visible steps. This:
#   1. slopes the body's rear wall down into the tail root,
#   2. telescopes each tail segment's front ring into the segment ahead,
#   3. adds a vertex-coloured hip collar over the seam (on the BODY, so
#      tail sway can never open the joint again).
# Run in a scene where the species GLB is already imported.
# Params: RUMP=(x_start, ramp, z_line, squash, y_squash); TAIL=depth/scale list.

def mean_colour(ob):
    me = ob.data
    ca = next((c for c in me.color_attributes
               if c.data_type in ("BYTE_COLOR", "FLOAT_COLOR")), None)
    if ca is None:
        return (0.5, 0.45, 0.4)
    dat = ca.data
    def rgb(d):
        c = getattr(d, "color", None)             # BYTE_COLOR gives .color, FLOAT_COLOR is the vector
        return c if c is not None else d
    n = len(dat)
    c = [sum(rgb(dat[i])[k] for i in range(n)) / n for k in range(3)]
    return c

def slope_rump(body_name, x0, ramp, z_line, squash, y_sq, ref=None):
    ob = bpy.data.objects[body_name]
    M = ob.matrix_world; Mi = M.inverted()
    zc = z_line
    if ref:                                          # sit the tail-line on the tail root
        t = bpy.data.objects[ref]
        ws = [t.matrix_world @ v.co for v in t.data.vertices]
        xf = max(w.x for w in ws)
        zc = sum(w.z for w in ws if w.x > xf - 0.4) / max(1, len([w for w in ws if w.x > xf - 0.4]))
    for v in ob.data.vertices:
        w = M @ v.co
        if w.x > x0: continue
        t = min((x0 - w.x) / ramp, 1.0) ** 1.2
        w2 = Vector((w.x, w.y * (1 - y_sq * t), zc + (w.z - zc) * (1 - squash * t)))
        v.co = Mi @ w2
    ob.data.update()

def telescope(mesh_name, depth, scale, win=0.30):
    ob = bpy.data.objects[mesh_name]
    M = ob.matrix_world; Mi = M.inverted()
    ws = [M @ v.co for v in ob.data.vertices]
    xm = sum(w.x for w in ws)/len(ws)
    Cf = Vector((0,0,0)); Cb = Vector((0,0,0)); nf = nb = 0
    for w in ws:
        if w.x > xm: Cf += w; nf += 1
        else: Cb += w; nb += 1
    if not nf or not nb: return
    Cf /= nf; Cb /= nb
    u = (Cf - Cb).normalized()
    proj = [(w - Cf).dot(u) for w in ws]
    pmax = max(proj); pmin = min(proj); w0 = pmax - (pmax - pmin) * win
    for v, w, p in zip(ob.data.vertices, ws, proj):
        if p <= w0: continue
        f = (p - w0) / (pmax - w0)
        foot = Cf + u * p
        w2 = foot + u * (f * depth) + (w - foot) * (1 - f * (1 - scale))
        v.co = Mi @ w2
    ob.data.update()

def collar(x0, x1, nseg, r_fun, parent, z_fun=None, col=None):
    """Open tube skirt from x0 (inside body) to x1 (over first tail segs)."""
    mesh = bpy.data.meshes.new("hipm"); ob = bpy.data.objects.new("hipm", mesh)
    bpy.context.collection.objects.link(ob)
    bm = bmesh.new(); prev = None
    for i in range(nseg + 1):
        x = x0 + (x1 - x0) * i / nseg
        ry, rz = r_fun(x)
        z = (z_fun(x) if z_fun else 2.4)
        ring = [bm.verts.new((x, ry*0.94*np.cos(a), z + rz*np.sin(a)))
                for a in np.linspace(0, 6.2832, 11)[:-1]]
        if prev:
            for j in range(10):
                bm.faces.new((prev[j], prev[(j+1) % 10], ring[(j+1) % 10], ring[j]))
        prev = ring
    bm.normal_update(); bm.to_mesh(mesh); bm.free()
    ca = mesh.color_attributes.new(name="COLOR_0", type="BYTE_COLOR", domain="POINT")
    for i in range(len(mesh.vertices)): ca.data[i].color = (*col, 1)
    ob.parent = parent
    return ob

def repair(tail_mocks=("tailm1","tailm2","tailm3","tailm4"),
           rump="auto",
           segs=((0.45,0.90),(0.34,0.86),(0.32,0.86),(0.30,0.86)),
           coll="auto"):
    if rump == "auto" or coll == "auto":
        t = bpy.data.objects[tail_mocks[0]]
        xf = max((t.matrix_world @ v.co).x for v in t.data.vertices)
        if rump == "auto":
            rump = ("bodym", xf + 0.32, 0.66, None, 0.52, 0.34)
        if coll == "auto":
            coll = (xf + 0.28, xf - 1.15)
    body, x0, ramp, zline, sq, ysq = rump
    if zline is None:
        slope_rump(body, x0, ramp, 0, sq, ysq, ref=tail_mocks[0])
    else:
        slope_rump(body, x0, ramp, zline, sq, ysq)
    for name, (dep, sc) in zip(tail_mocks, segs):
        if name in bpy.data.objects: telescope(name, dep, sc)
    if all(coll):
        x0c, x1c = coll
        src = bpy.data.objects[body]
        col = mean_colour(src)
        # collar centreline follows the tail root height, radius follows the local tube
        tr = bpy.data.objects[tail_mocks[0]]
        ws = [tr.matrix_world @ v.co for v in tr.data.vertices]
        xf = max(w.x for w in ws)
        root = [w for w in ws if w.x > xf - 0.55]
        zc = sum(w.z for w in root) / len(root)
        hh = (max(w.z for w in root) - min(w.z for w in root)) / 2
        hw = (max(w.y for w in root) - min(w.y for w in root)) / 2
        def rf(x):
            t = max(0.0, min(1.0, (x1c - x) / (x1c - x0c)))   # 0 at body end, 1 at tail end
            return (hw + 0.10 - 0.06*t, hh + 0.12 - 0.09*t)
        collar(x0c, x1c, 2, rf, bpy.data.objects["root"], z_fun=lambda x: zc - 0.05, col=col)
    print("repaired")
