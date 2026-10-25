"""s19: continuous coefficient scan.  Plateau est (s16) against t*rho(fl(c*L)) for c = k/64 (k = 1..64*2h), the grid
set by |c*L| itself (no added constant), with t = +-1 and a free offset and linear drift.  Usage: s19_cscan.py SRC BLK"""
import sys
from fractions import Fraction

import numpy as np

import s16_two

src, want = sys.argv[1], sys.argv[2]
m, n, Ls, est, rm = s16_two.plateaus(src, want)
h = (m + n) / 2
K = len(Ls)
base = np.column_stack([np.ones(K), np.arange(K)])
Lf = [Fraction(L) for L in Ls]
out = []
for k in range(1, int(64 * 2 * h) + 1):
    c = Fraction(k, 64)
    rho = []
    for L in Lf:
        v = c * L
        d = float(v)
        rho.append(float((Fraction(d) - v) * 2**53))
    rho = np.array(rho)
    for t in (1, -1):
        y = est - t * rho
        cf, *_ = np.linalg.lstsq(base, y, rcond=None)
        out.append(((y - base @ cf).std(), float(c), t))
out.sort()
print(want, "est sd", round(float(est.std()), 1))
for o in out[:10]:
    print(f"  sd {o[0]:6.2f} c={o[1]:g} t={o[2]:+d}")
