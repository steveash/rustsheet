"""EXP-internal variants on rows with an exact exponent (r2-log1p a = 1 rows: E = n*l - lnB, n = 2^k, all exact):
which stored piece inside V6 reproduces Excel's density.  Usage: expvar.py [step]"""
import collections
import sys

import mpmath

import m1lib as M
from need import exponent

mpf, R64, R53 = M.mpf, M.fl64, M.fl53
L2E, LN2 = M.W1.LOG2E64, M.W1.LN2_64
TWO = mpf(2)


def tail(n, f, st=""):
    """2^(n+f) the V6 way, with optional stores: 'f' f stored, 'u' u stored, 'w' 1+u stored, 'r' reciprocal
    stored, 's' signed F2XM1 (no reciprocal)."""
    if "f" in st:
        f = mpf(R53(f))
    if "s" in st:
        u = R64(TWO**f - 1)
        w = R64(1 + u)
    else:
        u = R64(TWO ** abs(f) - 1)
        if "u" in st:
            u = mpf(R53(u))
        w = R64(1 + u)
        if "w" in st:
            w = mpf(R53(w))
        if f < 0:
            w = R64(1 / w)
            if "r" in st:
                w = mpf(R53(w))
    return R53(w * TWO**n)


def variants(E):
    t = R64(E * L2E)
    n = mpmath.nint(t)
    out = {k: tail(n, t - n, k) for k in ("", "f", "u", "w", "r", "s", "fs")}
    # Cody-Waite style: r = E - n ln2 (extended ln2), r stored, f = r*log2e
    r = R64(E - R64(n * LN2))
    out["cw_rdouble"] = tail(n, R64(mpf(R53(r)) * L2E))
    # e^int(E) * e^frac(E): frac stored as double, e^k exact-rounded double table
    k = mpmath.nint(E)
    fr = mpf(R53(E - k))
    out["eint*efrac(dbl)"] = R53(mpf(R53(mpmath.exp(k))) * R64(mpmath.exp(fr)))
    out["eint64*efrac"] = R53(R64(mpmath.exp(k)) * R64(mpmath.exp(fr)))
    return out


if __name__ == "__main__":
    step = int(sys.argv[1]) if len(sys.argv) > 1 else 8
    sets = {"log1p": M.load_log1p_rows()[::step], "probe1 a=1": [r for r in M.load_probe1() if r["a"] == 1.0][::2]}
    for nm, rows in sets.items():
        c = collections.Counter()
        for r in rows:
            for k, v in variants(exponent(r)).items():
                c[k] += v == r["excel"]
        print(nm, len(rows), dict(c))
