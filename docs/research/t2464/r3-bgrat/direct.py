"""T-2512 r3-bgrat: Excel's direct bgrat cells (probe t2512-r3-bgrat-split) against bgrat models.

Models of D = BETA.DIST(x, a, b0) (direct bgrat branch, w = 0):
  M0      r2-tdist FINAL kernel (TOMS bgrat in doubles, Excel's EXP/LN/GAMMALN/log1p)
  CR      exact F at the double lnx
And the error distribution of D against F in ulps of D, per (a, b0) shape group."""
from __future__ import annotations

import math
import statistics as st
from collections import defaultdict

import mpmath

import bscore as score
from common import sl, split_kernel, xlk

K, rows = split_kernel()
res = [xlk.bx.kernel(a, b0, x, 1.0 - x, K.c) for a, b0, x, _c, _d in rows]
F = {(c["a"], c["b"], c["x"]): c["F"] for c in score.cells()}
grp = defaultdict(list)
for (a, b0, x, case, dx), (w, _w1, _br) in zip(rows, res):
    f = F[(a, b0, x)]
    u = float((mpmath.mpf(dx) - f) / mpmath.mpf(math.ulp(dx)))
    grp[(case.replace("t2464-", ""), round(a, 1), round(b0, 3))].append((w, dx, float(f), u))
print("| case | a~ | b0~ | n | M0 ex/w4/w15 | CR ex/w4/w15 | D - F (ulp of D) median, sd |")
print("|---|---|---|---|---|---|---|")
allm, allc = [], []
for k in sorted(grp):
    v = grp[k]
    m = sl.tally((t[0], t[1]) for t in v)
    c = sl.tally((t[2], t[1]) for t in v)
    allm += [(t[0], t[1]) for t in v]
    allc += [(t[2], t[1]) for t in v]
    us = [t[3] for t in v]
    print(f"| {k[0]} | {k[1]} | {k[2]} | {len(v)} | {m[1]}/{m[2]}/{m[3]} | {c[1]}/{c[2]}/{c[3]} | "
          f"{st.median(us):.1f}, {st.pstdev(us):.1f} |")
m, c = sl.tally(allm), sl.tally(allc)
print(f"| ALL | | | {m[0]} | {m[1]}/{m[2]}/{m[3]} | {c[1]}/{c[2]}/{c[3]} | |")
