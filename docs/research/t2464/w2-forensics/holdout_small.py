"""Hold-out for the (0.5,0.7) density hit: pdf = EXP(((a-1)*LN(x) + (b-1)*LN(fl(1-x))) - lnB), every op stored
x87 (fl53(fl64)), lnB = LN(GAMMA(p)*(GAMMA(q)/GAMMA(p+q))) (p = min, q = max; equal to R's lbeta here) or
GL b+(a-s). Scored on the tuning blocks and on every pdf cell NOT used to find it, by shape region."""
import collections, math
import numpy as np, mpmath
from common import *
from pdf_cands import blocks
g = blocks()
pts = [(k, p) for k, v in g.items() for p in v]
xs = sorted({p[0] for _, p in pts}); ab = sorted({(p[1], p[2]) for _, p in pts})
LNx = dict(zip(xs, xl("LN", xs))); om = sorted({1 - x for x in xs}); LN1 = dict(zip(om, xl("LN", om)))
vals = sorted({v for a, b in ab for v in (a, b, a + b)})
GL = dict(zip(vals, xl("GAMMALN", vals))); GA = dict(zip(vals, xl("GAMMA", vals)))
rat = {}
for a, b in ab:
    p, q = min(a, b), max(a, b); rat[(a, b)] = x87("*", GA[p], x87("/", GA[q], GA[p + q]))
lnrat = dict(zip(ab, xl("LN", [rat[k] for k in ab])))
def model(lbsel):
    E = []
    for k, (x, a, b, e, t) in pts:
        t1 = x87("*", a - 1, LNx[x]); t2 = x87("*", b - 1, LN1[1 - x])
        lb = lnrat[(a, b)] if lbsel == "ratio" else x87("+", GL[b], x87("-", GL[a], GL[a + b]))
        E.append(x87("-", x87("+", t1, t2), lb))
    return xl("EXP", E)
for lbsel in ("ratio", "GL b+(a-s)"):
    V = model(lbsel)
    reg = collections.defaultdict(lambda: np.zeros(4, int))
    for (k, (x, a, b, e, t)), v in zip(pts, V):
        tune = k in ("xruns-2:6", "xruns-2:7")
        if a == 1 or b == 1: r = "a or b = 1"
        elif max(a, b) < 1: r = "max(a,b)<1"
        elif min(a, b) < 1 and max(a, b) <= 2: r = "min<1, max<=2"
        elif min(a, b) < 1: r = "min<1, max>2"
        elif max(a, b) <= 2: r = "1<a,b<=2"
        elif min(a, b) <= 2: r = "min in (1,2], max>2"
        else: r = "a,b>2"
        reg[("TUNE " if tune else "HOLD ") + r] += [1, v == e, ulpdiff(v, e) <= 4, within15(v, e)]
    print("lnB =", lbsel)
    for r in sorted(reg): print(f"   {r:28s} n/exact/w4/w15 = {tuple(int(z) for z in reg[r])}")
# list the hold-out max(a,b)<1 cells
V = model("ratio")
print("hold-out max(a,b)<1 cells:")
for (k, (x, a, b, e, t)), v in zip(pts, V):
    if max(a, b) < 1 and k not in ("xruns-2:6", "xruns-2:7") and a != 1 and b != 1:
        print(f"   {k:34s} x={x!r} a={a} b={b} excel={e!r} model={v!r} ulps={(v-e)/math.ulp(e):+.0f}")
