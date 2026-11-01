"""T-3570 r5-bpser-lg: gl vs g1ln53 in the a0's best g set (lgbest.py's ceiling), split by the side of the center
(a0 below, at or above it), per center and b0. At 0.5 the sides are GAMMALN's K3 (x = fl(1 + a0) <= 1.5) and its
(1.5, 2] rational.
usage: side.py [R=4]"""
import math
import sys
from collections import Counter, defaultdict

import pin
from lib import LN, sx

kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
R = int(kw.get("R", "4"))
agg = defaultdict(Counter)
for p in pin.load():
    a0 = p["a0"]
    c = round(a0, 2)
    side = "below" if a0 < c else ("at" if a0 == c else "above")
    g = p["gl"]
    for _ in range(R):
        g = math.nextafter(g, -math.inf)
    score = {}
    for _ in range(2 * R + 1):
        score[g] = sum(pin.predict(p, g, x) == e for x, e in p["cells"])
        g = math.nextafter(g, math.inf)
    top = max(score.values())
    best = {g for g, s in score.items() if s == top}
    g1 = sx("-", pin.K.gamln1(a0), LN(a0))
    t = agg[(c, side)]
    t["n"] += 1
    t["differ"] += g1 != p["gl"]
    t["gl"] += p["gl"] in best and g1 not in best
    t["g1ln53"] += g1 in best and p["gl"] not in best
    t["both"] += g1 in best and p["gl"] in best and g1 != p["gl"]
    t["off"] += round((g1 - p["gl"]) / math.ulp(p["gl"]))
print("center side   n differ gl-only g1ln53-only both  sum(g1ln53-gl ulps)")
for k in sorted(agg):
    t = agg[k]
    print(f"{k[0]:5.2f}  {k[1]:6s} {t['n']:3d} {t['differ']:6d} {t['gl']:7d} {t['g1ln53']:11d} {t['both']:4d} {t['off']:+6d}")
