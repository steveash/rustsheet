"""T-2512 r3-bgrat: bgrat's stop rule on Excel's direct bgrat cells (probe t2512-r3-bgrat-split).

TOMS bgrat (r2-bratio's port, doubles, Excel's EXP/LN/log1p/GAMMALN, FINAL's lnx cut) called as the direct branch
does, bgrat(a, b0, x, y, w = 0, eps), with eps = 15e-15 (TOMS), 1e-15, 1e-16 and 1e-17 (eps 0 never ends grat1), scored against
Excel's D, plus the number of terms used."""
from __future__ import annotations

import sys
from collections import defaultdict

from common import split_kernel, table

K, rows = split_kernel()
if len(sys.argv) > 1:
    rows = rows[:: int(sys.argv[1])]
EPS = [("toms 15e-15", 15e-15), ("1e-15", 1e-15), ("1e-16", 1e-16), ("1e-17", 1e-17)]
g = defaultdict(list)
for a, b0, x, case, d in rows:
    for name, e in EPS:
        m = K.bgrat(a, b0, x, 1.0 - x, 0.0, e)
        g[(case.replace("t2464-", ""), name)].append((m, d))
        g[("ALL", name)].append((m, d))
table(g, ("case", "eps"))
