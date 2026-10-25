"""s13: per plateau of Excel's stored LN(x), the mean of est = (Excel - m1 model) + rho(fl(c*LN(x))), i.e. Excel's
stored power term minus c*LN(x) plus a smooth offset, and the step of c*LN(x) between plateaus (units)."""
import sys

import mpmath
import numpy as np

import cands as C
import m1_fixed as M
import plib as P

LDB, S, ex = C.LDB, C.S, C.ex
src, want = sys.argv[1], sys.argv[2]
rows, m, n = P.run_rows(src, want)
pl = []
for r in rows:
    x = r["x"]
    s, e = C.pieces(x, m, n)
    side1 = s["mx"] <= n
    c, sg = (s["a"], 1) if side1 else (s["b"], -1)
    p = LDB(c) * s["Lx"]
    rho = sg * float((ex(S(p)) - ex(p)) * P.U)
    res = P.D_units(r) - float((M.lnF_model(x, m, n, "xl") - mpmath.log(r["exact"])) * P.U)
    key = float(s["Lx"])
    if pl and pl[-1][0] == key:
        pl[-1][2].append(res + rho)
    else:
        pl.append((key, float(ex(p) * P.U), [res + rho]))
print(want, "c*LN ulp units", np.spacing(float(pl[0][1] / P.U)) * float(P.U))
prev = None
for key, pu, v in pl:
    print(f"LN {key!r:22s} n={len(v):2d} est {np.mean(v):8.1f} (sd {np.std(v):4.1f})  d(cL) {0 if prev is None else pu - prev:7.1f}")
    prev = pu
