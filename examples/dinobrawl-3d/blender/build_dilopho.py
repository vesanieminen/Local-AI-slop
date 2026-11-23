
import bpy, math
import dt
for o in list(bpy.data.objects):
    if not o.name.startswith("REF_"): bpy.data.objects.remove(o, do_unlink=True)
dt.reset()
P  = ("#2f2e33", "#141416", "#efe9dc", None, {"bellyY":1.1,"grad":0.3,"speck":0.1,"stripes":"#f1ece1","stripeFreq":1.3})
FR = ("#d8542a", "#7c2b13", "#f6c65c", "#f6e08a", {"bellyY":1.6,"grad":0.12,"speck":0.12})
NAIL = "#2a2a2c"
dt.joint("root", (0,0,0))
def mpts(pts, s): return [tuple(p[:1]) + (s*p[1],) + tuple(p[2:]) for p in pts]

for s in (1, -1):
    side = "L" if s == 1 else "R"
    dt.joint("thigh"+side, (0.1, 0.24*s, 1.7), "root")
    dt.joint("shin"+side, (0.2, 0, -0.7), "thigh"+side)
    dt.joint("foot"+side, (-0.2, 0, -0.64), "shin"+side)
    dt.joint("arm"+side, (1.15, 0.2*s, 2.1), "root")
    t = dt.tube("thighm"+side, mpts([(0.05,0.24,2.0),(0.14,0.24,1.6),(0.26,0.24,1.2),(0.3,0.24,1.02)],s),
                [(0.14,0.15),(0.19,0.22),(0.15,0.18),(0.09,0.1)], "thigh"+side, subsurf=1)
    sh = dt.tube("shinm"+side, mpts([(0.3,0.24,1.06),(0.24,0.24,0.78),(0.16,0.24,0.52),(0.11,0.24,0.38)],s),
                 [(0.095,0.1),(0.08,0.085),(0.06,0.065),(0.048,0.052)], "shin"+side, subsurf=1, prev_pt=(0.29,0.24*s,1.02))
    fb = dt.blob("footm"+side, mpts([(0.06,0.24,0.38,0.13,0.11,0.11),(0.22,0.24,0.32,0.12,0.09,0.08)],s), "foot"+side, 60)
    ft = dt.tube("toem"+side, mpts([(0.1,0.24,0.34),(0.3,0.24,0.25),(0.5,0.24,0.18),(0.62,0.24,0.14)],s),
                 [(0.06,0.06),(0.06,0.054),(0.04,0.036),(0.014,0.012)], "foot"+side, subsurf=1, prev_pt=(0.05,0.24*s,0.38))
    cl = dt.spike_line("clawm"+side, mpts([(0.6,0.24,0.15),(0.72,0.24,0.03)],s), 0.05, "foot"+side)
    ar = dt.tube("armm"+side, mpts([(0.95,0.2,2.16),(1.2,0.2,2.0),(1.38,0.2,1.86),(1.48,0.2,1.78)],s),
                 [(0.07,0.07),(0.062,0.062),(0.048,0.048),(0.034,0.034)], "arm"+side, subsurf=1, prev_pt=(0.7,0.2*s,2.2))
    ac = dt.spike_line("armclawm"+side, mpts([(1.48,0.2,1.78),(1.62,0.2,1.62)],s), 0.034, "arm"+side)
    for o in (t,sh,fb,ft,ar): dt.paint(o, *P)
    for o in (cl,ac): dt.paint_solid(o, NAIL)

body = dt.tube("bodym", [(-1.4,0,2.02),(-0.9,0,2.14),(-0.3,0,2.22),(0.35,0,2.23),(0.85,0,2.15),(1.22,0,2.05)],
               [(0.18,0.20),(0.28,0.33),(0.33,0.39),(0.31,0.37),(0.25,0.29),(0.16,0.19)], "root", subsurf=1)
dt.joint("neck", (1.15,0,2.1), "root")
nk = dt.tube("neckm", [(1.2,0,2.12),(1.45,0,2.28),(1.72,0,2.42)], [(0.20,0.19),(0.165,0.16),(0.135,0.13)], "neck", subsurf=1, prev_pt=(0.85,0,2.2))
dt.joint("head", (0.75,0,0.36), "neck")
hd = dt.blob("headm", [(1.9,0,2.44,0.20,0.19,0.20),(2.1,0,2.52,0.23,0.20,0.20),(2.34,0,2.42,0.19,0.14,0.13),(2.5,0,2.34,0.11,0.09,0.075)], "head", 130)
dt.joint("jaw", (0.08,0,-0.09), "head")
jw = dt.tube("jawm", [(1.94,0,2.36),(2.2,0,2.31),(2.46,0,2.3)], [(0.1,0.07),(0.085,0.058),(0.03,0.022)], "jaw", subsurf=1, prev_pt=(1.8,0,2.4))
dt.joint("spit", (0.88,0,-0.06), "head")
dt.eye_pair("eyesm", "head", (2.2,0,2.63), 0.05, 0.19)

# twin head crest
for y in (-0.075, 0.075):
    cr = dt.plate("crestm"+("L" if y<0 else "R"), [(1.9,2.72),(2.38,2.88),(2.66,3.08),(2.5,2.84),(2.1,2.74)], 0.035, "head", y_off=y)
    dt.paint(cr, *P)

# display frill on its own joint, twin membranes
dt.joint("frill", (0.3,0,-0.05), "neck")
for y, tag in ((-0.19,"L"), (0.19,"R")):
    prof = [(1.42 + math.sin(a)*0.52, 2.0 + math.cos(a)*0.55) for a in [(-1.45 + i/12*2.9) for i in range(13)]]
    fr = dt.plate("frillm"+tag, prof, 0.03, "frill", y_off=y)
    dt.paint(fr, *FR)

tp = [(-1.4,0,2.04),(-2.3,0,2.0),(-3.15,0,1.9),(-3.9,0,1.72),(-4.5,0,1.5)]
prev = "root"
for i in range(4):
    a, b = tp[i], tp[i+1]
    if i == 0: dt.joint("tail1", a, "root")
    else:
        pa = tp[i-1]
        dt.joint("tail"+str(i+1), (a[0]-pa[0],0,a[2]-pa[2]), prev)
    f0 = 0.30*(1-i/4)**0.7; f1 = 0.30*(1-(i+1)/4)**0.7
    mid = ((a[0]+b[0])/2, 0, (a[2]+b[2])/2 + 0.04)
    tl = dt.tube("tailm"+str(i+1), [a, mid, b],
                 [(f0,f0*0.92),((f0+f1)/2,(f0+f1)/2*0.88),(f1+0.015,f1*0.88+0.012)],
                 "tail"+str(i+1), subsurf=1, prev_pt=(tp[i-1] if i else (-0.8,0,2.06)))
    dt.paint(tl, *P)
    prev = "tail"+str(i+1)
# continuous tail: sleeve each segment over the one ahead of it
for i in range(1, 5):
    dt.flare("tailm" + str(i), 0.5)

for o in (body, nk, hd, jw): dt.paint(o, *P)
dt.assign_vcol("root")
tot = sum(len(p.vertices)-2 for o in bpy.data.objects if not o.name.startswith("REF_") and o.type=="MESH" for p in o.data.polygons)
print("dilopho built, tris:", tot)
