"""Beta density, log form: pdf = EXP(E), E assembled from t1 = (a-1)*ln x, t2 = (b-1)*ln(1-x) and lnB, with
variants of each piece. Reports per block the best variants (bit-exact counts) and a global table.
Run: python pdf_logform.py [min_shape_max]   (default: blocks whose min(a,b) <= 2)"""
import collections, math, sys, itertools
import numpy as np, mpmath
from common import *
from toms import rcall, call
from pdf_cands import blocks
M = mpmath.mpf
g = blocks()
pts_all = [p for v in g.values() for p in v]
xs = sorted({p[0] for p in pts_all}); ab = sorted({(p[1], p[2]) for p in pts_all})
LNx = dict(zip(xs, xl("LN", xs)))
om = sorted({1 - x for x in xs}); LN1 = dict(zip(om, xl("LN", om)))
vals = sorted({v for a, b in ab for v in (a, b, a + b)})
GL = dict(zip(vals, xl("GAMMALN", vals))); GA = dict(zip(vals, xl("GAMMA", vals)))
rlb = dict(zip(ab, rcall("lbeta", [(a, b, 0, 0) for a, b in ab])))
tlb = dict(zip(ab, call("betaln", [(a, b, 0, 0) for a, b in ab])))
LB = {}
for a, b in ab:
    p, q = min(a, b), max(a, b)
    d = {"GL a+(b-s)": x87("+", GL[a], x87("-", GL[b], GL[a + b])),
         "GL b+(a-s)": x87("+", GL[b], x87("-", GL[a], GL[a + b])),
         "GL (a+b)-s": x87("-", x87("+", GL[a], GL[b]), GL[a + b]),
         "R lbeta": rlb[(a, b)], "toms betaln": tlb[(a, b)], "exact": float(exact_lnbeta(a, b))}
    d["_ratio p*(q/s)"] = x87("*", GA[p], x87("/", GA[q], GA[p + q]))
    d["_ratio (p*q)/s"] = x87("/", x87("*", GA[p], GA[q]), GA[p + q])
    d["_ratio a*(b/s)"] = x87("*", GA[a], x87("/", GA[b], GA[a + b]))
    LB[(a, b)] = d
rk = [k for k in LB[ab[0]] if k.startswith("_")]
lnr = xl("LN", [LB[k][r] for k in ab for r in rk])
i = 0
for k in ab:
    for r in rk:
        LB[k]["LN(GAMMA " + r[7:] + ")"] = lnr[i]; i += 1
    for r in rk: del LB[k][r]
lbnames = list(LB[ab[0]])
def assemble(t1, t2, lb, order):
    if order == "(t1+t2)-lb": return x87("-", x87("+", t1, t2), lb)
    if order == "t1+(t2-lb)": return x87("+", t1, x87("-", t2, lb))
    if order == "(t1-lb)+t2": return x87("+", x87("-", t1, lb), t2)
    if order == "t2+(t1-lb)": return x87("+", t2, x87("-", t1, lb))
orders = ["(t1+t2)-lb", "t1+(t2-lb)", "(t1-lb)+t2"]
specs = list(itertools.product(lbnames, ["LN", "log1p", "LN(1-x)"][1:], orders))
specs = [(lbn, l1, o) for lbn, l1, o in specs]
res = collections.defaultdict(dict)
for spec in specs:
    lbn, l1m, order = spec
    E, idx = [], []
    for k, pts in g.items():
        for j, (x, a, b, e, t) in enumerate(pts):
            t1 = x87("*", a - 1, LNx[x])
            l1 = math.log1p(-x) if l1m == "log1p" else LN1[1 - x]
            t2 = x87("*", b - 1, l1)
            E.append(assemble(t1, t2, LB[(a, b)][lbn], order)); idx.append(k)
    V = xl("EXP", E)
    cnt = collections.defaultdict(lambda: np.zeros(2, int))
    j = 0
    for k, pts in g.items():
        for (x, a, b, e, t) in pts:
            cnt[k] += [1, V[j] == e]; j += 1
    for k in cnt: res[k][spec] = cnt[k]
for k in sorted(res):
    best = sorted(res[k].items(), key=lambda kv: -kv[1][1])[:3]
    a, b = g[k][0][1], g[k][0][2]
    print(f"{k:38s} a={a:<6.4g} b={b:<6.4g} n={best[0][1][0]:4d} " + " | ".join(f"{v[1]:4d} {s}" for s, v in best))
