"""In-sample a1/a2: T density against Excel's own beta densities times the Jacobian.
f_T = f_B(y; n/2, 1/2) |t| y^2 / n = f_B(z; 1/2, n/2) |t| y^2 / n = |t| f_F(t^2; 1, n).  Spread (sd, units) of
Excel's B over each composition, per df, and the bit-exact count of a few x87 orderings.  usage: s4_jac.py"""
from collections import defaultdict

import numpy as np

import tdlib as TD

L, st = TD.LDB, TD.st
rows = TD.load_probe("a1") + TD.load_probe("a2")
by = defaultdict(list)
for r in rows:
    by[r["n"]].append(r)


def comps(r):
    n, t = r["n"], abs(r["t"])
    tt = L(st(L(t) * L(t)))
    u = L(st(L(n) + tt))
    y = L(st(L(n) / u))
    F, H, G = L(r["betay"]), L(r["betaz"]), L(r["fdist"])
    return {
        "Fy: F*t*y*y/n": st(L(st(L(st(L(st(F * L(t))) * y)) * y)) / L(n)),
        "Fy: F*t/n*y*y (ext)": st(F * L(t) / L(n) * y * y),
        "Fy: F*(t*n)/(u*u) ext": st(F * L(t) * L(n) / (u * u)),
        "Fz: H*t*y*y/n": st(L(st(L(st(L(st(H * L(t))) * y)) * y)) / L(n)),
        "Fz: H*(t*n)/(u*u) ext": st(H * L(t) * L(n) / (u * u)),
        "G*t": st(G * L(t)),
    }


for n in sorted(by):
    rr = [r for r in by[n] if r["betay"] > 0]
    if len(rr) < 20:
        continue
    res = defaultdict(list)
    for r in rr:
        for k, v in comps(r).items():
            res[k].append((v, r["excel"]))
    print(f"n={n:g}")
    for k, pr in res.items():
        rel = np.array([(e - m) / e for m, e in pr]) / 2**-53
        print(f"   {k:24s} exact {sum(m == e for m, e in pr)}/{len(pr)}  mean {rel.mean():+.2f} sd {rel.std():.2f}")
