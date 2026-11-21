
import bpy, math
import dt
for o in list(bpy.data.objects):
    if not o.name.startswith("REF_"): bpy.data.objects.remove(o, do_unlink=True)
dt.reset()
P  = ("#6f7f4a", "#333a22", "#cfc7a2", None, {"bellyY":1.2,"grad":0.18,"speck":0.18,"spots":"#4d5a2c"})
PC = ("#b06a34", "#5a3016", "#e5b268", "#e5c07a", {"bellyY":1.9,"grad":0.12,"speck":0.12})
dt.joint("root", (0,0,0))

# FL x1.7 top1.25 z.62 ; BL x-1.6 top1.75 z.72 (high hips)
legs = [("FL", 1.75, 1.34, 0.62, 0.30, 0.20), ("FR", 1.75, 1.34, -0.62, 0.30, 0.20),
        ("BL", -1.6, 1.75, 0.72, 0.36, 0.24), ("BR", -1.6, 1.75, -0.72, 0.36, 0.24)]
for n, x, top, z, r0, r1 in legs:
    dt.joint("leg"+n, (x, z, top), "root")
    lg = dt.tube("legm"+n, [(x, z, top+0.55),(x, z, top*0.55),(x, z, 0.4),(x, z, 0.24)],
                 [(r0*0.95,r0*0.95),(r0,r0),(r1*1.05,r1*1.05),(r1*0.95,r1*0.95)], "leg"+n, subsurf=1)
    ft = dt.blob("footm"+n, [(x+0.05, z, 0.26, r1*1.15, r1*1.0, r1*0.7),(x+0.3, z, 0.17, r1*0.75, r1*0.7, r1*0.45)], "leg"+n, 50)
    dt.paint(lg, *P); dt.paint(ft, *P)

# arched body: low shoulders, high hips
body = dt.tube("bodym", [(-2.75,0,1.62),(-2.1,0,1.9),(-1.2,0,2.2),(-0.2,0,2.4),(0.7,0,2.35),(1.5,0,2.05),(2.3,0,1.6)],
               [(0.30,0.32),(0.55,0.62),(0.70,0.78),(0.75,0.84),(0.66,0.70),(0.50,0.52),(0.32,0.33)], "root", subsurf=1)
dt.joint("neck", (2.3,0,1.55), "root")
nk = dt.tube("neckm", [(2.28,0,1.55),(2.6,0,1.35),(2.9,0,1.18)], [(0.28,0.27),(0.24,0.23),(0.20,0.19)], "neck", subsurf=1, prev_pt=(1.9,0,1.75))
dt.joint("head", (0.62,-0.37,0), "neck")
hd = dt.blob("headm", [(2.98,0,1.12,0.19,0.16,0.17),(3.16,0,1.06,0.16,0.13,0.13),(3.32,0,0.99,0.11,0.10,0.09)], "head", 90)
dt.eye_pair("eyesm", "head", (3.14,0,1.16), 0.045, 0.13)
dt.joint("jaw", (0.14,0,-0.2), "head")
jw = dt.tube("jawm", [(3.05,0,0.96),(3.25,0,0.92),(3.45,0,0.9)], [(0.07,0.045),(0.055,0.035),(0.02,0.014)], "jaw", subsurf=1, prev_pt=(2.95,0,1.0))

# plates: alternating double row along the arch
top = [(-2.4,1.95),(-1.7,2.35),(-0.9,2.72),(-0.1,2.92),(0.7,2.85),(1.5,2.5)]
def quadplate(u, size):
    x0, z0 = top[int(u*(len(top)-1))] if False else (top[0][0]+(top[-1][0]-top[0][0])*u, 0)
    return None
def plate_at(u, s, y_off):
    # sample arch
    import math as _m
    i = u*(len(top)-1); i0 = int(i); fr = i-i0
    a = top[min(i0,len(top)-1)]; b = top[min(i0+1,len(top)-1)]
    x = a[0]+(b[0]-a[0])*u*1.0 if False else a[0]+(b[0]-a[0])*fr
    z = a[1]+(b[1]-a[1])*fr
    w = 0.55-0.35*abs(u-0.45); hgt = 0.62-0.3*abs(u-0.45)
    prof = [(x-w*s, z-0.08), (x-w*s*0.75, z+hgt*s*0.7), (x, z+hgt*s), (x+w*s*0.8, z+hgt*s*0.55), (x+w*s, z-0.08)]
    pl = dt.plate("platesm%02d%s" % (int(u*100), "L" if y_off<0 else "R"), prof, 0.06, "root", y_off=y_off)
    dt.paint(pl, *PC)
for i in range(7):
    u = 0.08 + i*0.14
    plate_at(u, 1.0, -0.10 if i%2 else 0.10)
    plate_at(u+0.07, 0.55, 0.10 if i%2 else -0.10)

tp = [(-2.75,0,1.6),(-3.5,0,1.5),(-4.3,0,1.45),(-4.9,0,1.45)]
prev = "root"
for i in range(3):
    a, b = tp[i], tp[i+1]
    if i == 0: dt.joint("tail1", a, "root")
    else:
        pa = tp[i-1]
        dt.joint("tail"+str(i+1), (a[0]-pa[0],0,a[2]-pa[2]), prev)
    f0 = 0.34*(1-i/3)**0.7; f1 = 0.34*(1-(i+1)/3)**0.7
    mid = ((a[0]+b[0])/2, 0, (a[2]+b[2])/2 + 0.05)
    tl = dt.tube("tailm"+str(i+1), [a, mid, b],
                 [(f0*1.1,f0),( (f0+f1)/2*1.1,(f0+f1)/2),(f1+0.02,(f1+0.02)*0.9)],
                 "tail"+str(i+1), subsurf=1, prev_pt=(tp[i-1] if i else (-2.3,0,1.7)))
    dt.paint(tl, *P)
    prev = "tail"+str(i+1)
# thagomizer: two pairs of spikes on tail3
for y, tag in ((-0.15,"L"), (0.15,"R")):
    s1 = dt.spike_line("spikem"+tag+"1", [(-4.35, y, 1.5),(-4.9, y*1.9, 2.25)], 0.085, "tail3", segs=5)
    s2 = dt.spike_line("spikem"+tag+"2", [(-4.05, y*1.9, 1.45),(-4.55, y*3.0, 2.0)], 0.075, "tail3", segs=5)
    dt.paint(s1, *PC); dt.paint(s2, *PC)

for o in (body, nk, hd, jw): dt.paint(o, *P)
dt.assign_vcol("root")
tot = sum(len(p.vertices)-2 for o in bpy.data.objects if not o.name.startswith("REF_") and o.type=="MESH" for p in o.data.polygons)
print("stego built, tris:", tot)
