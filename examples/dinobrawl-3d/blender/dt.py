"""dt — the helper toolkit used to build every Dino Brawl 3D model through Blender MCP.
Verbatim record: the base HELPERS definition plus every exec(..., dt.__dict__) patch
in original order (verified instruction-for-instruction against the live module that
built mosasaurus.glb). Coordinates: x forward, y lateral, z up, metres.
Build ALL new models through Blender MCP on the live Blender: exec this file ONCE,
then run a build_*.py script (also in this folder), then dt.export_glb().
See README.md beside this file."""


import bpy, bmesh, math, mathutils

REG = {"joints": {}, "objects": {}}
def reset(): REG["joints"] = {}; REG["objects"] = {}

def joint(name, local, parent=None):
    eo = bpy.data.objects.new(name, None)
    bpy.context.collection.objects.link(eo)
    if parent is not None: eo.parent = REG["objects"][parent]
    eo.location = local
    pw = REG["joints"].get(parent) if parent else mathutils.Matrix.Identity(4)
    REG["joints"][name] = pw @ mathutils.Matrix.Translation(local)
    REG["objects"][name] = eo
    return eo

def part(name, parent_joint, me):
    o = bpy.data.objects.new(name, me)
    bpy.context.collection.objects.link(o)
    o.parent = REG["objects"][parent_joint]
    o.matrix_parent_inverse = mathutils.Matrix.Identity(4)
    o.matrix_basis = REG["joints"][parent_joint].inverted()
    REG["objects"][name] = o
    return o

def apply_mods(o):
    ev = o.evaluated_get(bpy.context.evaluated_depsgraph_get())
    me = bpy.data.meshes.new_from_object(ev, preserve_all_data_layers=False)
    old = o.data; o.data = me
    if old.users == 0: bpy.data.meshes.remove(old)
    for m in list(o.modifiers): o.modifiers.remove(m)
    return o

def smooth(o):
    for p in o.data.polygons: p.use_smooth = True

def recalc(o):
    bm = bmesh.new(); bm.from_mesh(o.data)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    if bm.calc_volume(signed=True) < 0:
        for f in bm.faces: f.normal_flip()
    bm.to_mesh(o.data); bm.free()

def tube(name, pts, radii, parent, subsurf=1):
    me = bpy.data.meshes.new(name)
    me.from_pydata([tuple(p) for p in pts], [(i, i+1) for i in range(len(pts)-1)], [])
    o = part(name, parent, me)
    md = o.modifiers.new("skin", 'SKIN'); md.use_smooth_shade = True
    bpy.context.view_layer.update()
    svs = o.data.skin_vertices[0].data
    svs[0].use_root = True
    for sv, (w, h) in zip(svs, radii):
        sv.radius = (max(w, 0.008), max(h, 0.008))
    if subsurf:
        ss = o.modifiers.new("sub", 'SUBSURF'); ss.levels = subsurf; ss.render_levels = subsurf
    bpy.context.view_layer.update()
    apply_mods(o); smooth(o); recalc(o)
    return o

def blob(name, balls, parent, target_faces=90):
    mb = bpy.data.metaballs.new(name)
    mb.resolution = 0.05
    for b in balls:
        e = mb.elements.new(type='ELLIPSOID')
        e.co = b[:3]; e.radius = 1.0
        e.size_x, e.size_y, e.size_z = b[3], b[4], b[5]
    mo = bpy.data.objects.new(name + "_meta", mb)
    bpy.context.collection.objects.link(mo)
    bpy.context.view_layer.update()
    ev = mo.evaluated_get(bpy.context.evaluated_depsgraph_get())
    me = bpy.data.meshes.new_from_object(ev)
    bpy.data.objects.remove(mo, do_unlink=True); bpy.data.metaballs.remove(mb)
    o = part(name, parent, me)
    n = len(o.data.polygons)
    dm = o.modifiers.new("dec", 'DECIMATE'); dm.ratio = max(0.02, target_faces / max(n, 1))
    apply_mods(o); smooth(o); recalc(o)
    return o

def spike_line(name, pts, r, parent, segs=5):
    me0 = None
    for i in range(len(pts) - 1):
        a, b = mathutils.Vector(pts[i]), mathutils.Vector(pts[i+1])
        t0, t1 = i / (len(pts) - 1), (i + 1) / (len(pts) - 1)
        r1, r2 = r * (1 - t0) + 0.002, r * (1 - t1) + 0.002
        d = b - a; L = d.length
        bm = bmesh.new()
        bmesh.ops.create_cone(bm, cap_ends=True, segments=segs, radius1=r1, radius2=r2, depth=L)
        q = mathutils.Vector((0, 0, 1)).rotation_difference(d.normalized())
        for v in bm.verts: v.co = q @ v.co + (a + d * 0.5)
        me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
        if me0 is None: me0 = me
        else:
            tmpo = bpy.data.objects.new("_j1", me0); bpy.context.collection.objects.link(tmpo)
            srco = bpy.data.objects.new("_j2", me);  bpy.context.collection.objects.link(srco)
            bpy.ops.object.select_all(action='DESELECT')
            srco.select_set(True); tmpo.select_set(True)
            bpy.context.view_layer.objects.active = tmpo
            bpy.ops.object.join()
            bpy.data.objects.remove(srco, do_unlink=True)
            bpy.data.objects.remove(tmpo, do_unlink=True)
            me0 = tmpo.data
    o = part(name, parent, me0)
    smooth(o); recalc(o)
    return o

def plate(name, profile, thick, parent):
    bm = bmesh.new()
    vs = [bm.verts.new((p[0], 0, p[1])) for p in profile]
    bm.faces.new(vs)
    bmesh.ops.solidify(bm, geom=list(bm.verts) + list(bm.edges) + list(bm.faces), thickness=thick)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
    o = part(name, parent, me)
    recalc(o)
    return o

import math as _m
def _hash(x, y, z):
    h = _m.sin(x * 127.1 + y * 311.7 + z * 74.7) * 43758.5453
    return h - _m.floor(h)
