"""T-3570 r5-bpser-lg: which stored intermediate caps the per-a0 ceiling? Series and product variants, each scored by
the per-a0 ceiling over g (best count of the 8 cells for g within +-R ulps of Excel's GAMMALN(a0)), tune half only
unless split=hold. Variants (all start from fl(1/a), forward, stop when s stops changing):
  w3      c, t and s stored after every op (r2-bpser-small's model)
  sse     plain doubles, no x87 double rounding
  sreg    s kept in an x87 register (fl64 adds), stored once at the end; c and t stored
  creg    c kept in a register (fl64), t and s stored
  allreg  c, t and s all in registers, s stored at the end
  treg    t = c/(a+n) not stored before the add into s (fl64 then the stored add)
  sprod   s from sreg left unstored into the product: I = fl53(fl64(E * s64))
usage: sforms.py [R=2] [split=tune|hold|all] [centers=0.15,0.3]"""
import math
import sys
from collections import Counter, defaultdict

from mpmath import mpf

import pin
from lib import LN, fl53, fl64, sx, EXPX

kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
R = int(kw.get("R", "2"))
SPLIT = kw.get("split", "tune")
CENTERS = set(kw.get("centers", "0.15,0.3,0.4,0.5,0.6,0.7").split(","))


def fac(b, n, x, reg):
    if reg:
        return fl64(fl64(fl64(mpf(0.5) - fl64(mpf(b) / n)) + mpf(0.5)) * x)
    return mpf(sx("*", sx("+", sx("-", 0.5, sx("/", b, n)), 0.5), x))


def series(a, b, x, form):
    if form == "w3":
        return mpf(pin.lib.Model.series_w3(a, b, x))
    if form == "sse":
        s, c, n = 1.0 / a, 1.0, 0.0
        while True:
            n += 1.0
            c = c * ((0.5 - b / n) + 0.5) * x
            s2 = s + c / (a + n)
            if s2 == s:
                return mpf(s)
            s = s2
    creg = form in ("creg", "allreg")
    sreg = form in ("sreg", "allreg", "sprod")
    treg = form in ("treg", "allreg")
    s = mpf(sx("/", 1.0, a)) if not form == "allreg" else fl64(mpf(1) / a)
    c, n = mpf(1), 0.0
    for _ in range(100000):
        n += 1.0
        c = fl64(c * fac(b, n, x, creg)) if creg else mpf(sx("*", float(c), float(fac(b, n, x, False))))
        if not creg:
            c = mpf(float(c))
        t = fl64(c / fl64(mpf(a) + n)) if treg else mpf(sx("/", float(c), sx("+", a, n)))
        s2 = fl64(s + t) if sreg else mpf(sx("+", float(s), float(t)))
        if s2 == s:
            break
        s = s2
    return s if form == "sprod" else mpf(fl53(s))


def predict(p, g, x, s, form):
    L = sx("+", g, sx("-", p["glb"], p["glab"]))
    alx = mpf(sx("*", p["a0"], LN(x)))
    e = EXPX(fl64(alx - mpf(L)))
    return fl53(fl64(mpf(e) * s)) if form == "sprod" else sx("*", e, float(s))


FORMS = ("w3", "sse", "sreg", "creg", "allreg", "treg", "sprod")
pts = [p for p in pin.load() if f"{p['a0']:.2f}".rstrip("0") in CENTERS or f"{p['a0']:.2f}" in CENTERS]
pts = [p for p in pts if SPLIT == "all" or pin.hold(p["a0"]) == (SPLIT == "hold")]
agg = defaultdict(Counter)
for p in pts:
    gs = [p["gl"]]
    for _ in range(R):
        gs = [math.nextafter(gs[0], -math.inf)] + gs + [math.nextafter(gs[-1], math.inf)]
    key = f"{p['a0']:.2f}"
    for form in FORMS:
        ss = [series(p["a0"], p["b0"], x, form) for x in p["xs"]]
        best = max(sum(predict(p, g, x, s, form) == e for (x, e), s in zip(p["cells"], ss)) for g in gs)
        agg[key][form] += best
        agg["all"][form] += best
    agg[key]["cells"] += 8
    agg["all"]["cells"] += 8
print(f"split={SPLIT} R={R}   " + " ".join(f"{f:>7s}" for f in FORMS))
for key in sorted(agg):
    print(f"{key:5s} {agg[key]['cells']:5d}  " + " ".join(f"{agg[key][f]:7d}" for f in FORMS))
