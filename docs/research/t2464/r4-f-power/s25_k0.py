"""s25: delta0 = nearest double of K' minus K' (units) per shape/side, and the HKP11 residual (sd, off-mode cells,
mean) at K_in = that double (no fitting)."""
import sys

import mpmath
import numpy as np

import m7_flex as M7
import plib as P
import s3_order

src, minmax = sys.argv[1], float(sys.argv[2])
for blk, rs, m, n in P.runs(src, minmax):
    side1 = m * rs[0]["x"] <= n
    K0 = s3_order.kprime(m, n, side1)
    d0 = float((mpmath.mpf(float(K0)) - K0) * P.U)
    D = np.array([P.D_units(r) for r in rs])
    d = []
    for r, Dv in zip(rs, D):
        t, lx = M7.terms(r["x"], m, n, 0, 0)
        d.append(Dv - float((M7.value(t, lx, "HKP", (1, 1)) - mpmath.log(r["exact"])) * P.U))
    d = np.array(d)
    vals, cnt = np.unique(np.round(d), return_counts=True)
    mv = vals[np.argmax(cnt)]
    print(f"{blk:20s} side{1 if side1 else 2} K'={float(K0):10.5f} delta0 {d0:+7.1f} | sd {d.std():6.1f} off {int(np.sum(np.abs(d - mv) > 3)):3d} mode {mv:+.0f}")