def _rgb(s):
    s = s.lstrip("#"); return [int(s[i:i+2], 16) / 255 for i in (0, 2, 4)]
def _mix(a, b, t): return [a[i] + (b[i] - a[i]) * t for i in range(3)]
def _c01(v): return max(0.0, min(1.0, v))

def paint(o, base, dark, belly, top, opts=None):
    opts = opts or {}
    B = _rgb(base); D = _rgb(dark); BE = _rgb(belly)
    T = _rgb(top) if top else None
    ca = o.data.color_attributes.new(name="COLOR_0", type='BYTE_COLOR', domain='POINT')
    cols = []
    for v in o.data.vertices:
        x, y, z = v.co
        gy, gz = z, -y
        c = _mix(B, BE, _c01(opts.get("bellyY", 0) + 0.22 - gy * opts.get("grad", 0.16)))
        c = _mix(c, D, _c01((gy - opts.get("backY", 1.5)) * 0.25))
        if T: c = _mix(c, T, _c01((gy - opts.get("topY", 2.2)) * 0.4))
        c = _mix(c, D, _hash(round(x * 7), round(gy * 7), round(gz * 7)) * opts.get("speck", 0.16))
        st = opts.get("stripes")
        if st:
            w = _m.sin((x + opts.get("stripeTilt", 0) * gz) * opts.get("stripeFreq", 1.4))
            if w > 0.72: c = _mix(c, _rgb(st), _c01((w - 0.72) * 3))
        sp = opts.get("spots")
        if sp and _hash(round(x * 3), round(gy * 3), round(gz * 3)) > 0.82:
            c = _mix(c, _rgb(sp), 0.75)
        cols.append((c[0], c[1], c[2], 1.0))
    ca.data.foreach_set("color", [ch for c in cols for ch in c])

def paint_solid(o, color):
    ca = o.data.color_attributes.new(name="COLOR_0", type='BYTE_COLOR', domain='POINT')
    r_, g_, b_ = _rgb(color)
    for c in ca.data: c.color = (r_, g_, b_, 1.0)

def eye_pair(name, parent, center, r, spread, color="#f4ecd6"):
    for s in (-1, 1):
        bm = bmesh.new()
        bmesh.ops.create_uvsphere(bm, u_segments=8, v_segments=6, radius=r)
        for v in bm.verts: v.co += mathutils.Vector((center[0], center[1] + s * spread, center[2]))
        me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
        o = part(name + ("L" if s < 0 else "R"), parent, me)
        paint_solid(o, color); smooth(o); recalc(o)


# ---- patch FIX @1795268759581 ----

def spike_line(name, pts, r, parent, segs=5):
    me0 = None
    for i in range(len(pts) - 1):
        a, b = mathutils.Vector(pts[i]), mathutils.Vector(pts[i+1])
        t0, t1 = i / (len(pts) - 1), (i + 1) / (len(pts) - 1)
        r1, r2 = r * (1 - t0) + 0.002, r * (1 - t1) + 0.002
        d = b - a; L = d.length
        bm = bmesh.new()
        bmesh.ops.create_cone(bm, cap_ends=True, segments=segs, radius1=r1, radius2=r2, depth=L)
        q = mathutils.Vector((0, 0, 1)).rotation_difference(d.normalized())
        for v in bm.verts: v.co = q @ v.co + (a + d * 0.5)
        me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
        if me0 is None: me0 = me
        else:
            tmpo = bpy.data.objects.new("_j1", me0); bpy.context.collection.objects.link(tmpo)
            srco = bpy.data.objects.new("_j2", me);  bpy.context.collection.objects.link(srco)
            bpy.ops.object.select_all(action='DESELECT')
            srco.select_set(True); tmpo.select_set(True)
            bpy.context.view_layer.objects.active = tmpo
            bpy.ops.object.join()
            joined = tmpo.data
            bpy.data.objects.remove(srco, do_unlink=True)
            bpy.data.objects.remove(tmpo, do_unlink=True)
            me0 = joined
    o = part(name, parent, me0)
    smooth(o); recalc(o)
    return o


# ---- patch FIX @1795268780989 ----

def spike_line(name, pts, r, parent, segs=5):
    me0 = None
    for i in range(len(pts) - 1):
        a, b = mathutils.Vector(pts[i]), mathutils.Vector(pts[i+1])
        t0, t1 = i / (len(pts) - 1), (i + 1) / (len(pts) - 1)
        r1, r2 = r * (1 - t0) + 0.002, r * (1 - t1) + 0.002
        d = b - a; L = d.length
        bm = bmesh.new()
        bmesh.ops.create_cone(bm, cap_ends=True, segments=segs, radius1=r1, radius2=r2, depth=L)
        q = mathutils.Vector((0, 0, 1)).rotation_difference(d.normalized())
        for v in bm.verts: v.co = q @ v.co + (a + d * 0.5)
        me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
        if me0 is None: me0 = me
        else:
            tmpo = bpy.data.objects.new("_j1", me0); bpy.context.collection.objects.link(tmpo)
            srco = bpy.data.objects.new("_j2", me);  bpy.context.collection.objects.link(srco)
            bpy.ops.object.select_all(action='DESELECT')
            srco.select_set(True); tmpo.select_set(True)
            bpy.context.view_layer.objects.active = tmpo
            bpy.ops.object.join()
            joined = tmpo.data
            if srco.name in bpy.data.objects: bpy.data.objects.remove(srco, do_unlink=True)
            bpy.data.objects.remove(tmpo, do_unlink=True)
            me0 = joined
    o = part(name, parent, me0)
    smooth(o); recalc(o)
    return o


# ---- patch FIX @1795268804516 ----

