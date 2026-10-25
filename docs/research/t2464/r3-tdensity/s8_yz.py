"""In-sample a1/a2: Excel's density error (units 2^-53 relative to the exact density at the double t) regressed on the
relative rounding errors (units) of y = fl(n/u), z = fl(tt/u), q = fl(tt/n), w = fl(1+q), tt = fl(t t), u = fl(n+tt),
per block.  Coefficients are d ln f / d ln(piece): y^(n/2) z^(1/2) predicts y n/2 and z 1/2.
usage: s8_yz.py"""
from collections import defaultdict

import mpmath
import numpy as np

import tdlib as TD

mp = mpmath.mpf
U = 2.0**-53
rows = [r for r in TD.load_probe("a1") + TD.load_probe("a2") if not r["blk"].startswith("df")]
by = defaultdict(list)
for r in rows:
    by[(r["n"], r["blk"])].append(r)
names = ["tt", "u", "q", "w", "y", "z"]
for key in sorted(by):
    rr = by[key]
    n = key[0]
    X, Y = [], []
    for r in rr:
        p = TD.pieces(r["t"], n)
        T, N = mp(r["t"]), mp(n)
        tt, u = mp(float(p["tt"])), mp(float(p["u"]))
        ideal = {"tt": T * T, "u": N + tt, "q": tt / N, "w": 1 + mp(float(p["q"])), "y": N / u, "z": tt / u}
        X.append([float((mp(float(p[k])) - ideal[k]) / ideal[k]) / U for k in names])
        ex = mpmath.exp(TD.exact_logpdf(r["t"], n))
        Y.append(float((r["excel"] - ex) / ex) / U)
    X, Y = np.array(X), np.array(Y)
    sel = [j for j in range(len(names)) if X[:, j].std() > 1e-6]
    A = np.column_stack([X[:, sel], np.ones(len(Y))])
    coef, *_ = np.linalg.lstsq(A, Y, rcond=None)
    print(f"n={n:<3g} {key[1]:18s} sd {Y.std():.2f} -> {(Y - A @ coef).std():.2f} |",
          " ".join(f"{names[j]} {c:+.2f}" for j, c in zip(sel, coef)), f"c {coef[-1]:+.2f}")
