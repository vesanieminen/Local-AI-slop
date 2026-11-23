
import bpy
import dt
for o in list(bpy.data.objects):
    if not o.name.startswith("REF_"): bpy.data.objects.remove(o, do_unlink=True)
dt.reset()
P = ("#9a7c46", "#3b2f1d", "#dac9a2", None, {"bellyY":0.55,"grad":0.3,"speck":0.15,"stripes":"#463518","stripeFreq":2.0})
NAIL = "#eae2ca"
dt.joint("root", (0,0,0))
def mpts(pts, s): return [tuple(p[:1]) + (s*p[1],) + tuple(p[2:]) for p in pts]

for s in (1, -1):
    side = "L" if s == 1 else "R"
    dt.joint("thigh"+side, (0.05, 0.17*s, 1.2), "root")
    dt.joint("shin"+side, (0.15, 0, -0.5), "thigh"+side)
    dt.joint("foot"+side, (-0.14, 0, -0.44), "shin"+side)
    dt.joint("arm"+side, (0.85, 0.145*s, 1.5), "root")
    t = dt.tube("thighm"+side, mpts([(0.02,0.17,1.56),(0.10,0.17,1.35),(0.18,0.17,1.1),(0.23,0.17,0.9),(0.24,0.17,0.76)],s),
                [(0.10,0.11),(0.14,0.16),(0.155,0.17),(0.10,0.12),(0.07,0.08)], "thigh"+side, subsurf=1)
    sh = dt.tube("shinm"+side, mpts([(0.24,0.17,0.8),(0.17,0.17,0.6),(0.10,0.17,0.42),(0.06,0.17,0.3)],s),
                 [(0.072,0.078),(0.06,0.063),(0.048,0.05),(0.038,0.04)], "shin"+side, subsurf=1, prev_pt=(0.23,0.17*s,0.78))
    fb = dt.blob("footm"+side, mpts([(0.0,0.17,0.3,0.10,0.085,0.085),(0.14,0.17,0.26,0.10,0.075,0.07),(0.26,0.17,0.22,0.08,0.06,0.055)],s), "foot"+side, 60)
    ft = dt.tube("toem"+side, mpts([(0.05,0.17,0.28),(0.22,0.17,0.21),(0.36,0.17,0.16),(0.46,0.17,0.12)],s),
                 [(0.05,0.05),(0.05,0.045),(0.034,0.03),(0.012,0.01)], "foot"+side, subsurf=1, prev_pt=(0.0,0.17*s,0.3))
    cl = dt.spike_line("clawm"+side, mpts([(0.44,0.17,0.13),(0.55,0.17,0.02)],s), 0.04, "foot"+side)
    sc = dt.spike_line("sicklem"+side, mpts([(0.14,0.17,0.23),(0.22,0.17,0.35),(0.26,0.17,0.43)],s), 0.03, "foot"+side)
    ar = dt.tube("armm"+side, mpts([(0.55,0.145,1.6),(0.8,0.15,1.46),(0.98,0.15,1.32),(1.06,0.15,1.26)],s),
                 [(0.055,0.055),(0.05,0.05),(0.035,0.035),(0.024,0.024)], "arm"+side, subsurf=1, prev_pt=(0.35,0.145*s,1.62))
    ac = dt.spike_line("armclawm"+side, mpts([(1.06,0.15,1.26),(1.2,0.15,1.1)],s), 0.024, "arm"+side)
    for o in (t,sh,fb,ft,ar): dt.paint(o, *P)
    for o in (cl,sc,ac): dt.paint_solid(o, NAIL)

body = dt.tube("bodym", [(-1.1,0,1.42),(-0.7,0,1.49),(-0.2,0,1.53),(0.3,0,1.53),(0.7,0,1.48),(0.98,0,1.4)],
               [(0.15,0.16),(0.24,0.27),(0.29,0.31),(0.28,0.30),(0.22,0.24),(0.12,0.14)], "root", subsurf=1)
dt.joint("neck", (0.85,0,1.5), "root")
nk = dt.tube("neckm", [(0.92,0,1.53),(1.12,0,1.65),(1.34,0,1.76)], [(0.155,0.15),(0.12,0.12),(0.098,0.092)], "neck", subsurf=1, prev_pt=(0.6,0,1.55))
dt.joint("head", (0.57,0,0.26), "neck")
hd = dt.blob("headm", [(1.42,0,1.76,0.17,0.165,0.17),(1.6,0,1.85,0.2,0.16,0.17),(1.84,0,1.75,0.17,0.125,0.115),(1.98,0,1.68,0.12,0.09,0.075)], "head", 110)
dt.joint("jaw", (0.065,0,-0.075), "head")
jw = dt.tube("jawm", [(1.42,0,1.63),(1.68,0,1.6),(1.94,0,1.57)], [(0.08,0.06),(0.065,0.047),(0.022,0.017)], "jaw", subsurf=1, prev_pt=(1.28,0,1.68))
dt.eye_pair("eyesm", "head", (1.63,0,1.94), 0.04, 0.145)
cr = dt.plate("crestm", [(1.34,1.99),(1.08,2.06),(0.82,1.93),(1.06,1.88)], 0.05, "head")

tp = [(-1.15,0,1.43),(-1.85,0,1.41),(-2.5,0,1.33),(-3.1,0,1.19),(-3.65,0,1.0)]
prev = "root"
for i in range(4):
    a, b = tp[i], tp[i+1]
    if i == 0: dt.joint("tail1", (-1.05,0,1.42), "root")
    else:
        pa = tp[i-1]
        dt.joint("tail"+str(i+1), (tp[i][0]-pa[0],0,tp[i][2]-pa[2]), prev)
    f0 = 0.24*(1-i/4)**0.7; f1 = 0.24*(1-(i+1)/4)**0.7
    mid = ((a[0]+b[0])/2, 0, (a[2]+b[2])/2 + 0.03)
    tl = dt.tube("tailm"+str(i+1), [a, mid, b],
                 [(f0,f0*0.9),((f0+f1)/2,(f0+f1)/2*0.85),(f1+0.012,f1*0.85+0.01)],
                 "tail"+str(i+1), subsurf=1, prev_pt=(tp[i-1] if i else (-0.65,0,1.46)))
    dt.paint(tl, *P)
    prev = "tail"+str(i+1)

# continuous tail: sleeve each segment over the one ahead of it
for i in range(1, 5):
    dt.flare("tailm" + str(i), 0.5)

for o in (body, nk, hd, jw, cr): dt.paint(o, *P)
dt.assign_vcol("root")
tot = sum(len(p.vertices)-2 for o in bpy.data.objects if not o.name.startswith("REF_") and o.type=="MESH" for p in o.data.polygons)
print("raptor v6, tris:", tot)