def spike_line(name, pts, r, parent, segs=5):
    bm = bmesh.new()
    for i in range(len(pts) - 1):
        a, b = mathutils.Vector(pts[i]), mathutils.Vector(pts[i+1])
        t0, t1 = i / (len(pts) - 1), (i + 1) / (len(pts) - 1)
        r1, r2 = r * (1 - t0) + 0.002, r * (1 - t1) + 0.002
        d = b - a; L = d.length
        tmp = bmesh.new()
        bmesh.ops.create_cone(tmp, cap_ends=True, segments=segs, radius1=r1, radius2=r2, depth=L)
        q = mathutils.Vector((0, 0, 1)).rotation_difference(d.normalized())
        for v in tmp.verts: v.co = q @ v.co + (a + d * 0.5)
        me = bpy.data.meshes.new("_tmp"); tmp.to_mesh(me); tmp.free()
        bm.from_mesh(me)
        bpy.data.meshes.remove(me)
    me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
    o = part(name, parent, me)
    smooth(o); recalc(o)
    return o


# ---- patch FIX @1795268871617 ----

def tube(name, pts, radii, parent, subsurf=1, prev_pt=None, inflate=1.10):
    pts = [tuple(p) for p in pts]
    radii = [(max(w,0.008)*inflate, max(h,0.008)*inflate) for (w,h) in radii]
    if prev_pt is not None:
        a = mathutils.Vector(pts[0]); pv = mathutils.Vector(prev_pt)
        back = (a - pv).normalized() * 0.22
        pts.insert(0, tuple(a + back))
        radii.insert(0, tuple(c * 1.14 for c in radii[0]))
    me = bpy.data.meshes.new(name)
    me.from_pydata(pts, [(i, i+1) for i in range(len(pts)-1)], [])
    o = part(name, parent, me)
    md = o.modifiers.new("skin", 'SKIN'); md.use_smooth_shade = True
    bpy.context.view_layer.update()
    svs = o.data.skin_vertices[0].data
    svs[0].use_root = True
    for sv, (w, h) in zip(svs, radii):
        sv.radius = (w, h)
    if subsurf:
        ss = o.modifiers.new("sub", 'SUBSURF'); ss.levels = subsurf; ss.render_levels = subsurf
    bpy.context.view_layer.update()
    apply_mods(o); smooth(o); recalc(o)
    return o


# ---- patch FIX @1795268930755 ----

def mat_vcol():
    m = bpy.data.materials.get("VCOL")
    if m: return m
    m = bpy.data.materials.new("VCOL")
    m.use_nodes = True
    nt = m.node_tree
    bsdf = next(n for n in nt.nodes if n.type == "BSDF_PRINCIPLED")
    ca = nt.nodes.new("ShaderNodeVertexColor")
    try: ca.layer_name = "COLOR_0"
    except Exception: pass
    nt.links.new(ca.outputs["Color"], bsdf.inputs["Base Color"])
    return m

def assign_vcol(root_name="root"):
    import collections
    seen = collections.defaultdict(set)
    def walk(o):
        for c in o.children:
            if c.type == "MESH":
                c.data.materials.clear()
                c.data.materials.append(mat_vcol())
            walk(c)
    o = bpy.data.objects.get(root_name)
    if o: walk(o)

def tube(name, pts, radii, parent, subsurf=1, prev_pt=None, inflate=1.04, subdiv=1):
    pts = [tuple(p) for p in pts]
    radii = [(max(w,0.008)*inflate, max(h,0.008)*inflate) for (w,h) in radii]
    if prev_pt is not None:
        a = mathutils.Vector(pts[0]); pv = mathutils.Vector(prev_pt)
        back = (a - pv).normalized() * 0.22
        pts.insert(0, tuple(a + back))
        radii.insert(0, tuple(c * 1.14 for c in radii[0]))
    me = bpy.data.meshes.new(name)
    me.from_pydata(pts, [(i, i+1) for i in range(len(pts)-1)], [])
    o = part(name, parent, me)
    md = o.modifiers.new("skin", 'SKIN'); md.use_smooth_shade = True
    try: md.subdivisions = subdiv
    except Exception: pass
    bpy.context.view_layer.update()
    svs = o.data.skin_vertices[0].data
    svs[0].use_root = True
    for sv, (w, h) in zip(svs, radii):
        sv.radius = (w, h)
    if subsurf:
        ss = o.modifiers.new("sub", 'SUBSURF'); ss.levels = subsurf; ss.render_levels = subsurf
    bpy.context.view_layer.update()
    apply_mods(o); smooth(o); recalc(o)
    return o


# ---- patch FIX @1795269104990 ----

def mat_vcol():
    m = bpy.data.materials.get("VCOL")
    if m: return m
    m = bpy.data.materials.new("VCOL")
    m.use_nodes = True
    nt = m.node_tree
    bsdf = next(n for n in nt.nodes if n.type == "BSDF_PRINCIPLED")
    ca = nt.nodes.new("ShaderNodeVertexColor")
    ca.layer_name = "COLOR_0"
    nt.links.new(ca.outputs["Color"], bsdf.inputs["Base Color"])
    return m

def _resample(pts, radii, half=True):
    out_p, out_r = [], []
    for i in range(len(pts)-1):
        a, b = pts[i], pts[i+1]
        ra, rb = radii[i], radii[i+1]
        out_p.append(tuple(a)); out_r.append(tuple(ra))
        if half:
            m = tuple((a[k]+b[k])/2 for k in range(3))
            out_p.append(m)
            out_r.append(((ra[0]+rb[0])/2, (ra[1]+rb[1])/2))
    out_p.append(tuple(pts[-1])); out_r.append(tuple(radii[-1]))
    return out_p, out_r

def tube(name, pts, radii, parent, subsurf=1, prev_pt=None, inflate=1.02, dense=True):
    pts = [tuple(p) for p in pts]
    radii = [(max(w,0.008)*inflate, max(h,0.008)*inflate) for (w,h) in radii]
    if prev_pt is not None:
        a = mathutils.Vector(pts[0]); pv = mathutils.Vector(prev_pt)
        back = (a - pv).normalized() * 0.25
        pts.insert(0, tuple(a + back))
        radii.insert(0, tuple(c * 1.12 for c in radii[0]))
    if dense:
        pts, radii = _resample(pts, radii)
    me = bpy.data.meshes.new(name)
    me.from_pydata(pts, [(i, i+1) for i in range(len(pts)-1)], [])
    o = part(name, parent, me)
    md = o.modifiers.new("skin", 'SKIN'); md.use_smooth_shade = True
    bpy.context.view_layer.update()
    svs = o.data.skin_vertices[0].data
    svs[0].use_root = True
    for sv, (w, h) in zip(svs, radii):
        sv.radius = (w, h)
    if subsurf:
        ss = o.modifiers.new("sub", 'SUBSURF'); ss.levels = subsurf; ss.render_levels = subsurf
    bpy.context.view_layer.update()
    apply_mods(o); smooth(o); recalc(o)
    return o

