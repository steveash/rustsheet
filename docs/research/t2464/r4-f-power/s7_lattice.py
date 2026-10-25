"""s7: is Excel's F = EXP(E) for a stored double E?  Per run: fraction of cells whose Excel value is EXP_V6 of some
double (r3-loader-bd0's admissible-E set, non-empty), and the phase of ln(Excel) on the grid ulp(ln F)."""
import math
import sys

import mpmath
import numpy as np

import plib as P

R3 = P.F.R3
src, minmax = sys.argv[1], float(sys.argv[2])
for blk, rs, m, n in P.runs(src, minmax):
    hit = 0
    ph = []
    for r in rs:
        es = R3.eset(r["excel"])
        hit += bool(es)
        le = mpmath.log(mpmath.mpf(r["excel"]))
        u = math.ulp(float(le))
        ph.append(float((le / u) - mpmath.nint(le / u)))
    R3.save_cache()
    lnF = float(mpmath.log(rs[0]["exact"]))
    print(f"{blk:20s} lnF {lnF:8.2f} ulp(lnF) {math.ulp(lnF) * 2**53:5.0f} units | EXP-of-double {hit}/{len(rs)} | phase sd {np.std(ph):.3f}")
