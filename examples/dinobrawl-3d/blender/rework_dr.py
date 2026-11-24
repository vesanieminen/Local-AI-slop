"""rework_dr.py — the 2026-11-24 Distortus rex restyle, run live on the imported
assets/dr.glb through Blender MCP (the standing rule: models are shaped in the
live session, never by headless factories). This module is the recipe as
executed; run it in a session with the OLD dr.glb imported to reproduce:

    exec(open(".../blender/rework_dr.py").read())
    rework_dr()          # then export the root and strip the .001 name suffixes

The look follows the Prehistoric-Park "predator" reference: humped shoulders,
thick neck, big deep blunt skull, long arms with huge scythe claws, and a sand
hide with rust freckling — replacing the old blue-grey.
"""

import bpy, math
from mathutils import Vector

# ---------------------------------------------------------------- helpers
def _hash(x, y, z):
    h = math.sin(x * 127.1 + y * 311.7 + z * 74.7) * 43758.5453
    return h - math.floor(h)

def _rgb(s):
    s = s.lstrip("#"); return [int(s[i:i + 2], 16) / 255 for i in (0, 2, 4)]

def _mix(a, b, t):
    t = max(0.0, min(1.0, t)); return [a[i] + (b[i] - a[i]) * t for i in range(3)]

def wverts(ob, fn):
    M = ob.matrix_world; Mi = M.inverted()
    for v in ob.data.vertices:
        v.co = Mi @ fn(M @ v.co)

def scale_about(o, c, s):
    wverts(o, lambda p: c + (p - c) * s)

def repaint(ob, base, dark, belly, rust=0.80, speck=0.26, grad=0.16,
            zc=3.2, rustmix=0.45, back=0.16, keep=None):
    me = ob.data
    for ca in list(me.color_attributes): me.color_attributes.remove(ca)
    ca = me.color_attributes.new(name="COLOR_0", type="BYTE_COLOR", domain="POINT")
    M = ob.matrix_world; cols = []
    TAN, BELLY, DEEP, RUST, SPOT = base, belly, _rgb("#4a3b25"), _rgb("#8f4f2e"), _rgb("#4a3a28")
    for v in me.vertices:
        p = M @ v.co
        c = _mix(TAN, BELLY, 0.22 + (zc - p.z) * grad)
        c = _mix(c, dark, max(0.0, min(1.0, (p.z - zc - 0.55) * back)))
        c = _mix(c, DEEP, _hash(round(p.x * 6), round(p.y * 6), round(p.z * 6)) * speck)
        if _hash(round(p.x * 1.7), round(p.y * 1.7), round(p.z * 1.7)) > rust:
            c = _mix(c, RUST, rustmix)
        if _hash(round(p.x * 3.1), round(p.y * 3.1), round(p.z * 3.1)) > 0.9:
            c = _mix(c, SPOT, 0.6)
        if keep: c = keep(p, c)
        cols.append((c[0], c[1], c[2], 1.0))
    ca.data.foreach_set("color", [ch for c in cols for ch in c])

# ---------------------------------------------------------------- geometry
def _geometry():
    O = bpy.data.objects
    hump = lambda x: 0.30 * math.exp(-((x - 1.35) / 0.9) ** 2) + 0.12 * math.exp(-((x - 0.2) / 1.6) ** 2)
    thick = lambda x: 1.0 + 0.16 * math.exp(-((x - 1.2) / 1.1) ** 2) + 0.08 * math.exp(-((x - 0.1) / 1.8) ** 2)
    wverts(O["bodym"], lambda p: Vector((p.x, p.y * thick(p.x), p.z + hump(p.x))))

    n0, n1 = Vector((1.74, 0, 3.35)), Vector((4.02, 0, 3.82))
    def neck_fn(p):
        t = max(0.0, min(1.0, (p.x - n0.x) / (n1.x - n0.x)))
        zc = n0.z + (n1.z - n0.z) * t
        return Vector((p.x, p.y * 1.32, zc + (p.z - zc) * 1.30))
    wverts(O["neckm"], neck_fn)

    H = O["head"].matrix_world.translation
    for nm in ["headm", "dome", "browm", "eyesm", "teethUm"]:
        scale_about(O[nm], H, 1.28)
    J = O["jaw"].matrix_world.translation
    for nm in ["jawm", "teethLm"]:
        scale_about(O[nm], J, 1.24)

    for joint, meshes in [("armL", ["armmL", "armclawL"]), ("armR", ["armmR", "armclawR"]),
                          ("arm2L", ["arm2mL", "arm2clawL"]), ("arm2R", ["arm2mR", "arm2clawR"])]:
        C = O[joint].matrix_world.translation
        for nm in meshes: scale_about(O[nm], C, 1.2)
    for nm in ["armclawL", "armclawR", "arm2clawL", "arm2clawR"]:
        o = O[nm]; M = o.matrix_world
        pts = [M @ v.co for v in o.data.vertices]
        base = Vector((sum(p.x for p in pts) / len(pts), sum(p.y for p in pts) / len(pts),
                       max(p.z for p in pts)))
        scale_about(o, base, 1.35)

    for nm, ax in [("thighmL", (-0.84, 0.6, 2.2)), ("thighmR", (-0.84, -0.6, 2.2))]:
        c = Vector(ax)
        wverts(O[nm], lambda p, c=c: c + Vector((p.x - c.x, (p.y - c.y) * 1.14, (p.z - c.z) * 1.08)))

    def hip_fn(p):
        w = math.exp(-((p.x + 1.9) / 0.9) ** 2)
        return Vector((p.x, p.y * (1 + 0.14 * w), p.z))
    wverts(O["bodym"], hip_fn)
    wverts(O["hipm"], hip_fn)

    EYE_X = 4.28
    def snout_fn(p):                       # blunt snout, boxier back skull
        if p.x > EYE_X:
            p = Vector((p.x - (p.x - EYE_X) * 0.12, p.y, p.z))
        if p.x < EYE_X + 0.15 and p.z > 3.92:
            w = min(1.0, (EYE_X + 0.15 - p.x) / 0.8)
            p = Vector((p.x, p.y, p.z + 0.10 * w))
        return p
    for nm in ["headm", "teethUm", "jawm", "teethLm", "eyesm", "dome", "browm"]:
        wverts(O[nm], snout_fn)