def blob(name, balls, parent, target_faces=90, threshold=0.42):
    mb = bpy.data.metaballs.new(name)
    mb.resolution = 0.04
    mb.threshold = threshold
    for b in balls:
        e = mb.elements.new(type='ELLIPSOID')
        e.co = b[:3]; e.radius = 1.0
        e.size_x, e.size_y, e.size_z = b[3], b[4], b[5]
    mo = bpy.data.objects.new(name + "_meta", mb)
    bpy.context.collection.objects.link(mo)
    bpy.context.view_layer.update()
    ev = mo.evaluated_get(bpy.context.evaluated_depsgraph_get())
    me = bpy.data.meshes.new_from_object(ev)
    bpy.data.objects.remove(mo, do_unlink=True); bpy.data.metaballs.remove(mb)
    o = part(name, parent, me)
    sm = o.modifiers.new("sm", 'SMOOTH'); sm.factor = 0.6; sm.iterations = 3
    n = len(o.data.polygons)
    dm = o.modifiers.new("dec", 'DECIMATE'); dm.ratio = max(0.02, target_faces / max(n, 1))
    apply_mods(o); smooth(o); recalc(o)
    return o


# ---- patch FIX @1795269188442 ----

def blob(name, balls, parent, target_faces=90):
    import mathutils as _mu
    mb = bpy.data.metaballs.new(name)
    mb.resolution = 0.04
    mb.threshold = 0.55
    for b in balls:
        e = mb.elements.new(type='ELLIPSOID')
        e.co = b[:3]; e.radius = 1.0
        e.size_x, e.size_y, e.size_z = b[3], b[4], b[5]
    mo = bpy.data.objects.new(name + "_meta", mb)
    bpy.context.collection.objects.link(mo)
    bpy.context.view_layer.update()
    ev = mo.evaluated_get(bpy.context.evaluated_depsgraph_get())
    me = bpy.data.meshes.new_from_object(ev)
    bpy.data.objects.remove(mo, do_unlink=True); bpy.data.metaballs.remove(mb)
    o = part(name, parent, me)
    xs = [v.co.x for v in o.data.vertices]; ys = [v.co.y for v in o.data.vertices]; zs = [v.co.z for v in o.data.vertices]
    diag = ((max(xs)-min(xs))**2 + (max(ys)-min(ys))**2 + (max(zs)-min(zs))**2) ** 0.5
    rm = o.modifiers.new("rm", 'REMESH'); rm.mode = 'VOXELS'
    rm.voxel_size = max(diag / 14.0, 0.03)
    sm = o.modifiers.new("sm", 'SMOOTH'); sm.factor = 0.5; sm.iterations = 2
    n = len(o.data.polygons)
    dm = o.modifiers.new("dec", 'DECIMATE'); dm.ratio = max(0.02, target_faces / max(n, 1))
    apply_mods(o); smooth(o); recalc(o)
    return o


# ---- patch FIX @1795269221617 ----

def blob(name, balls, parent, target_faces=90):
    mb = bpy.data.metaballs.new(name)
    mb.resolution = 0.04
    mb.threshold = 0.55
    for b in balls:
        e = mb.elements.new(type='ELLIPSOID')
        e.co = b[:3]; e.radius = 1.0
        e.size_x, e.size_y, e.size_z = b[3], b[4], b[5]
    mo = bpy.data.objects.new(name + "_meta", mb)
    bpy.context.collection.objects.link(mo)
    bpy.context.view_layer.update()
    ev = mo.evaluated_get(bpy.context.evaluated_depsgraph_get())
    me = bpy.data.meshes.new_from_object(ev)
    bpy.data.objects.remove(mo, do_unlink=True); bpy.data.metaballs.remove(mb)
    o = part(name, parent, me)
    xs = [v.co.x for v in o.data.vertices]; ys = [v.co.y for v in o.data.vertices]; zs = [v.co.z for v in o.data.vertices]
    diag = ((max(xs)-min(xs))**2 + (max(ys)-min(ys))**2 + (max(zs)-min(zs))**2) ** 0.5
    rm = o.modifiers.new("rm", 'REMESH'); rm.mode = 'VOXEL'
    rm.voxel_size = max(diag / 14.0, 0.03)
    sm = o.modifiers.new("sm", 'SMOOTH'); sm.factor = 0.5; sm.iterations = 2
    n = len(o.data.polygons)
    dm = o.modifiers.new("dec", 'DECIMATE'); dm.ratio = max(0.02, target_faces / max(n, 1))
    apply_mods(o); smooth(o); recalc(o)
    return o


# ---- patch FIX @1795269520722 ----

def plate(name, profile, thick, parent, y_off=0.0):
    bm = bmesh.new()
    vs = [bm.verts.new((p[0], y_off, p[1])) for p in profile]
    bm.faces.new(vs)
    bmesh.ops.solidify(bm, geom=list(bm.verts) + list(bm.edges) + list(bm.faces), thickness=thick)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
    o = part(name, parent, me)
    recalc(o)
    return o


# ---- patch FIX @1795269730319 ----

