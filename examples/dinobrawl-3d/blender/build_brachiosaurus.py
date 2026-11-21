
import bpy
import dt
for o in list(bpy.data.objects):
    if not o.name.startswith("REF_") and o.type != 'LIGHT': bpy.data.objects.remove(o, do_unlink=True)
dt.reset()
P = ("#8d8f6a", "#3d4030", "#ded6b4", None, {"bellyY":2.6,"grad":0.15,"speck":0.15,"spots":"#6a6c4a"})
dt.joint("root", (0,0,0))

legs = [("FL", 1.6, 3.1, 0.92, 0.5, 0.32), ("FR", 1.6, 3.1, -0.92, 0.5, 0.32),
        ("BL", -1.8, 3.3, 0.86, 0.55, 0.34), ("BR", -1.8, 3.3, -0.86, 0.55, 0.34)]
for n, x, top, z, r0, r1 in legs:
    dt.joint("leg"+n, (x, z, top), "root")
    lg = dt.tube("legm"+n, [(x, z, top+0.55),(x, z, top*0.55),(x, z, 0.55),(x, z, 0.3)],
                 [(r0*0.92,r0*0.92),(r0,r0*1.05),(r1*1.08,r1*1.08),(r1*0.98,r1*0.98)], "leg"+n, subsurf=1)
    ft = dt.blob("footm"+n, [(x+0.05, z, 0.32, r1*1.25, r1*1.1, r1*0.85),(x+0.42, z, 0.22, r1*0.9, r1*0.85, r1*0.6)], "leg"+n, 60)
    dt.paint(lg, *P); dt.paint(ft, *P)

body = dt.tube("bodym", [(-3.2,0,3.35),(-2.3,0,3.85),(-1.0,0,4.15),(0.3,0,4.25),(1.3,0,4.15),(2.0,0,3.9)],
               [(0.45,0.5),(0.95,1.1),(1.2,1.35),(1.25,1.4),(1.05,1.2),(0.7,0.78)], "root", subsurf=1)

# steep up-swept neck: 3 joints
np_ = [(1.9,0,4.05),(2.9,0,5.05),(3.7,0,5.95),(4.35,0,6.55)]
prev = "root"
for i in range(3):
    a, b = np_[i], np_[i+1]
    if i == 0: dt.joint("neck1", a, "root")
    else:
        pa = np_[i-1]
        dt.joint("neck"+str(i+1), (a[0]-pa[0],0,a[2]-pa[2]), prev)
    r0 = 0.70*(1-i/3)**0.55; r1 = 0.70*(1-(i+1)/3)**0.55
    mid = ((a[0]+b[0])/2, 0, (a[2]+b[2])/2 + 0.10)
    nk = dt.tube("neckm"+str(i+1), [a, mid, b],
                 [(r0,r0*0.98),((r0+r1)/2,(r0+r1)/2*0.96),(r1+0.03,r1*0.98+0.03)],
                 "neck"+str(i+1), subsurf=1, tuck=0.55, prev_pt=(np_[i-1] if i else (0.9,0,4.2)))
    dt.paint(nk, *P)
    prev = "neck"+str(i+1)
dt.joint("head", (4.72-4.35,0,6.85-6.55), "neck3")
hd = dt.blob("headm", [(4.5,0,6.82,0.26,0.24,0.26),(4.78,0,6.86,0.24,0.2,0.21),(5.05,0,6.72,0.17,0.15,0.14)], "head", 110)
cr = dt.plate("crestm", [(4.45,7.02),(4.72,7.42),(4.98,7.35),(5.18,7.0),(4.8,6.98)], 0.08, "head")
dt.paint(cr, *P)
dt.eye_pair("eyesm", "head", (4.82,0,7.0), 0.055, 0.2)
dt.joint("jaw", (0.28,0,-0.32), "head")
jw = dt.tube("jawm", [(4.6,0,6.6),(4.9,0,6.55),(5.15,0,6.52)], [(0.11,0.07),(0.09,0.055),(0.03,0.02)], "jaw", subsurf=1, prev_pt=(4.45,0,6.66))

tp = [(-3.2,0,3.3),(-4.3,0,3.18),(-5.4,0,2.93),(-6.3,0,2.58),(-7.1,0,2.18)]
prev = "root"
for i in range(4):
    a, b = tp[i], tp[i+1]
    if i == 0: dt.joint("tail1", a, "root")
    else:
        pa = tp[i-1]
        dt.joint("tail"+str(i+1), (a[0]-pa[0],0,a[2]-pa[2]), prev)
    f0 = 0.60*(1-i/4)**0.62; f1 = 0.60*(1-(i+1)/4)**0.62
    mid = ((a[0]+b[0])/2, 0, (a[2]+b[2])/2 + 0.06)
    tl = dt.tube("tailm"+str(i+1), [a, mid, b],
                 [(f0*1.05,f0*0.92),((f0+f1)/2*1.05,(f0+f1)/2*0.92),(f1+0.02,(f1+0.02)*0.92)],
                 "tail"+str(i+1), subsurf=1, tuck=0.55, prev_pt=(tp[i-1] if i else (-2.4,0,3.5)))
    dt.paint(tl, *P)
    prev = "tail"+str(i+1)

for o in (body, hd, jw): dt.paint(o, *P)
dt.assign_vcol("root")
tot = sum(len(p.vertices)-2 for o in bpy.data.objects if not o.name.startswith("REF_") and o.type=="MESH" for p in o.data.polygons)
print("brachiosaurus built, tris:", tot)
n = dt.export_glb("root", "/home/dev/code/Local-AI-slop/examples/dinobrawl-3d/assets/brachiosaurus.glb")
print("exported", n)
