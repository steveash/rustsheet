"""Interval inversion for the correction S in l = fl53(fl64(2r + 2r*t2*S)) (exact product, r = fl(a/fl(a+2))):
each pinned row bounds dS = S_excel - S_exact (units 2^-54 = ulp(1/3)); prints the per-bin band of a dS that is
constant in the bin, and the rows that force |dS| > 1.  Usage: sband.py nbins rmin rmax [names]"""
import math
import sys

import mpmath

import pin
import r2lib as R

mpf = R.mpf
RMODE = 'x'  # r = x87-stored a/(a+2); 'd' for IEEE double
U = mpf(2) ** -54


def bounds(r, l):
    """dS interval [lo, hi] (in U) with RN(2r + 2r^3 (S+dS)) == l (single rounding approximation)."""
    m = mpf(r)
    S = (mpmath.atanh(m) - m) / m**3
    u = mpf(math.ulp(l))
    k = 2 * m**3
    a = (mpf(l) - u / 2 - 2 * m) / k - S
    b = (mpf(l) + u / 2 - 2 * m) / k - S
    lo, hi = (a, b) if a < b else (b, a)
    return float(lo / U), float(hi / U)


def rows(names):
    out = []
    for nm in names:
        for p, n, k, l in pin.load(nm):
            if k in ("none", "multi") or p > 0.375:
                continue
            r = R.r_of(p, RMODE)
            out.append((-r, r, l, *bounds(r, l)))
    return out


if __name__ == "__main__":
    nb, rmin, rmax = int(sys.argv[1]), float(sys.argv[2]), float(sys.argv[3])
    rs = rows(sys.argv[4:] or ["runs", "grid", "hold"])
    for j in range(nb):
        a, b = rmin + (rmax - rmin) * j / nb, rmin + (rmax - rmin) * (j + 1) / nb
        sel = [x for x in rs if a <= x[0] < b]
        if not sel:
            continue
        lo = max(x[3] for x in sel)
        hi = min(x[4] for x in sel)
        print(f"-r [{a:.4f},{b:.4f}) n={len(sel):4d} dS band [{lo:+8.2f},{hi:+8.2f}] {'CONFLICT' if lo > hi else ''}")