def blob(name, balls, parent, target_faces=90):
    """union of ellipsoid balls via exact sphere shells + voxel remesh"""
    import bmesh as _bm
    bm = _bm.new()
    for b in balls:
        tmp = _bm.new()
        _bm.ops.create_uvsphere(tmp, u_segments=14, v_segments=10, radius=1.0)
        for v in tmp.verts:
            v.co.x *= b[3]; v.co.y *= b[4]; v.co.z *= b[5]
            v.co += _bm.verts  # placeholder no-op
        for v in tmp.verts:
            v.co += __import__("mathutils").Vector(b[:3])
        me = bpy.data.meshes.new("_tmp"); tmp.to_mesh(me); tmp.free()
        bm.from_mesh(me)
        bpy.data.meshes.remove(me)
    me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
    o = part(name, parent, me)
    xs = [v.co.x for v in o.data.vertices]; ys = [v.co.y for v in o.data.vertices]; zs = [v.co.z for v in o.data.vertices]
    diag = ((max(xs)-min(xs))**2 + (max(ys)-min(ys))**2 + (max(zs)-min(zs))**2) ** 0.5
    rm = o.modifiers.new("rm", 'REMESH'); rm.mode = 'VOXEL'
    rm.voxel_size = max(diag / 16.0, 0.02)
    sm = o.modifiers.new("sm", 'SMOOTH'); sm.factor = 0.5; sm.iterations = 2
    n = len(o.data.polygons)
    dm = o.modifiers.new("dec", 'DECIMATE'); dm.ratio = max(0.02, target_faces / max(n, 1))
    apply_mods(o); smooth(o); recalc(o)
    return o

def plate(name, profile, thick, parent, y_off=0.0):
    import bmesh as _bm
    bm = _bm.new()
    vs = [bm.verts.new((p[0], y_off, p[1])) for p in profile]
    f = bm.faces.new(vs)
    ret = _bm.ops.extrude_face_region(bm, geom=[f])
    new_verts = [g for g in ret["geom"] if isinstance(g, _bm.types.MeshVert)]
    for v in new_verts: v.co.y += thick
    _bm.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
    o = part(name, parent, me)
    recalc(o)
    return o


# ---- patch FIX @1795269748456 ----

def blob(name, balls, parent, target_faces=90):
    bm = bmesh.new()
    for b in balls:
        tmp = bmesh.new()
        bmesh.ops.create_uvsphere(tmp, u_segments=14, v_segments=10, radius=1.0)
        for v in tmp.verts:
            v.co.x *= b[3]; v.co.y *= b[4]; v.co.z *= b[5]
            v.co += mathutils.Vector(b[:3])
        me = bpy.data.meshes.new("_tmp"); tmp.to_mesh(me); tmp.free()
        bm.from_mesh(me)
        bpy.data.meshes.remove(me)
    me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
    o = part(name, parent, me)
    xs = [v.co.x for v in o.data.vertices]; ys = [v.co.y for v in o.data.vertices]; zs = [v.co.z for v in o.data.vertices]
    diag = ((max(xs)-min(xs))**2 + (max(ys)-min(ys))**2 + (max(zs)-min(zs))**2) ** 0.5
    rm = o.modifiers.new("rm", 'REMESH'); rm.mode = 'VOXEL'
    rm.voxel_size = max(diag / 16.0, 0.02)
    sm = o.modifiers.new("sm", 'SMOOTH'); sm.factor = 0.5; sm.iterations = 2
    n = len(o.data.polygons)
    dm = o.modifiers.new("dec", 'DECIMATE'); dm.ratio = max(0.02, target_faces / max(n, 1))
    apply_mods(o); smooth(o); recalc(o)
    return o


# ---- patch FIX @1795269756216 ----

def plate(name, profile, thick, parent, y_off=0.0):
    bm = bmesh.new()
    vs = [bm.verts.new((p[0], y_off, p[1])) for p in profile]
    f = bm.faces.new(vs)
    probe = bm.verts[0]
    ret = bmesh.ops.extrude_face_region(bm, geom=[f])
    moved = [g for g in ret["geom"] if type(g) is type(probe)]
    for v in moved: v.co.y += thick
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
    o = part(name, parent, me)
    recalc(o)
    return o


# ---- patch FIX @1795269762130 ----

def plate(name, profile, thick, parent, y_off=0.0):
    bm = bmesh.new()
    vs = [bm.verts.new((p[0], y_off, p[1])) for p in profile]
    f = bm.faces.new(vs)
    probe = bm.verts[0]
    ret = bmesh.ops.extrude_face_region(bm, geom=[f])
    moved = [g for g in ret["geom"] if type(g) is type(probe)]
    for v in moved: v.co.y += thick
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
    o = part(name, parent, me)
    recalc(o)
    return o


# ---- patch FIX @1795269767023 ----

def plate(name, profile, thick, parent, y_off=0.0):
    bm = bmesh.new()
    vs = [bm.verts.new((p[0], y_off, p[1])) for p in profile]
    f = bm.faces.new(vs)
    bm.verts.ensure_lookup_table()
    probe = bm.verts[0]
    ret = bmesh.ops.extrude_face_region(bm, geom=[f])
    moved = [g for g in ret["geom"] if type(g) is type(probe)]
    for v in moved: v.co.y += thick
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
    o = part(name, parent, me)
    recalc(o)
    return o


# ---- patch EXP @1795269884994 ----

def export_glb(root_name, path):
    def descend(o, acc):
        acc.append(o)
        for c in o.children: descend(c, acc)
        return acc
    bpy.ops.object.select_all(action='DESELECT')
    sel = descend(bpy.data.objects[root_name], [])
    for o in sel: o.select_set(True)
    bpy.context.view_layer.objects.active = bpy.data.objects[root_name]
    bpy.ops.export_scene.gltf(filepath=path, export_format='GLB', use_selection=True,
                              export_materials='NONE', export_apply=True, export_extras=False)
    return len(sel)


# ---- patch FIX @1795269920006 ----

