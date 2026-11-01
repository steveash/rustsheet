"""T-3570 r5-bpser-lg: product compositions with a free per-a0 constant K (the shape factor 1/(a B)), against the
EXP(alx - L) form, scored by the per-a0 ceiling (best count of the 8 cells over K within +-W ulps of the median of
the cells' implied K). X is x^a computed four ways; s is w3's series.
  pow:   X = x^a correctly rounded                    explna: X = EXP(fl(a LN x))  (Excel EXP, LN)
  pow64: X = fl64(x^a) then stored (same as pow)      expx:   X = EXP of the extended a*LN(x)
Products: "XK.s" = fl(fl(X K) s), "Xs.K" = fl(fl(X s) K), "X.Ks" = fl(X fl(K s)).
usage: kforms.py [W=40] [split=tune] [centers=0.15,0.3]"""
import math
import statistics
import sys
from collections import Counter, defaultdict

import mpmath
from mpmath import mpf

import pin
from lib import EXP, EXPX, LN, fl64, sx

kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
W = int(kw.get("W", "40"))
SPLIT = kw.get("split", "tune")
CENTERS = set(kw.get("centers", "0.15,0.3").split(","))


def X(form, a, x):
    if form == "pow":
        with mpmath.workdps(40):
            return float(mpf(x) ** mpf(a))
    if form == "explna":
        return EXP(sx("*", a, LN(x)))
    return EXPX(fl64(mpf(a) * mpf(LN(x))))


PRODS = {"XK.s": lambda X, K, s: sx("*", sx("*", X, K), s), "Xs.K": lambda X, K, s: sx("*", sx("*", X, s), K),
         "X.Ks": lambda X, K, s: sx("*", X, sx("*", K, s))}
pts = [p for p in pin.load() if f"{p['a0']:.2f}".rstrip("0") in CENTERS or f"{p['a0']:.2f}" in CENTERS]
pts = [p for p in pts if SPLIT == "all" or pin.hold(p["a0"]) == (SPLIT == "hold")]
agg = defaultdict(Counter)
for p in pts:
    key = f"{p['a0']:.2f}"
    ss = [pin.lib.Model.series_w3(p["a0"], p["b0"], x) for x in p["xs"]]
    for xf in ("pow", "explna", "expx"):
        xv = [X(xf, p["a0"], x) for x in p["xs"]]
        k0 = statistics.median(e / (X_ * s) for (x, e), X_, s in zip(p["cells"], xv, ss))
        ks = [k0]
        for _ in range(W):
            ks = [math.nextafter(ks[0], -math.inf)] + ks + [math.nextafter(ks[-1], math.inf)]
        for pn, f in PRODS.items():
            best = max(sum(f(X_, K, s) == e for (x, e), X_, s in zip(p["cells"], xv, ss)) for K in ks)
            agg[key][f"{xf}:{pn}"] += best
            agg["all"][f"{xf}:{pn}"] += best
    agg[key]["cells"] += 8
    agg["all"]["cells"] += 8
cols = [f"{xf}:{pn}" for xf in ("pow", "explna", "expx") for pn in PRODS]
print(f"split={SPLIT} W={W} cells " + " ".join(f"{c:>12s}" for c in cols))
for key in sorted(agg):
    print(f"{key:5s} {agg[key]['cells']:12d} " + " ".join(f"{agg[key][c]:12d}" for c in cols))
