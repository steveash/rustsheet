"""T-3570 r5-bpser-lg: compositions of E = EXP(alx - L) with the series s, scored by the per-a0 ceiling over g
(g within +-R ulps of Excel's GAMMALN(a0), so L is free per a0). alx = fl(a LN x), s = w3's series, L as pin.py.
  base   fl(EXPX(fl64(alx - L)) * s)                     (r2-bpser-small's model)
  z53    fl(EXP(fl(alx - L)) * s)
  lns53  EXP(fl(fl(alx + LN(s)) - L))                    (ln s folded into the exponent, stored)
  lns64  EXPX(fl64(fl64(alx - L) + LN(s)))
  lnsb   EXPX(fl64(alx - fl(L - LN(s))))
  diva   fl(fl(E / a) * fl(a * s))
  mula   fl(fl(E * fl(a * s)) / a)
usage: zforms2.py [R=2] [split=tune] [centers=0.15,0.3]"""
import math
import sys
from collections import Counter, defaultdict

from mpmath import mpf

import pin
from lib import EXP, EXPX, LN, fl64, sx

kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
R = int(kw.get("R", "2"))
SPLIT = kw.get("split", "tune")
CENTERS = set(kw.get("centers", "0.15,0.3").split(","))


def forms(a, L, alx, s):
    E = EXPX(fl64(mpf(alx) - mpf(L)))
    return {
        "base": sx("*", E, s),
        "z53": sx("*", EXP(sx("-", alx, L)), s),
        "lns53": EXP(sx("-", sx("+", alx, LN(s)), L)),
        "lns64": EXPX(fl64(fl64(mpf(alx) - mpf(L)) + mpf(LN(s)))),
        "lnsb": EXPX(fl64(mpf(alx) - mpf(sx("-", L, LN(s))))),
        "diva": sx("*", sx("/", E, a), sx("*", a, s)),
        "mula": sx("/", sx("*", E, sx("*", a, s)), a),
    }


NAMES = ("base", "z53", "lns53", "lns64", "lnsb", "diva", "mula")
pts = [p for p in pin.load() if f"{p['a0']:.2f}".rstrip("0") in CENTERS or f"{p['a0']:.2f}" in CENTERS]
pts = [p for p in pts if SPLIT == "all" or pin.hold(p["a0"]) == (SPLIT == "hold")]
agg = defaultdict(Counter)
for p in pts:
    a = p["a0"]
    gs = [p["gl"]]
    for _ in range(R):
        gs = [math.nextafter(gs[0], -math.inf)] + gs + [math.nextafter(gs[-1], math.inf)]
    pre = [(sx("*", a, LN(x)), pin.lib.Model.series_w3(a, p["b0"], x), e) for x, e in p["cells"]]
    best = Counter()
    for g in gs:
        L = sx("+", g, sx("-", p["glb"], p["glab"]))
        hit = Counter()
        for alx, s, e in pre:
            for nm, v in forms(a, L, alx, s).items():
                hit[nm] += v == e
        for nm in NAMES:
            best[nm] = max(best[nm], hit[nm])
    key = f"{a:.2f}"
    for k in (key, "all"):
        agg[k].update(best)
        agg[k]["cells"] += 8
print(f"split={SPLIT} R={R} cells " + " ".join(f"{n:>6s}" for n in NAMES))
for key in sorted(agg):
    print(f"{key:5s} {agg[key]['cells']:10d} " + " ".join(f"{agg[key][n]:6d}" for n in NAMES))
