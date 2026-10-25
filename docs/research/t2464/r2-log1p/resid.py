"""Interval inversion for l = RN53(2*atanh(r)*(1+eps)) with r = fl(a/fl(a+2)) (IEEE double): per pinned row,
d = (v - l)/ulp(l) where v = exact 2 atanh(r). A row fits the 'ideal' model iff |d| < 0.5; the required relative error
of the correction otherwise lies in [(-0.5-d)ulp/v, (0.5-d)ulp/v].  Prints misses and |d| statistics per region."""
import math
import sys

import mpmath

import pin
import r2lib as R


def rows_d(names, pmax=0.375):
    out = []
    for nm in names:
        for p, n, k, l in pin.load(nm):
            if k in ("none", "multi") or p > pmax:
                continue
            a = -p
            r = a / (a + 2.0)
            v = 2 * mpmath.atanh(R.mpf(r))
            u = math.ulp(l)
            d = float((v - R.mpf(l)) / u)
            out.append((p, r, l, d, nm))
    return out


if __name__ == "__main__":
    rs = rows_d(sys.argv[1:])
    miss = [x for x in rs if abs(x[3]) >= 0.5]
    print("rows", len(rs), "misses", len(miss))
    for p, r, l, d, nm in sorted(miss):
        # relative error of the correction needed: e in units of ulp(l) relative to v
        lo, hi = (-0.5 - d), (0.5 - d)
        print(f"{nm:5s} p={p!r:24} r={r: .6e} d={d:+.4f} need eps*v/ulp in [{lo:+.4f},{hi:+.4f}]  rel {lo*math.ulp(l)/abs(l):+.2e}..{hi*math.ulp(l)/abs(l):+.2e}")
