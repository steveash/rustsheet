"""s21: residual sequence of one run under an m5 order and K offset."""
import sys

import mpmath
import numpy as np

import m5_sums as M5
import plib as P

src, want, order, j = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
rows = [r for r in P.load(src) if r["blk"] == want]
m, n = rows[0]["d1"], rows[0]["d2"]
res = []
for r in rows:
    t = M5.terms(r["x"], m, n, j)
    res.append(P.D_units(r) - float((M5.ex(M5.total(t, order, "e")) - mpmath.log(r["exact"])) * P.U))
res = np.array(res)
print(" ".join(f"{v:.0f}" for v in res))
