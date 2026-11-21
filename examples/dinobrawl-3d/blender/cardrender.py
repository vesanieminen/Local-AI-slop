
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
