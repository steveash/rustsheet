"""T-3567 r5-fdf1 c0: which (x, y) and which tail does F at d1 = 1 compute? Exact-math candidates."""
import csv, math, sys, mpmath
from pathlib import Path
mpmath.mp.dps = 50
HERE = Path(__file__).resolve().parent
rows = []
for r in csv.DictReader(open(HERE.parent / "cells.tsv"), delimiter="\t"):
    if r["fn"] not in ("FDIST", "F.DIST.RT", "F.DIST"): continue
    p = r["args"].split("|")
    try:
        x, m, n = float(p[0]), float(p[1]), float(p[2])
        ex = float(r["excel"])
    except ValueError:
        continue
    if r["fn"] == "F.DIST" and p[3] not in ("True", "1.0", "-1.0", "100.0", "0.5"): continue
    if r["fn"] == "F.DIST" and p[3] in ("False", "0.0"): continue
    m, n = math.trunc(m), math.trunc(n)
    if m != 1 or n < 3 or x <= 0: continue
    upper = r["fn"] != "F.DIST"
    rows.append((r["fn"], x, n, upper, ex))
def ulp(v): return math.ulp(v)
def I(a, b, x): return mpmath.betainc(a, b, 0, x, regularized=True)
cands = {}
def C(name):
    def d(f): cands[name] = f; return f
    return d
@C("y=fl(n/fl(n+x)), exact I_y(n/2,1/2)")
def _(x, n):
    s = n + x; y = n / s; return I(n / 2, 0.5, y)
@C("xx=fl(x/fl(n+x)), 1-I_xx(1/2,n/2) exact")
def _(x, n):
    s = n + x; xx = x / s; return 1 - I(0.5, n / 2, xx)
@C("xx, RN(1-RN(I_xx))")
def _(x, n):
    s = n + x; xx = x / s; return mpmath.mpf(1 - float(I(0.5, n / 2, xx)))
@C("y=fl(1-xx)")
def _(x, n):
    s = n + x; xx = x / s; return I(n / 2, 0.5, 1 - xx)
@C("r=fl(n/x), y=fl(r/fl(1+r))")
def _(x, n):
    r = n / x; return I(n / 2, 0.5, r / (1 + r))
@C("y=fl(1/fl(1+fl(x/n)))")
def _(x, n):
    return I(n / 2, 0.5, 1 / (1 + x / n))
@C("T: t=fl(sqrt x), y=fl(n/fl(n+fl(t*t)))")
def _(x, n):
    t = math.sqrt(x); s = n + t * t; return I(n / 2, 0.5, n / s)
@C("exact x")
def _(x, n):
    return I(n / 2, 0.5, mpmath.mpf(n) / (n + mpmath.mpf(x)))
print(len(rows), "cells (d1=1, d2>=3)")
for name, f in cands.items():
    ex_ = w4 = 0; errs = []
    for fn, x, n, up, e in rows:
        v = f(x, n)
        if not up: v = 1 - v
        vf = float(v)
        d = (vf - e) / ulp(e)
        errs.append(d)
        ex_ += vf == e; w4 += abs(d) <= 4
    errs.sort()
    print(f"{name:45s} exact {ex_:4d} w4 {w4:4d}  median err {errs[len(errs)//2]:+.1f} ulp")
