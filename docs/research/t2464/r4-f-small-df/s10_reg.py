"""Per (shape, side) least squares of Excel's D (units, relative to exact) on local rounding errors of candidate
pieces (absolute units): eLN = LN(x) - ln x, eL = L_xl(fl r) - log1p(r_exact), and the ln-x coefficient c of M-xd
(a | -b) and of r3's form (a-1 | -(b+1)). Prints slopes and residual sd. Usage: s10_reg.py maxd"""
import sys

import numpy as np

import slib as L

maxd = float(sys.argv[1])
rows = [r for r in L.load("mya") if max(r["d1"], r["d2"]) <= maxd]
U = L.U


def fr(v):
    p, q = L.LDB(v).as_integer_ratio()
    return L.mpf(p) / q


g: dict = {}
for r in rows:
    x, m, n = r["x"], r["d1"], r["d2"]
    s, rr = L.side_r(x, m, n)
    rex = L.mpf(m) * x / n if s == 1 else n / (L.mpf(m) * x)
    eLN = float((fr(L.LN(x)) - L.mpmath.log(x)) / U)
    eL = float((fr(L.L_xl(rr)) - L.mpmath.log1p(rex)) / U)
    D = L.D_units(r["excel"], r)
    g.setdefault((m, n, s, r["blk"]), []).append((D, eLN, eL))
for (m, n, s, b), v in sorted(g.items()):
    a = np.array(v)
    h = (m + n) / 2
    X = np.column_stack([np.ones(len(a)), a[:, 1], a[:, 2]])
    coef, *_ = np.linalg.lstsq(X, a[:, 0], rcond=None)
    res = a[:, 0] - X @ coef
    c = m / 2 if s == 1 else -n / 2
    print(f"({m:g},{n:g}) s{s} {b[-6:]:6s} D sd {a[:,0].std():5.2f} | slope eLN {coef[1]:+6.2f} (M-xd c {c:+.1f})"
          f" eL {coef[2]:+7.2f} (-h {-h:+.1f}) | res sd {res.std():5.2f}")