# ---------------------------------------------------------------- colours
def _colours():
    O = bpy.data.objects
    TAN, DARK, BELLY = _rgb("#bda985"), _rgb("#7a6749"), _rgb("#e2d6b8")
    for nm in ["bodym", "neckm", "hipm", "tailm1", "tailm2", "tailm3", "tailm4", "thighmL", "thighmR"]:
        repaint(O[nm], TAN, DARK, BELLY)
    for nm in ["footmL", "footmR", "shinmL", "shinmR"]:
        repaint(O[nm], _rgb("#9c8a68"), DARK, BELLY, rust=0.83)

    def headkeep(p, c):
        if _hash(round(p.x * 2.4), round(p.y * 2.4), round(p.z * 2.4)) > 0.86:
            c = _mix(c, _rgb("#8f4f2e"), 0.3)
        return c
    repaint(O["headm"], _rgb("#b5a17d"), DARK, BELLY, keep=headkeep)
    repaint(O["dome"], _rgb("#b3a183"), DARK, BELLY, rust=0.88, speck=0.34)
    repaint(O["browm"], _rgb("#66503a"), DARK, BELLY, rust=0.99, speck=0.2)

    repaint(O["jawm"], _rgb("#b5a17d"), DARK, BELLY, rust=0.86)   # bright gullet, tight band
    jawO = O["jawm"]; ca = jawO.data.color_attributes["COLOR_0"]; pk = _rgb("#b58172")
    for i, v in enumerate(jawO.data.vertices):
        p = jawO.matrix_world @ v.co
        if p.z > 3.62 and p.x < 4.3 and abs(p.y) < 0.34:
            ca.data[i].color = (*_mix(list(ca.data[i].color[:3]), pk, 0.8), 1.0)

    for nm in ["armmL", "armmR", "arm2mL", "arm2mR"]:
        repaint(O[nm], _rgb("#978468"), DARK, BELLY, rust=0.84)
    for nm in ["armclawL", "armclawR", "arm2clawL", "arm2clawR", "clawmL", "clawmR"]:
        repaint(O[nm], _rgb("#26211a"), _rgb("#181410"), _rgb("#3a3228"), rust=0.99, speck=0.25, grad=0.05)
    repaint(O["platesm"], _rgb("#6d4f36"), DARK, BELLY, rust=0.97, speck=0.28)
    bc = _rgb("#ece4d2")
    for nm in ["teethUm", "teethLm"]:
        me = O[nm].data
        for ca in list(me.color_attributes): me.color_attributes.remove(ca)
        ca = me.color_attributes.new(name="COLOR_0", type="BYTE_COLOR", domain="POINT")
        for i in range(len(ca.data)): ca.data[i].color = (*bc, 1.0)

    eye = O["eyesm"]                                       # bigger, amber
    C = Vector((0.0, 0.0, 0.0))
    for v in eye.data.vertices: C += eye.matrix_world @ v.co
    C /= len(eye.data.vertices)
    M = eye.matrix_world; Mi = M.inverted()
    for v in eye.data.vertices:
        p = M @ v.co; v.co = Mi @ (C + (p - C) * 1.28)
    ca = eye.data.color_attributes["COLOR_0"]
    lums = [sum(ca.data[i].color[:3]) / 3 for i in range(len(ca.data))]
    AMB, DK = _rgb("#dd9f3f"), _rgb("#14100c")
    for i in range(len(ca.data)):
        ca.data[i].color = (*(AMB if lums[i] > 0.5 else DK), 1.0)

def rework_dr():
    _geometry()
    _colours()
    print("dr reworked — export, then strip the .001 suffixes from node names")