def tube(name, pts, radii, parent, subsurf=1, prev_pt=None, inflate=1.02, dense=True, tuck=0.35):
    pts = [tuple(p) for p in pts]
    radii = [(max(w,0.008)*inflate, max(h,0.008)*inflate) for (w,h) in radii]
    if prev_pt is not None:
        a = mathutils.Vector(pts[0]); pv = mathutils.Vector(prev_pt)
        back = (a - pv).normalized() * tuck
        pts.insert(0, tuple(a + back))
        radii.insert(0, tuple(c * 1.10 for c in radii[0]))
    if dense:
        pts, radii = _resample(pts, radii)
    me = bpy.data.meshes.new(name)
    me.from_pydata(pts, [(i, i+1) for i in range(len(pts)-1)], [])
    o = part(name, parent, me)
    md = o.modifiers.new("skin", 'SKIN'); md.use_smooth_shade = True
    bpy.context.view_layer.update()
    svs = o.data.skin_vertices[0].data
    svs[0].use_root = True
    for sv, (w, h) in zip(svs, radii):
        sv.radius = (w, h)
    if subsurf:
        ss = o.modifiers.new("sub", 'SUBSURF'); ss.levels = subsurf; ss.render_levels = subsurf
    bpy.context.view_layer.update()
    apply_mods(o); smooth(o); recalc(o)
    return o


# ---- patch CARD @1795269962156 ----

def render_card(key, yaw=-0.62, pitch=0.16, fit=0.9):
    import mathutils
    root = bpy.data.objects["root"]
    import collections
    acc = [root]
    i = 0
    while i < len(acc):
        acc.extend(acc[i].children); i += 1
    corners = []
    for o in acc:
        if o.type != 'MESH': continue
        for c in o.bound_box:
            corners.append(o.matrix_world @ mathutils.Vector(c))
    xs=[v.x for v in corners]; ys=[v.y for v in corners]; zs=[v.z for v in corners]
    ctr = mathutils.Vector(((min(xs)+max(xs))/2, (min(ys)+max(ys))/2, (min(zs)+max(zs))/2))
    cp = math.cos(pitch)
    u = mathutils.Vector((math.sin(yaw)*cp, -math.cos(yaw)*cp, math.sin(pitch))).normalized()
    D = 60.0
    cam_data = bpy.data.cameras.new("cardcam"); cam_data.type='ORTHO'
    cam = bpy.data.objects.new("cardcam", cam_data)
    bpy.context.collection.objects.link(cam)
    q = u.to_track_quat('Z','Y')
    cam.matrix_world = mathutils.Matrix.Translation(ctr + u*D) @ q.to_matrix().to_4x4()
    inv = cam.matrix_world.inverted()
    vs = [inv @ v for v in corners]
    span_u = max(v.x for v in vs) - min(v.x for v in vs)
    span_v = max(v.y for v in vs) - min(v.y for v in vs)
    cam_data.ortho_scale = max(span_u, span_v/(180.0/320.0)) / fit
    scene = bpy.context.scene
    scene.camera = cam
    scene.render.resolution_x = 320; scene.render.resolution_y = 180
    scene.render.film_transparent = True
    scene.render.image_settings.file_format = 'PNG'
    scene.render.image_settings.color_mode = 'RGBA'
    out = "/home/dev/code/Local-AI-slop/examples/dinobrawl-3d/assets/select/%s.png" % key
    scene.render.filepath = out
    bpy.ops.render.render(write_still=True)
    bpy.data.objects.remove(cam, do_unlink=True)
    bpy.data.cameras.remove(cam_data)
    return out


# ---- patch CARD2 @1795270262901 ----

