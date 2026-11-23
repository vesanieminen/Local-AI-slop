import bpy, math
import dt
for o in list(bpy.data.objects):
    if not o.name.startswith("REF_"): bpy.data.objects.remove(o, do_unlink=True)
dt.reset()
P  = ("#2f6478","#1c3c4c","#cfd6c6","#24506a",{"bellyY":1.05,"grad":0.22,"speck":0.06})
FIN= ("#27506b","#16303d","#a8bcaa",None,{"bellyY":0.9,"grad":0.2})
T  = ("#e8e2cc","#b5aa8c","#f6f1e2",None,{})
dt.joint("root",(0,0,0))

# ---- tail: 5 joints, the swim motor (smoother taper) ----
tp = [(-1.3,0,1.0),(-2.0,0,0.98),(-2.75,0,0.95),(-3.45,0,0.9),(-4.1,0,0.82)]
rad = lambda i: 0.15+0.27*(1-i/5)**0.95
prev = "root"
for i in range(5):
    a, b = tp[i], tp[i+1] if i < 4 else (-4.75,0,0.74)
    if i == 0: dt.joint("tail1", a, "root")
    else:
        pa = tp[i-1]; dt.joint("tail"+str(i+1),(a[0]-pa[0],0,a[2]-pa[2]),prev)
    f0 = rad(i); f1 = rad(i+1)
    mid = ((a[0]+b[0])/2,0,(a[2]+b[2])/2+0.03)
    tl = dt.tube("tailm"+str(i+1),[a,mid,b],
        [(f0*1.12,f0*0.95),((f0+f1)/2*1.12,(f0+f1)/2*0.95),(f1+0.03,(f1+0.03)*0.9)],
        "tail"+str(i+1), subsurf=1, tuck=0.55, prev_pt=(tp[i-1] if i else (-0.8,0,1.0)))
    dt.paint(tl, *P)
    prev = "tail"+str(i+1)
# continuous tail: sleeve each segment over the one ahead of it
for i in range(1, 6):
    dt.flare("tailm" + str(i), 0.5)

# fluke: vertical fan fin with a notch (hypocercal)
fl = dt.plate("flukem",[(-3.7,1.55),(-5.05,2.0),(-4.82,1.05),(-4.98,0.45),(-3.85,0.55)], 0.09, "tail5", y_off=0.0)
dt.paint(fl, *FIN)

# ---- body: barrel that gets deepest at the hips ----
body = dt.tube("bodym",[(-1.5,0,0.98),(-0.6,0,1.02),(0.5,0,1.08),(1.5,0,1.14),(2.3,0,1.18),(3.05,0,1.14)],
    [(0.30,0.26),(0.52,0.60),(0.60,0.68),(0.58,0.64),(0.46,0.50),(0.33,0.32)], "root", subsurf=1)
dt.joint("neck",(3.0,0,1.12),"root")
nk = dt.tube("neckm",[(2.95,0,1.13),(3.3,0,1.15),(3.6,0,1.16)],
    [(0.34,0.33),(0.30,0.30),(0.26,0.27)], "neck", subsurf=1, tuck=0.55, prev_pt=(2.6,0,1.16))
dt.joint("head",(0.45,0,0.02),"neck")
hd = dt.blob("headm",[(3.7,0,1.16,0.30,0.26,0.24),(4.05,0,1.12,0.24,0.20,0.17),(4.35,0,1.05,0.15,0.12,0.10)],"head",110)
dt.eye_pair("eyesm","head",(3.72,0,1.28),0.055,0.21,color='#101418')
dt.joint("jaw",(0.5,0,-0.10),"head")
jw = dt.tube("jawm",[(3.85,0,0.98),(4.15,0,0.93),(4.45,0,0.88)],
    [(0.20,0.20),(0.14,0.14),(0.06,0.06)], "jaw", subsurf=1, prev_pt=(3.6,0,1.02))

# ---- teeth: conical rows, uppers on the skull, lowers on the jaw ----
for i,(ux,uz) in enumerate([(3.78,1.02),(3.98,0.98),(4.18,0.93),(4.35,0.88)]):
    for y,tag in ((0.14-0.02*i,"L"),(-0.14+0.02*i,"R")):
        t1 = dt.spike_line("teethmU%d%s"%(i,tag),[(ux,y,uz),(ux+0.03,y*1.15,uz-0.17)],0.042,"head",segs=4)
        dt.paint(t1, *T)
for i,(lx,lz) in enumerate([(3.82,0.96),(4.02,0.91),(4.22,0.86)]):
    for y,tag in ((0.12-0.02*i,"L"),(-0.12+0.02*i,"R")):
        t2 = dt.spike_line("teethmL%d%s"%(i,tag),[(lx,y,lz),(lx+0.02,y*1.15,lz+0.15)],0.036,"jaw",segs=4)
        dt.paint(t2, *T)

# ---- four paddles: fore flippers sweep back, hind flippers smaller ----
for tag, s in (("L",1),("R",-1)):
    dt.joint("flip"+tag,(1.9,0.28*s,0.74),"root")
    fp = dt.blob("flipper"+tag+"m",[
        (1.72,0.45*s,0.74, 0.34,0.30,0.095),
        (1.50,0.88*s,0.70, 0.30,0.32,0.072),
        (1.27,1.28*s,0.65, 0.24,0.28,0.056),
        (1.05,1.60*s,0.60, 0.16,0.20,0.042)], "flip"+tag, 70)
    dt.paint(fp, *FIN)
    dt.joint("flip2"+tag,(-0.33,0.22*s,0.68),"root")
    f2 = dt.blob("flipper2"+tag+"m",[
        (-0.42,0.40*s,0.68, 0.26,0.24,0.065),
        (-0.62,0.66*s,0.63, 0.20,0.22,0.048),
        (-0.80,0.88*s,0.58, 0.13,0.15,0.036)], "flip2"+tag, 50)
    dt.paint(f2, *FIN)

for o in (body, nk, hd, jw): dt.paint(o, *P)
dt.assign_vcol("root")
tot = sum(len(p.vertices)-2 for o in bpy.data.objects if not o.name.startswith("REF_") and o.type=="MESH" for p in o.data.polygons)
print("mosasaurus built, tris:", tot)
