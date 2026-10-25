"""Small p (p < 1e-4): forward power-sum model vs CR log1p on BINOM pins and the density-only tiny pins.
Usage: small_p.py"""
import math

import coef_scan as C
import pin
import r2lib as R

f = C.make(C.BASE)
bins = [(0, 1e-9), (1e-9, 1e-7), (1e-7, 1e-6), (1e-6, 1e-5), (1e-5, 1e-4)]
tab = {b: [0, 0, 0, 0] for b in bins}
for nm in ("runs", "grid", "hold", "tiny"):
    for p, n, k, l in pin.load(nm):
        if k in ("none", "multi") or p >= 1e-4:
            continue
        for b in bins:
            if b[0] <= p < b[1]:
                t = tab[b]
                t[0] += f(R.r_of(p)) == l
                t[1] += f(R.r_of(p, "d")) == l
                t[2] += R.cr_log1p(-p) == l
                t[3] += 1
for b, t in tab.items():
    print(f"p in [{b[0]:.0e},{b[1]:.0e}): fwd(r x87) {t[0]}/{t[3]}  fwd(r double) {t[1]}/{t[3]}  CR {t[2]}/{t[3]}")
