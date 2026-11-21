
import bpy
import dt
for o in list(bpy.data.objects):
    if not o.name.startswith("REF_") and o.type != 'LIGHT': bpy.data.objects.remove(o, do_unlink=True)
dt.reset()
P = ("#6c86a0", "#33424f", "#c9d6dd", None, {"bellyY":1.9,"grad":0.15,"speck":0.15,"stripes":"#3f5262","stripeFreq":1.05})
dt.joint("root", (0,0,0))
def mpts(pts, s): return [tuple(p[:1]) + (s*p[1],) + tuple(p[2:]) for p in pts]

legs = [("FL", 1.5, 2.45, 0.74, 0.36, 0.23), ("FR", 1.5, 2.45, -0.74, 0.36, 0.23),
        ("BL", -1.75, 2.65, 0.68, 0.40, 0.25), ("BR", -1.75, 2.65, -0.68, 0.40, 0.25)]
for n, x, top, z, r0, r1 in legs:
    dt.joint("leg"+n, (x, z, top), "root")
    lg = dt.tube("legm"+n, [(x, z, top+0.45),(x, z, top*0.6),(x, z, 0.5),(x, z, 0.28)],
                 [(r0*0.9,r0),(r0,r0*1.05),(r1*1.05,r1),(r1*0.95,r1*0.95)], "leg"+n, subsurf=1)
    ft = dt.blob("footm"+n, [(x+0.05, z, 0.3, r1*1.2, r1*1.05, r1*0.8),(x+0.35, z, 0.2, r1*0.85, r1*0.8, r1*0.55)], "leg"+n, 55)
    dt.paint(lg, *P); dt.paint(ft, *P)

body = dt.tube("bodym", [(-3.3,0,2.45),(-2.4,0,2.8),(-1.2,0,3.0),(0.2,0,3.1),(1.2,0,2.95),(1.9,0,2.7)],
               [(0.35,0.4),(0.75,0.85),(0.95,1.05),(1.0,1.1),(0.85,0.95),(0.55,0.6)], "root", subsurf=1)
# neck: 3 articulated links + head
nc = [(-0.6,0,3.05),(0.9,0,3.0),(1.75,0,2.8),(3.1,0,3.3),(4.4,0,3.55),(5.45,0,3.5)]
# link1 starts inside body
joints_pts = [(1.7,0,2.75),(3.1,0,3.3),(4.4,0,3.62),(5.5,0,3.78)]
prev = "root"
for i in range(3):
    a, b = joints_pts[i], joints_pts[i+1]
    if i == 0: dt.joint("neck1", a, "root")
    else:
        pa = joints_pts[i-1]
        dt.joint("neck"+str(i+1), (a[0]-pa[0],0,a[2]-pa[2]), prev)
    r0 = 0.46*(1-i/3)**0.55; r1 = 0.46*(1-(i+1)/3)**0.55
    mid = ((a[0]+b[0])/2, 0, (a[2]+b[2])/2 + 0.06)
    nk = dt.tube("neckm"+str(i+1), [a, mid, b],
                 [(r0,r0*0.95),((r0+r1)/2,(r0+r1)/2*0.95),(r1+0.02,r1*0.95+0.02)],
                 "neck"+str(i+1), subsurf=1, tuck=0.55, prev_pt=(joints_pts[i-1] if i else (0.9,0,3.05)))
    dt.paint(nk, *P)
    prev = "neck"+str(i+1)
dt.joint("head", (5.9-5.5,0,3.72-3.78), "neck3")
hd = dt.blob("headm", [(5.75,0,3.71,0.2,0.17,0.18),(6.0,0,3.67,0.18,0.14,0.14),(6.25,0,3.57,0.12,0.10,0.09)], "head", 90)
dt.eye_pair("eyesm", "head", (5.91,0,3.82), 0.045, 0.145)
dt.joint("jaw", (0.3,0,-0.2), "head")
jw = dt.tube("jawm", [(5.77,0,3.55),(6.05,0,3.51),(6.29,0,3.49)], [(0.08,0.05),(0.06,0.04),(0.022,0.015)], "jaw", subsurf=1, prev_pt=(5.65,0,3.61))

tp = [(-3.3,0,2.42),(-4.5,0,2.38),(-5.8,0,2.28),(-7.0,0,2.08),(-8.1,0,1.83),(-9.0,0,1.48),(-9.7,0,1.08)]
prev = "root"
for i in range(6):
    a, b = tp[i], tp[i+1]
    if i == 0: dt.joint("tail1", a, "root")
    else:
        pa = tp[i-1]
        dt.joint("tail"+str(i+1), (a[0]-pa[0],0,a[2]-pa[2]), prev)
    f0 = 0.46*(1-i/6)**0.6; f1 = 0.46*(1-(i+1)/6)**0.6
    mid = ((a[0]+b[0])/2, 0, (a[2]+b[2])/2 + 0.04)
    tl = dt.tube("tailm"+str(i+1), [a, mid, b],
                 [(f0*1.05,f0*0.9),((f0+f1)/2*1.05,(f0+f1)/2*0.9),(f1+0.015,(f1+0.015)*0.9)],
                 "tail"+str(i+1), subsurf=1, tuck=0.55, prev_pt=(tp[i-1] if i else (-2.6,0,2.55)))
    dt.paint(tl, *P)
    prev = "tail"+str(i+1)

for o in (body, hd, jw): dt.paint(o, *P)
dt.assign_vcol("root")
tot = sum(len(p.vertices)-2 for o in bpy.data.objects if not o.name.startswith("REF_") and o.type=="MESH" for p in o.data.polygons)
print("diplodocus built, tris:", tot)
n = dt.export_glb("root", "/home/dev/code/Local-AI-slop/examples/dinobrawl-3d/assets/diplodocus.glb")
print("exported", n, "nodes")
