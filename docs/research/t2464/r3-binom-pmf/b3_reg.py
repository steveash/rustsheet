"""Per-block regression of Excel's exponent error D on the rounding errors of Loader's log-form bd0 pieces (each
feature signed as it enters E, in 2^-53 units; coefficient 1 = that rounding is in Excel).  Log-form sides only.
Usage: b3_reg.py <probe> <fn> [block substring] [split] [shift]"""
import math
import sys
from fractions import Fraction as F

import mpmath
import numpy as np

import blib as B

name, fn = sys.argv[1], sys.argv[2]
sub = sys.argv[3] if len(sys.argv) > 3 else ""
SPLIT = "split" in sys.argv[4:]
SHIFT = "shift" in sys.argv[4:]  # NEGBINOM as p * dbinom_raw(k-1, n-1, p, q): features of the shifted kernel
rows = [r for r in B.load_probe(name) if r["fn"] == fn and sub in r["blk"]]
U = mpmath.mpf(2) ** -53
LDB = np.longdouble


def ld(v):
    return mpmath.mpf(F(*LDB(v).as_integer_ratio()).numerator) / F(*LDB(v).as_integer_ratio()).denominator


def side(k, M_d, M_x, tag):
    """features of bd0(k, M) log form: M as computed (double) vs exact M_x."""
    k = mpmath.mpf(k)
    Md = mpmath.mpf(M_d)
    f = {}
    if SPLIT:  # M's rounding through the ratio k/M (coefficient k/M) and through the linear M - k term (-1)
        f[tag + ".Mr"] = (k / Md) * (Md - M_x) / U
        f[tag + ".Ml"] = -(Md - M_x) / U
    else:
        f[tag + ".M"] = -(1 - k / Md) * (Md - M_x) / U
    r = B.st(LDB(k) / LDB(M_d))
    f[tag + ".div"] = -k * (mpmath.mpf(r) - k / Md) / (k / Md) / U
    l = B.LN(r)
    f[tag + ".ln"] = -k * (mpmath.mpf(l) - mpmath.log(mpmath.mpf(r))) / U
    t = B.st(LDB(k) * LDB(l))
    f[tag + ".mul"] = -(mpmath.mpf(t) - k * mpmath.mpf(l)) / U
    dd = B.st(LDB(M_d) - LDB(k))
    f[tag + ".sub"] = -(mpmath.mpf(dd) - (Md - k)) / U
    return f


blks = {}
for r in rows:
    blks.setdefault(r["blk"], []).append(r)
for blk, rs in sorted(blks.items()):
    k, n = rs[0]["k"], rs[0]["n"]
    if SHIFT:
        k, n = k - 1, n - 1
        if k < 1:
            continue
    X, Y = [], []
    names = None
    for r in rs:
        p = r["p"]
        P = mpmath.mpf(p)
        q = B.st(1 - LDB(p))
        Q = mpmath.mpf(q)
        npd = B.st(LDB(n) * LDB(p))
        nqd = B.st(LDB(n) * LDB(q))
        f = {"q": None}
        vp = abs(k - npd) / (k + npd)
        vq = abs((n - k) - nqd) / ((n - k) + nqd)
        f = {}
        if vp >= 0.2:
            f.update(side(k, npd, n * P, "p"))
        if vq >= 0.2:
            f.update(side(n - k, nqd, n * (1 - P), "q"))
        else:  # series side: only M's rounding (and q's) enter, via d bd0/dM
            Md = mpmath.mpf(nqd)
            f["q.M"] = -(1 - (n - k) / Md) * (Md - n * (1 - P)) / U
        if vp < 0.2:
            Md = mpmath.mpf(npd)
            f["p.M"] = -(1 - k / Md) * (Md - n * P) / U
        Dv = float((mpmath.log(r["excel"]) - B.exact_ln(r)) / U)
        if names is None:
            names = sorted(f)
        if sorted(f) != names:
            continue
        X.append([float(f[c]) for c in names] + [1.0])
        Y.append(Dv)
    X, Y = np.array(X), np.array(Y)
    c, *_ = np.linalg.lstsq(X, Y, rcond=None)
    res = Y - X @ c
    print(f"{blk:26s} n={len(Y)} sd D {Y.std():7.2f} -> {res.std():5.2f}  " +
          " ".join(f"{nm}={cv:+.2f}" for nm, cv in zip(names + ["c"], c)))
