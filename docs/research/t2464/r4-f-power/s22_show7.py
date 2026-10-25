"""s22: residual sequence of one run under an m7 candidate (order, flags, j, Hs)."""
import sys

import mpmath
import numpy as np

import m7_flex as M7
import plib as P

src, want, order, flags, j, hs = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5]), int(sys.argv[6])
fl = tuple(int(c) for c in flags)
rows = [r for r in P.load(src) if r["blk"] == want]
m, n = rows[0]["d1"], rows[0]["d2"]
res = []
for r in rows:
    t, lx = M7.terms(r["x"], m, n, j, hs)
    res.append(P.D_units(r) - float((M7.value(t, lx, order, fl) - mpmath.log(r["exact"])) * P.U))
res = np.array(res)
print(f"sd {res.std():.1f}")
print(" ".join(f"{v:.0f}" for v in res))
