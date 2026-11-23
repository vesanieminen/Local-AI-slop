import bpy
import dt

for o in list(bpy.data.objects):
    if not o.name.startswith("REF_"):
        bpy.data.objects.remove(o, do_unlink=True)
dt.reset()

# Human — the little two-legged islanders of the village. Joints the game
# animates: legL/legR (hip), armL/armR (shoulder), head, spear (a child of
# armR; the game hides it on unarmed villagers). Forward is +x. Hip/shoulder
# swings are rotations about Y: negative = limb forward. Mesh names follow
# the house rules (bodym = torso column, legm*/armm* overlap it so limbCheck
# passes). Built and pose-tested in live Blender in stages; stage order here
# matches how it was inspect-fixed with look:
#   1 legs+torso, 2 arms+head, 3 belt+spear, then walk/thrust/ride poses.
SKIN  = ("#e3b089", "#9d6b47", "#9d6b47", None, {"bellyY": 1.2, "grad": 0.10, "speck": 0.08})
PANTS = ("#6a5644", "#31281e", "#2c241c", None, {"bellyY": 0.4, "grad": 0.10, "speck": 0.10})
TUNIC = ("#a8623e", "#59341f", "#7d4b2e", None, {"bellyY": 0.6, "grad": 0.10, "speck": 0.10})
BELT  = ("#4a3423", "#2e2015", "#2e2015", None, {"bellyY": 0})
SOLE  = ("#3c2f24", "#241c14", "#241c14", None, {"bellyY": 0})
HAIR  = ("#33241a", "#1d130c", "#1d130c", None, {"bellyY": 0})
WOOD  = ("#8a6a3c", "#55401f", "#55401f", None, {"bellyY": 0, "grad": 0.10})
STONE = ("#c8c2b4", "#7c7466", "#7c7466", None, {"bellyY": 0, "grad": 0.10})


def mpts(pts, s):
    return [tuple(p[:1]) + (s * p[1],) + tuple(p[2:]) for p in pts]


dt.joint("root", (0, 0, 0))

# ---- stage 1: legs + torso ----
for s in (1, -1):
    side = "L" if s == 1 else "R"
    dt.joint("leg" + side, (0.0, 0.09 * s, 0.94), "root")
    lg = dt.tube("legm" + side, mpts([(0.01, 0.09, 1.02), (0.01, 0.09, 0.78),
                                      (0.02, 0.09, 0.52), (0.02, 0.09, 0.26),
                                      (0.02, 0.09, 0.07)], s),
                 [(0.085, 0.09), (0.07, 0.075), (0.055, 0.06), (0.045, 0.05),
                  (0.04, 0.045)], "leg" + side, subsurf=1)
    ft = dt.blob("footm" + side, mpts([(0.06, 0.09, 0.045, 0.115, 0.05, 0.045),
                                       (0.17, 0.09, 0.03, 0.07, 0.045, 0.03)], s),
                 "leg" + side, 60)
    dt.paint(lg, *PANTS)
    dt.paint(ft, *SOLE)
body = dt.tube("bodym", [(0.0, 0, 0.90), (0.0, 0, 1.06), (0.0, 0, 1.24), (0.0, 0, 1.40)],
               [(0.125, 0.11), (0.135, 0.12), (0.15, 0.13), (0.145, 0.125)], "root", subsurf=1)
chest = dt.blob("chestm", [(0.01, 0, 1.33, 0.125, 0.165, 0.125)], "root", 80)
dt.paint(body, *TUNIC)
dt.paint(chest, *TUNIC)

# ---- stage 2: arms + head ----
for s in (1, -1):
    side = "L" if s == 1 else "R"
    dt.joint("arm" + side, (0.0, 0.16 * s, 1.40), "root")
    am = dt.tube("armm" + side, mpts([(0.0, 0.15, 1.46), (0.02, 0.20, 1.24),
                                      (0.045, 0.215, 1.02), (0.055, 0.205, 0.90)], s),
                 [(0.055, 0.055), (0.045, 0.045), (0.035, 0.035), (0.028, 0.028)],
                 "arm" + side, subsurf=1)
    hd = dt.blob("handm" + side, mpts([(0.055, 0.20, 0.87, 0.045, 0.04, 0.05)], s),
                 "arm" + side, 40)
    dt.paint(am, *TUNIC)
    dt.paint(hd, *SKIN)
dt.joint("head", (0.0, 0, 1.50), "root")
nk = dt.tube("neckm", [(0.0, 0, 1.40), (0.0, 0, 1.52)], [(0.048, 0.048), (0.045, 0.045)],
             "head", subsurf=1, prev_pt=(0.0, 0, 1.30))
hd = dt.blob("headm", [(0.02, 0, 1.65, 0.115, 0.115, 0.125), (0.07, 0, 1.585, 0.085, 0.095, 0.085)],
             "head", 110)
hr = dt.blob("hairm", [(-0.03, 0, 1.70, 0.12, 0.12, 0.095), (-0.07, 0, 1.61, 0.10, 0.115, 0.105)],
             "head", 90)
dt.eye_pair("eyesm", "head", (0.115, 0, 1.665), 0.018, 0.088, color="#1d1712")
dt.paint(nk, *SKIN)
dt.paint(hd, *SKIN)
dt.paint(hr, *HAIR)

# ---- stage 3: belt + spear through the right fist ----
bt = dt.tube("beltm", [(0.0, 0, 0.935), (0.0, 0, 0.995)], [(0.148, 0.135), (0.142, 0.13)],
             "root", subsurf=1, prev_pt=(0.0, 0, 0.86))
dt.paint(bt, *BELT)
# spear pivot sits in the right hand at (0.055, -0.205, 0.90)
dt.joint("spear", (0.055, -0.045, -0.455), "armR")
sp = dt.tube("spearm", [(0.055, -0.205, 0.66), (0.06, -0.205, 1.20), (0.065, -0.205, 1.74),
                        (0.065, -0.205, 1.98)],
             [(0.022, 0.022), (0.024, 0.024), (0.022, 0.022), (0.016, 0.016)],
             "spear", subsurf=1)
tp = dt.spike_line("tipm", [(0.065, -0.205, 1.98), (0.07, -0.205, 2.24)], 0.04, "spear", segs=6)
dt.paint(sp, *WOOD)
dt.paint(tp, *STONE)

dt.assign_vcol("root")
tot = sum(len(p.vertices) - 2 for o in bpy.data.objects
          if not o.name.startswith("REF_") and o.type == "MESH" for p in o.data.polygons)
print("humans built, tris:", tot)
