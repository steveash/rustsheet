"""s12: at each cell of one run, the residual (Excel - m1) beside the position of the exact product c*LN(x) on the
stored grid (fraction of an ulp from the rounding midpoint) and the Excel LN(x) vs exact ln x difference."""
import sys

import mpmath
import numpy as np

import cands as C
import m1_fixed as M
import plib as P

ex = C.ex
src, want = sys.argv[1], sys.argv[2]
rows, m, n = P.run_rows(src, want)
for r in rows[:80]:
    x = r["x"]
    s, e = C.pieces(x, m, n)
    side1 = s["mx"] <= n
    c = s["a"] if side1 else s["b"]
    p = mpmath.mpf(c) * ex(s["Lx"])
    u = mpmath.mpf(np.spacing(float(p)))
    frac = float(p / u - mpmath.floor(p / u))
    pe = mpmath.mpf(c) * e["lx"]
    frace = float(pe / u - mpmath.floor(pe / u))
    D = P.D_units(r)
    Dm = float((M.lnF_model(x, m, n, "xl") - mpmath.log(r["exact"])) * P.U)
    eL = float((ex(s["Lx"]) - e["lx"]) * P.U)
    print(f"{D - Dm:8.1f}  frac(c*LN) {frac:.3f}  frac(c*ln exact) {frace:.3f}  eLN {eL:+.2f}")
