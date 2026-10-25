"""Refine the log-form density on small-shape blocks: ln(1-x) as (i) log1p(-x) libm, (ii) Excel LN(fl(1-x)),
(iii) extended ln of the exact 1-x kept at 64 bits (x87 FYL2X without a store), (iv) the same rounded to double;
products and sums stored x87 (fl53(fl64)) or kept extended."""
import collections, math, sys, itertools
import numpy as np, mpmath
from common import *
from pdf_cands import blocks
M = mpmath.mpf
g = blocks()
keys = [k for k in g if min(g[k][0][1], g[k][0][2]) <= 2 and not k.startswith("beta_exact")]
keys += [k for k in g if k.startswith("beta_exact")]
pts_all = [p for k in keys for p in g[k]]
xs = sorted({p[0] for p in pts_all}); ab = sorted({(p[1], p[2]) for p in pts_all})
LNx = dict(zip(xs, xl("LN", xs))); om = sorted({1 - x for x in xs}); LN1 = dict(zip(om, xl("LN", om)))
vals = sorted({v for a, b in ab for v in (a, b, a + b)})
GL = dict(zip(vals, xl("GAMMALN", vals))); GA = dict(zip(vals, xl("GAMMA", vals)))
LB = {}
for a, b in ab:
    LB[(a, b)] = {"GL a+(b-s)": x87("+", GL[a], x87("-", GL[b], GL[a + b])),
                  "GL b+(a-s)": x87("+", GL[b], x87("-", GL[a], GL[a + b])),
                  "exact": float(exact_lnbeta(a, b))}
def t2v(x, b, mode):
    if mode == "log1p": return M(x87("*", b - 1, math.log1p(-x)))
    if mode == "LN(fl(1-x))": return M(x87("*", b - 1, LN1[1 - x]))
    l = fl64(mpmath.log(1 - M(x)))
    if mode == "ext ln, ext prod": return fl64(M(b - 1) * l)
    if mode == "ext ln, stored prod": return M(fl53(fl64(M(b - 1) * l)))
def t1v(x, a, mode):
    if mode == "stored": return M(x87("*", a - 1, LNx[x]))
    return fl64(M(a - 1) * M(LNx[x]))
res = collections.defaultdict(dict)
for lbn in ["GL a+(b-s)", "GL b+(a-s)", "exact"]:
    for m2 in ["log1p", "LN(fl(1-x))", "ext ln, ext prod", "ext ln, stored prod"]:
        for m1 in ["stored", "ext"]:
            for order in ["(t1+t2)-lb", "t1+(t2-lb)", "(t1-lb)+t2"]:
                E = []
                for k in keys:
                    for x, a, b, e, t in g[k]:
                        t1, t2, lb = t1v(x, a, m1), t2v(x, b, m2), M(LB[(a, b)][lbn])
                        if order == "(t1+t2)-lb": E.append(fl53(fl64(fl64(t1 + t2) - lb)))
                        elif order == "t1+(t2-lb)": E.append(fl53(fl64(t1 + fl64(t2 - lb))))
                        else: E.append(fl53(fl64(fl64(t1 - lb) + t2)))
                V = xl("EXP", E); j = 0
                for k in keys:
                    c = 0
                    for x, a, b, e, t in g[k]:
                        c += V[j] == e; j += 1
                    res[k][(lbn, m2, m1, order)] = c
for k in keys:
    best = sorted(res[k].items(), key=lambda kv: -kv[1])[:4]
    a, b = g[k][0][1], g[k][0][2]
    print(f"{k:34s} a={a:<6.4g} b={b:<6.4g} n={len(g[k]):4d} " + " | ".join(f"{v:4d} {s}" for s, v in best))
tot = collections.Counter()
for k in keys:
    for s, v in res[k].items(): tot[s] += v
print("global best:", tot.most_common(5), "n=", sum(len(g[k]) for k in keys))
