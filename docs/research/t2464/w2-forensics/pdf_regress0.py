"""Regress Excel's pdf relative error (units 2^-53) on intermediate rounding errors, beta_exact.tsv pdf rows."""
import math, sys
import numpy as np, mpmath
from common import *

rows = [r for r in load_beta_exact() if not r["cum"] and r["a"] != 1 and r["b"] != 1]
xs = [r["x"] for r in rows]
LN = dict(zip(xs, xl("LN", xs)))
oms = [1 - x for x in xs]
LN1 = dict(zip(oms, xl("LN", oms)))
ab = sorted({(r["a"], r["b"]) for r in rows})
gl = {}
vals = sorted({v for a, b in ab for v in (a, b, a + b)})
for v, g in zip(vals, xl("GAMMALN", vals)):
    gl[v] = g
Y = []; C = {k: [] for k in ["am1_dlnx", "rnd_am1lnx", "bm1_dln1x", "rnd_bm1ln1x", "dlnB_xl"]}
grp = []
for r in rows:
    x, a, b = r["x"], r["a"], r["b"]
    X = mpmath.mpf(x)
    Lx = LN[x]; L1 = LN1[1 - x]
    e = rel_err_units(r["excel"], exact_pdf(x, a, b))
    Y.append(e)
    C["am1_dlnx"].append(float((a - 1) * (mpmath.mpf(Lx) - mpmath.log(X))) / U)
    p = (a - 1) * Lx
    C["rnd_am1lnx"].append(float(mpmath.mpf(p) - mpmath.mpf(a - 1) * mpmath.mpf(Lx)) / U)
    C["bm1_dln1x"].append(float((b - 1) * (mpmath.mpf(L1) - mpmath.log(1 - X))) / U)
    q = (b - 1) * L1
    C["rnd_bm1ln1x"].append(float(mpmath.mpf(q) - mpmath.mpf(b - 1) * mpmath.mpf(L1)) / U)
    lnb = x87("+", gl[a], x87("-", gl[b], gl[a + b]))
    C["dlnB_xl"].append(-float(mpmath.mpf(lnb) - exact_lnbeta(a, b)) / U)
    grp.append(r["case"].split("/")[-1][:20])
Y = np.array(Y); C = {k: np.array(v) for k, v in C.items()}
ok = np.abs(Y) < 200
print("n", ok.sum(), "std", Y[ok].std())
coef, s0, s1 = ols(Y[ok], {k: v[ok] for k, v in C.items()})
for k, (c, s) in coef.items(): print(f"  {k:14s} {c:8.3f} +- {s:.3f}")
print("std before", s0, "after", s1)
for g in sorted(set(grp)):
    m = ok & (np.array(grp) == g)
    if m.sum() < 30: continue
    coef, s0, s1 = ols(Y[m], {k: v[m] for k, v in C.items()})
    print(g, m.sum(), " ".join(f"{k}={c:.2f}±{s:.2f}" for k, (c, s) in coef.items()), f"std {s0:.2f}->{s1:.2f}")
