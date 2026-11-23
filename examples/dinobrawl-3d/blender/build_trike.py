
import bpy, math
import dt
for o in list(bpy.data.objects):
    if not o.name.startswith("REF_"): bpy.data.objects.remove(o, do_unlink=True)
dt.reset()
P  = ("#8a7a58", "#453a28", "#d8cdae", None, {"bellyY":1.05,"grad":0.2,"speck":0.16,"spots":"#5c4f37"})
FR = ("#9a6a4a", "#4b2f20", "#e0a96a", "#c8623a", {"bellyY":1.5,"grad":0.14,"speck":0.12})
HN = ("#efe6cd", "#8f8364", "#fffaf0", None, {"bellyY":2.0,"grad":0.1,"speck":0.05})
dt.joint("root", (0,0,0))

legs = [("FL", 1.3, 1.52, 0.66, 0.33, 0.22), ("FR", 1.3, 1.52, -0.66, 0.33, 0.22),
        ("BL", -1.3, 1.45, 0.60, 0.36, 0.24), ("BR", -1.3, 1.45, -0.60, 0.36, 0.24)]
for n, x, top, z, r0, r1 in legs:
    dt.joint("leg"+n, (x, z, top), "root")
    lg = dt.tube("legm"+n, [(x, z, top+0.4),(x, z, top*0.62),(x, z, 0.4),(x, z, 0.24)],
                 [(r0*0.9,r0*0.9),(r0*0.98,r0*0.98),(r1*1.05,r1*1.05),(r1*0.95,r1*0.95)], "leg"+n, subsurf=1)
    ft = dt.blob("footm"+n, [(x+0.05, z, 0.26, r1*1.15, r1*1.0, r1*0.75),(x+0.28, z, 0.18, r1*0.8, r1*0.75, r1*0.5)], "leg"+n, 50)
    dt.paint(lg, *P); dt.paint(ft, *P)

body = dt.tube("bodym", [(-2.2,0,1.48),(-1.4,0,1.75),(-0.5,0,1.9),(0.45,0,1.97),(1.1,0,1.9),(1.5,0,1.82)],
               [(0.34,0.32),(0.60,0.58),(0.72,0.70),(0.74,0.71),(0.60,0.56),(0.44,0.44)], "root", subsurf=1)
dt.joint("neck", (1.45,0,1.82), "root")
nk = dt.tube("neckm", [(1.42,0,1.84),(1.72,0,1.92),(2.0,0,1.96)], [(0.42,0.40),(0.36,0.34),(0.30,0.28)], "neck", subsurf=1, prev_pt=(1.05,0,1.9))
dt.joint("head", (0.8,0,0.14), "neck")
hd = dt.blob("headm", [(1.9,0,2.0,0.32,0.36,0.32),(2.25,0,2.05,0.34,0.36,0.33),(2.6,0,1.95,0.26,0.25,0.24),(2.85,0,1.82,0.17,0.18,0.15)], "head", 150)
dt.eye_pair("eyesm", "head", (2.55,0,2.18), 0.075, 0.27)
dt.joint("jaw", (0.28,0,-0.42), "head")
jw = dt.tube("jawm", [(2.4,0,1.78),(2.72,0,1.68),(3.0,0,1.6)], [(0.13,0.08),(0.10,0.065),(0.045,0.032)], "jaw", subsurf=1, prev_pt=(2.2,0,1.86))
dt.paint(jw, *HN)
prof = [(1.82 - math.cos(a)*0.92, 1.96 + math.sin(a)*1.02) for a in [(-1.45 + i/16*2.9) for i in range(17)]]
fr = dt.plate("frillm", prof, 0.16, "head")
dt.paint(fr, *FR)
for y in (-0.34, 0.34):
    hn = dt.spike_line("hornm"+("L" if y>0 else "R"), [(2.45, y, 2.3),(3.05, y*1.45, 2.98),(3.28, y*1.7, 3.22)], 0.11, "head", segs=6)
    dt.paint(hn, *HN)
nh = dt.spike_line("nosehornm", [(2.8,0,1.92),(3.3,0,2.2),(3.46,0,2.34)], 0.10, "head", segs=6)
dt.paint(nh, *HN)

tp = [(-2.2,0,1.44),(-2.85,0,1.32),(-3.4,0,1.12),(-3.85,0,0.92)]
prev = "root"
for i in range(3):
    a, b = tp[i], tp[i+1]
    if i == 0: dt.joint("tail1", a, "root")
    else:
        pa = tp[i-1]
        dt.joint("tail"+str(i+1), (a[0]-pa[0],0,a[2]-pa[2]), prev)
    f0 = 0.38*(1-i/3)**0.7; f1 = 0.38*(1-(i+1)/3)**0.7
    mid = ((a[0]+b[0])/2, 0, (a[2]+b[2])/2)
    tl = dt.tube("tailm"+str(i+1), [a, mid, b],
                 [(f0*1.15,f0),( (f0+f1)/2*1.15,(f0+f1)/2),(f1+0.02,(f1+0.02)*0.9)],
                 "tail"+str(i+1), subsurf=1, prev_pt=(tp[i-1] if i else (-1.7,0,1.5)))
    dt.paint(tl, *P)
    prev = "tail"+str(i+1)
# continuous tail: sleeve each segment over the one ahead of it
for i in range(1, 4):
    dt.flare("tailm" + str(i), 0.5)

for o in (body, nk, hd): dt.paint(o, *P)
dt.assign_vcol("root")
tot = sum(len(p.vertices)-2 for o in bpy.data.objects if not o.name.startswith("REF_") and o.type=="MESH" for p in o.data.polygons)
print("trike v2, tris:", tot)
