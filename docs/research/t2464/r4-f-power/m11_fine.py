"""m11: fine delta scan (step 1/16 unit) of K inside fl(fl(K + H) + P) - ln x with the first sum stored ('11') or
extended ('01'), around a coarse centre.  Prints the best few (off-mode cell count, sd).
Usage: m11_fine.py SRC BLK FLAGS CENTRE HALFWIDTH [pext]   (pext: c*LN(x) unstored)"""
import sys

import mpmath
import numpy as np

import m7_flex as M7
import plib as P
import s3_order

src, want, flags, centre, hw = sys.argv[1], sys.argv[2], sys.argv[3], float(sys.argv[4]), float(sys.argv[5])
fl = tuple(int(c) for c in flags)
rows = [r for r in P.load(src) if r["blk"] == want]
m, n = rows[0]["d1"], rows[0]["d2"]
K0 = s3_order.kprime(m, n, m * rows[0]["x"] <= n)
D = np.array([P.D_units(r) for r in rows])
exl = [mpmath.log(r["exact"]) for r in rows]
PEXT = len(sys.argv) > 6 and sys.argv[6] == "pext"


def terms(x):
    t, lx = M7.terms(x, m, n, 0, 0)
    if PEXT:  # the power product kept extended (64-bit) into the stored sum
        s, _ = M7.C.pieces(x, m, n)
        side1 = s["mx"] <= n
        c, sg = (s["a"], 1) if side1 else (s["b"], -1)
        t["P"] = sg * (M7.LDB(c) * s["Lx"])
    return t, lx


T = [terms(r["x"]) for r in rows]
out = []
for dl in np.arange(centre - hw, centre + hw + 1e-9, 1 / 16):
    Kd = P.F.R3._ld_of(K0 + mpmath.mpf(float(dl)) / P.U)
    d = np.array([float((M7.value(dict(t, K=Kd), lx, "HKP", fl) - e) * P.U) for (t, lx), e in zip(T, exl)]) * -1 + D
    vals, cnt = np.unique(np.round(d), return_counts=True)
    mv = vals[np.argmax(cnt)]
    out.append((int(np.sum(np.abs(d - mv) > 3)), d.std(), float(dl), float(mv)))
out.sort()
for o in out[:6]:
    print(f"{want} HKP{flags} delta {o[2]:+.4f} off {o[0]} sd {o[1]:.2f} mode {o[3]:+.0f}")