def render_card2(key, yaw=-0.62, pitch=0.16, fit=0.9, out=None):
    """pixel-identical port of the factory renderCard (flat Lambert rasterizer)."""
    import bpy, math
    if out is None:
        out = "/home/dev/code/Local-AI-slop/examples/dinobrawl-3d/assets/select/%s.png" % key
    w, h, S = 320, 180, 2
    root = bpy.data.objects["root"]
    acc, stack = [], [root]
    while stack:
        o = stack.pop(); acc.append(o); stack.extend(o.children)
    tris = []
    for o in acc:
        if o.type != "MESH": continue
        me = o.data; me.calc_loop_triangles()
        ca = me.color_attributes.get("COLOR_0")
        mw = o.matrix_world
        co = [mw @ v.co for v in me.vertices]
        cols = [tuple(ca.data[i].color) for i in range(len(me.vertices))] if ca else [(1,0,1)]*len(me.vertices)
        for lt in me.loop_triangles:
            i0, i1, i2 = lt.vertices
            v0, v1, v2 = co[i0], co[i1], co[i2]
            fn = lt.normal.normalized()
            # Blender (x fwd, y side, z up) -> factory (x fwd, y up, z side-negated)
            V = [(v.x, v.z, -v.y) for v in (v0, v1, v2)]
            FN = (fn.x, fn.z, -fn.y)
            c0, c1, c2 = cols[i0], cols[i1], cols[i2]
            C = ((c0[0]+c1[0]+c2[0])/3, (c0[1]+c1[1]+c2[1])/3, (c0[2]+c1[2]+c2[2])/3)
            tris.append((V, FN, C))
    lo = [1e9]*3; hi = [-1e9]*3
    for V, FN, C in tris:
        for v in V:
            for k in range(3):
                lo[k] = min(lo[k], v[k]); hi[k] = max(hi[k], v[k])
    mid = [(lo[k]+hi[k])/2 for k in range(3)]
    spanX, spanY, spanZ = hi[0]-lo[0], hi[1]-lo[1], hi[2]-lo[2]
    sc = min((w-16)*fit/max(spanX, spanZ), (h-16)*fit/spanY) * 0.98
    cyy, syy = math.cos(yaw), math.sin(yaw)
    cp, sp2 = math.cos(pitch), math.sin(pitch)
    def proj(p):
        x, y, z = p[0]-mid[0], p[1]-mid[1], p[2]-mid[2]
        x1 = x*cyy - z*syy; z1 = x*syy + z*cyy
        y1 = y*cp - z1*sp2
        return (x1*sc + w/2, h/2 - y1*sc, y*sp2 + z1*cp)
    ln = math.sqrt(0.35*0.35 + 0.8*0.8 + 0.5*0.5)
    L = (-0.35/ln, 0.8/ln, -0.5/ln)
    W, H = w*S, h*S
    buf = bytearray(W*H*4)
    zb = [-1e9]*(W*H)
    for V, FN, C in tris:
        d = FN[0]*L[0] + FN[1]*L[1] + FN[2]*L[2]
        sh = 0.42 + 0.58*(d if d > 0 else 0)
        rim = 1 + 0.10*abs(FN[2])
        c = tuple(min(1.0, x*sh*rim) for x in C)
        cr, cg, cb = int(c[0]*255+0.5), int(c[1]*255+0.5), int(c[2]*255+0.5)
        q = []
        for v in V:
            sx, sy, sz = proj(v)
            q.append((sx*S, sy*S, sz))
        xs = [v[0] for v in q]; ys = [v[1] for v in q]
        x0 = max(0, int(min(xs))); x1 = min(W-1, int(max(xs))+1)
        y0 = max(0, int(min(ys))); y1 = min(H-1, int(max(ys))+1)
        denom = (q[1][1]-q[2][1])*(q[0][0]-q[2][0]) + (q[2][0]-q[1][0])*(q[0][1]-q[2][1])
        if abs(denom) < 1e-6: continue
        for yy in range(y0, y1+1):
            py = yy + 0.5
            row = yy*W
            for xx in range(x0, x1+1):
                px = xx + 0.5
                a = ((q[1][1]-q[2][1])*(px-q[2][0]) + (q[2][0]-q[1][0])*(py-q[2][1]))/denom
                if a < 0: continue
                b = ((q[2][1]-q[0][1])*(px-q[2][0]) + (q[0][0]-q[2][0])*(py-q[2][1]))/denom
                if b < 0: continue
                cc2 = 1 - a - b
                if cc2 < 0: continue
                zz = a*q[0][2] + b*q[1][2] + cc2*q[2][2]
                o = row + xx
                if zz <= zb[o]: continue
                zb[o] = zz
                k = o*4
                buf[k] = cr; buf[k+1] = cg; buf[k+2] = cb; buf[k+3] = 255
    # downsample straight-alpha
    raw = bytearray(w*h*4)
    for yy in range(h):
        for xx in range(w):
            rs = gs = bs = alc = 0
            for dy in range(S):
                base = ((yy*S+dy)*W + xx*S)*4
                for dx in range(S):
                    o = base + dx*4
                    rs += buf[o]; gs += buf[o+1]; bs += buf[o+2]; alc += buf[o+3]
            n4 = S*S; o2 = (yy*w+xx)*4
            raw[o2] = rs//n4; raw[o2+1] = gs//n4; raw[o2+2] = bs//n4; raw[o2+3] = alc//n4
    img = bpy.data.images.new("cardtmp", w, h, alpha=True)
    try: img.colorspace_settings.name = "Raw"
    except Exception: pass
    import array
    px = array.array("f", bytes(len(w*h*4)*4))
    for i in range(w*h*4):
        px[i] = raw[i]/255.0
    # flip rows: raw row 0 = top, Blender pixels row 0 = bottom
    pw = array.array("f", [0.0]*(w*h*4))
    for yy in range(h):
        src = (h-1-yy)*w*4; dst = yy*w*4
        pw[dst:dst+w*4] = px[src:src+w*4]
    img.pixels = pw[:]
    img.filepath_raw = out
    img.file_format = "PNG"
    img.save()
    bpy.data.images.remove(img)
    return out


# ---- patch CARD2 @1795270306605 ----

