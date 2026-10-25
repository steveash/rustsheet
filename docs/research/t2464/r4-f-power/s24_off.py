"""s24: for one run under fl(fl(K' + delta + H) + P) - ln x (x87), list the plateaus whose residual is off the mode,
with the exact product c*LN(x)'s position on its grid and the first sum's position."""
import sys
from fractions import Fraction

import mpmath
import numpy as np

import cands as C
import m10_sse as M10
import plib as P
import s3_order

src, want, dl = sys.argv[1], sys.argv[2], int(sys.argv[3])
rows = [r for r in P.load(src) if r["blk"] == want]
m, n = rows[0]["d1"], rows[0]["d2"]
D = np.array([P.D_units(r) for r in rows])
K = s3_order.kprime(m, n, m * rows[0]["x"] <= n) + mpmath.mpf(dl) / P.U
pre = M10.prep(rows, m, n, "x87", 0)
d = []
for (Pv, H, lx, e), Dv in zip(pre, D):
    d.append(Dv - float((M10.rnd(M10.rnd(K + H, "x87") + Pv, "x87") - lx - e) * P.U))
d = np.array(d)
vals, cnt = np.unique(np.round(d), return_counts=True)
mv = vals[np.argmax(cnt)]
for i, r in enumerate(rows):
    if abs(d[i] - mv) > 3:
        s, _ = C.pieces(r["x"], m, n)
        side1 = s["mx"] <= n
        c = s["a"] if side1 else s["b"]
        p = Fraction(c) * Fraction(float(s["Lx"]))
        u = Fraction(np.spacing(float(p)))
        S1 = M10.rnd(K + pre[i][1], "x87")
        tot = S1 + pre[i][0]
        ut = mpmath.mpf(np.spacing(float(tot)))
        print(f"cell {i} res-mode {d[i] - mv:+.0f} frac(cL) {float(p / u % 1):.4f} sum {float(tot):.6f} frac(sum) {float(tot / ut % 1):.4f}")
