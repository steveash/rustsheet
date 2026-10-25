"""Interval coverage: for each cell, the relative offsets c (units 2^-53) such that RN(exact*(1+c)) == excel.
Report the best c (fine grid) and its coverage per block: a pure-constant-times-accurate-value ceiling."""
import math
import mpmath
import dlib as L

rows = L.load()
U = mpmath.mpf(2) ** -53
for blk, idx in sorted(L.blocks(rows).items()):
    ivs = []
    for i in idx:
        r = rows[i]
        v = r["excel"]
        u = math.ulp(v)
        lo = float(((mpmath.mpf(v) - mpmath.mpf(u) / 2) / r["exact"] - 1) / U)
        hi = float(((mpmath.mpf(v) + mpmath.mpf(u) / 2) / r["exact"] - 1) / U)
        ivs.append((lo, hi))
    best = max(((sum(lo <= c / 20 <= hi for lo, hi in ivs), c / 20) for c in range(-800, 801)))
    r0 = rows[idx[0]]
    print(f"{blk:14s} ({r0['a']:g},{r0['b']:g}) best c {best[1]:+6.2f} covers {best[0]}/{len(idx)}")