def render_card2(key, yaw=-0.62, pitch=0.16, fit=0.9, out=None):
    """pixel-style port of the factory renderCard (flat Lambert rasterizer)."""
    import bpy, math
    if out is None:
        out = "/home/dev/code/Local-AI-slop/examples/dinobrawl-3d/assets/select/%s.png" % key
    w, h, S = 320, 180, 2
    root = bpy.data.objects["root"]
    acc, stack = [], [root]
    while stack:
        o = stack.pop(); acc.append(o); stack.extend(o.children)
    tris = []
    for o in acc:
        if o.type != "MESH": continue
        me = o.data; me.calc_loop_triangles()
        ca = me.color_attributes.get("COLOR_0")
        mw = o.matrix_world
        co = [mw @ v.co for v in me.vertices]
        cols = [tuple(ca.data[i].color) for i in range(len(me.vertices))] if ca else [(1.0,0.0,1.0)]*len(me.vertices)
        for lt in me.loop_triangles:
            i0, i1, i2 = lt.vertices
            v0, v1, v2 = co[i0], co[i1], co[i2]
            fn = lt.normal.normalized()
            V = [(v.x, v.z, -v.y) for v in (v0, v1, v2)]
            FN = (fn.x, fn.z, -fn.y)
            c0, c1, c2 = cols[i0], cols[i1], cols[i2]
            C = ((c0[0]+c1[0]+c2[0])/3.0, (c0[1]+c1[1]+c2[1])/3.0, (c0[2]+c1[2]+c2[2])/3.0)
            tris.append((V, FN, C))
    lo = [1e9]*3; hi = [-1e9]*3
    for V, FN, C in tris:
        for v in V:
            if v[0] < lo[0]: lo[0] = v[0]
            if v[1] < lo[1]: lo[1] = v[1]
            if v[2] < lo[2]: lo[2] = v[2]
            if v[0] > hi[0]: hi[0] = v[0]
            if v[1] > hi[1]: hi[1] = v[1]
            if v[2] > hi[2]: hi[2] = v[2]
    mid = [(lo[k]+hi[k])/2 for k in range(3)]
    spanX, spanY, spanZ = hi[0]-lo[0], hi[1]-lo[1], hi[2]-lo[2]
    sc = min((w-16)*fit/max(spanX, spanZ), (h-16)*fit/spanY) * 0.98
    cyy, syy = math.cos(yaw), math.sin(yaw)
    cp, sp2 = math.cos(pitch), math.sin(pitch)
    def proj(p):
        x, y, z = p[0]-mid[0], p[1]-mid[1], p[2]-mid[2]
        x1 = x*cyy - z*syy; z1 = x*syy + z*cyy
        y1 = y*cp - z1*sp2
        return (x1*sc + w/2.0, h/2.0 - y1*sc, y*sp2 + z1*cp)
    ln = (0.35*0.35 + 0.8*0.8 + 0.5*0.5) ** 0.5
    L = (-0.35/ln, 0.8/ln, -0.5/ln)
    W, H = w*S, h*S
    buf = bytearray(W*H*4)
    zb = [-1e9]*(W*H)
    for V, FN, C in tris:
        d = FN[0]*L[0] + FN[1]*L[1] + FN[2]*L[2]
        sh = 0.42 + 0.58*(d if d > 0 else 0.0)
        rim = 1.0 + 0.10*abs(FN[2])
        cr = int(min(1.0, C[0]*sh*rim)*255+0.5)
        cg = int(min(1.0, C[1]*sh*rim)*255+0.5)
        cb = int(min(1.0, C[2]*sh*rim)*255+0.5)
        q = []
        for v in V:
            sx, sy, sz = proj(v)
            q.append((sx*S, sy*S, sz))
        minx = min(q[0][0], q[1][0], q[2][0]); maxx = max(q[0][0], q[1][0], q[2][0])
        miny = min(q[0][1], q[1][1], q[2][1]); maxy = max(q[0][1], q[1][1], q[2][1])
        x0 = 0 if minx < 0 else int(minx); x1 = (W-1) if maxx > W-1 else int(maxx)+1
        y0 = 0 if miny < 0 else int(miny); y1 = (H-1) if maxy > H-1 else int(maxy)+1
        denom = (q[1][1]-q[2][1])*(q[0][0]-q[2][0]) + (q[2][0]-q[1][0])*(q[0][1]-q[2][1])
        if abs(denom) < 1e-6: continue
        qa0, qa1, qa2 = q[0], q[1], q[2]
        for yy in range(y0, y1+1):
            py = yy + 0.5
            base = (yy*W)*4
            za = qa1[1]-qa2[1]; zc = qa2[0]-qa1[0]
            a0 = za*(py-qa2[1])  # const part wrt px? a = (za*(px-q2x) + zc*(py-q2y))/den
            for xx in range(x0, x1+1):
                px = xx + 0.5
                a = (za*(px-qa2[0]) + zc*(py-qa2[1]))/denom
                if a < 0: continue
                b = ((qa2[1]-qa0[1])*(px-qa0[0]) + (qa0[0]-qa2[0])*(py-qa0[1]))/denom
                if b < 0: continue
                g = 1.0 - a - b
                if g < 0: continue
                zz = a*qa0[2] + b*qa1[2] + g*qa2[2]
                o = base + xx*4
                if zz <= zb[o//4]: continue
                zb[o//4] = zz
                buf[o] = cr; buf[o+1] = cg; buf[o+2] = cb; buf[o+3] = 255
    raw = bytearray(w*h*4)
    n4 = S*S
    for yy in range(h):
        for xx in range(w):
            rs = gs = bs = alc = 0
            for dy in range(S):
                b0 = ((yy*S+dy)*W + xx*S)*4
                for dx in range(S):
                    o = b0 + dx*4
                    rs += buf[o]; gs += buf[o+1]; bs += buf[o+2]; alc += buf[o+3]
            o2 = (yy*w+xx)*4
            raw[o2] = rs//n4; raw[o2+1] = gs//n4; raw[o2+2] = bs//n4; raw[o2+3] = alc//n4
    img = bpy.data.images.new("cardtmp", w, h, alpha=True)
    try: img.colorspace_settings.name = "Raw"
    except Exception: pass
    pw = [0.0]*(w*h*4)
    di = 0
    for yy in range(h):
        src0 = (h-1-yy)*w*4
        for i in range(w*4):
            pw[di] = raw[src0+i]/255.0; di += 1
    img.pixels = pw
    img.filepath_raw = out
    img.file_format = "PNG"
    img.save()
    bpy.data.images.remove(img)
    return out



# ---- patch FIX @tail-continuity ------------------------------------------

def flare(name, k=0.5, win=0.28, depth=0.10):
    """Bulge a tube's leading end outward around its own axis so a chain of
    tubes reads as one continuous limb/tail instead of a bead chain. Call
    after building, on each tube that plugs into the end of the one ahead."""
    import mathutils
    ob = bpy.data.objects[name]
    for m in list(ob.modifiers):                      # bake subsurf first:
        bpy.context.view_layer.objects.active = ob    # a live subsurf would
        bpy.ops.object.modifier_apply(modifier=m.name)  # soak up the bulge
    M = ob.matrix_world; Mi = M.inverted()
    ws = [M @ v.co for v in ob.data.vertices]
    xm = sum(w.x for w in ws)/len(ws)
    Cf = mathutils.Vector((0, 0, 0)); Cb = mathutils.Vector((0, 0, 0)); nf = nb = 0
    for w in ws:
        if w.x > xm: Cf += w; nf += 1
        else: Cb += w; nb += 1
    if not nf or not nb: return
    Cf /= nf; Cb /= nb
    u = (Cb - Cf).normalized() * -1.0
    proj = [(w - Cf).dot(u) for w in ws]
    pmax = max(proj); pmin = min(proj); w0 = pmax - (pmax - pmin) * win
    for v, w, p in zip(ob.data.vertices, ws, proj):
        if p <= w0: continue
        f = (p - w0) / (pmax - w0)
        foot = Cf + u * p
        w2 = foot + u * (f * depth) + (w - foot) * (1 + k * f)
        v.co = Mi @ w2
    ob.data.update()


import sys as _sys
if 'dt' not in _sys.modules or _sys.modules['dt'] is None:
    import types as _types
    _newmod = _types.ModuleType('dt')
    _newmod.__dict__.update({k: v for k, v in globals().items() if not k.startswith('__')})
    _sys.modules['dt'] = _newmod
else:
    _sys.modules['dt'].__dict__.update({k: v for k, v in globals().items()
                                        if not k.startswith('__')})


