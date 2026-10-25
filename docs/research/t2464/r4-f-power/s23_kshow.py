"""s23: residual sequence of one run under an m7 order/flags with K = K' + delta (units)."""
import sys

import mpmath
import numpy as np

import m7_flex as M7
import plib as P
import s3_order

src, want, order, flags, dl = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], float(sys.argv[5])
fl = tuple(int(c) for c in flags)
rows = [r for r in P.load(src) if r["blk"] == want]
m, n = rows[0]["d1"], rows[0]["d2"]
K0 = s3_order.kprime(m, n, m * rows[0]["x"] <= n)
Kd = P.F.R3._ld_of(K0 + mpmath.mpf(dl) / P.U)
res = []
for r in rows:
    t, lx = M7.terms(r["x"], m, n, 0, 0)
    t = dict(t, K=Kd)
    res.append(P.D_units(r) - float((M7.value(t, lx, order, fl) - mpmath.log(r["exact"])) * P.U))
res = np.array(res)
print(f"sd {res.std():.1f}")
print(" ".join(f"{v:.0f}" for v in res))
