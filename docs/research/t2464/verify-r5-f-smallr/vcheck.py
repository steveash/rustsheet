"""verify-r5-f-smallr: check the worker's xs1 pieces against own implementations, on given cases.
Own pieces: r (side_r), Excel's 14-term log1p series with x87 stored ops (fl53(fl64(.)) via mpmath), the exact
constant K_exact (mpmath), and delta = K_of - K_exact in units (must be small and shape-smooth, not fitted here)."""
from __future__ import annotations

import sys

import mpmath

import vparse as V
import vscore as S

mpmath.mp.prec = 200


def rb(v, bits):
    v = mpmath.mpf(v)
    return mpmath.mpf(mpmath.libmp.normalize(*v._mpf_[:3], v._mpf_[3], bits, "n")) if v else v


def x87(op, a, b):
    A, B = mpmath.mpf(a), mpmath.mpf(b)
    r = {"+": A + B, "*": A * B, "/": A / B}[op]
    return float(rb(rb(r, 64), 53))


def log1p_ser(r):
    """2 atanh(t), t = r/(r+2): 2(t + sum_k t^(2k+1)/(2k+1)), k = 1..14, x87 stored ops (Excel's, per r2-log1p)."""
    t = x87("/", r, r + 2.0)
    t2 = x87("*", t, t)
    p, s = t, None
    for k in range(1, 15):
        p = x87("*", p, t2)
        c = float(mpmath.mpf(1) / (2 * k + 1))
        tm = x87("*", p, c)
        s = tm if s is None else x87("+", s, tm)
    return 2 * x87("+", t, s)


def K_exact(m, n, side):
    M, N = mpmath.mpf(m), mpmath.mpf(n)
    a, b = M / 2, N / 2
    lb = mpmath.log(mpmath.beta(a, b))
    return (a if side == 1 else -b) * mpmath.log(M / N) - lb


if __name__ == "__main__":
    FM = S.model()
    R, SD, TD = FM.R, FM.SD, FM.R.F.TD
    bad_r = bad_L = 0
    tot = 0
    deltas = {}
    for cid in sys.argv[1:]:
        for cell, x, m, n, e in V.rows(cid)[0]:
            side, r = S.side_r(x, m, n)
            s1, a, b, h, rr, P = R.base(x, m, n)
            tot += 1
            bad_r += (rr != r) or ((side == 1) != s1)
            bad_L += float(TD.log1p_xl(r)) != log1p_ser(r)
            if (m, n, side) not in deltas:
                pq = SD.K_of(m, n, side).as_integer_ratio()  # the extended K exactly
                deltas[(m, n, side)] = float((mpmath.mpf(pq[0]) / pq[1] - K_exact(m, n, side)) * mpmath.mpf(2) ** 53)
    print("cells", tot, "r/side mismatches", bad_r, "log1p mismatches", bad_L)
    for k, d in sorted(deltas.items()):
        print("shape", k, "K_of - K_exact =", round(d, 1), "units; worker delta", round(SD.delta(*k), 2))
