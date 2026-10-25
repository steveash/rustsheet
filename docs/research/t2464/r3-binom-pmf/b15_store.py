"""Is there one stored sum S = fl(c + Y) in Excel's exponent, Y = -X (the model's x part) and c a constant?
Scans c over one lattice period of S (c = c0 + u units) and reports the best correlation (and slope) between the
model residual r = D + dX and the rounding error of S.  c0 = exact Cs (S = lc) or Cs - h (S = E).
Usage: b15_store.py <probe> <fn> <block substring> [lc|E]"""
import math
import sys
from fractions import Fraction

import mpmath
import numpy as np

import blib as B

LDB = B.LDB
name, fn, sub = sys.argv[1:4]
which = sys.argv[4] if len(sys.argv) > 4 else "lc"
rows = B.load_probe(name)
U = 2.0**-53
o = dict(mr="xx")


def fr(v):
    return Fraction(*LDB(v).as_integer_ratio())


for (f_, blk), idx in sorted(B.blocks(rows, fn).items()):
    if sub not in blk:
        continue
    rs = [rows[i] for i in idx]
    k, n = rs[0]["k"], rs[0]["n"]
    cs = B.stirlerr_exact(n) - B.stirlerr_exact(k) - B.stirlerr_exact(n - k)
    h = mpmath.log(2 * mpmath.pi * k * (n - k) / n) / 2
    c0 = cs if which == "lc" else cs - h
    res, Ys = [], []
    for r in rs:
        P = mpmath.mpf(r["p"])
        b1, b2, _ = B.xpart_v(r["p"], k, n, o)
        Xm = fr(b1 + b2)
        Xe = sum(kk * mpmath.log(kk / M) + M - kk for kk, M in ((k, n * P), (n - k, n * (1 - P))))
        dX = (mpmath.mpf(Xm.numerator) / Xm.denominator - Xe) / U
        Dv = (mpmath.log(r["excel"]) - B.exact_ln(r)) / U
        res.append(float(Dv + dX))
        Ys.append(-Xm)
    res = np.array(res) - np.mean(res)
    c0f = Fraction(mpmath.mpf(c0).man) * Fraction(2) ** int(mpmath.mpf(c0).exp) if False else Fraction(str(mpmath.nstr(c0, 40)))
    S0 = float(c0f + Ys[0])
    per = math.ulp(S0) / U
    best = (0, 0, 0)
    for u in np.arange(0, per, per / 200):
        c = c0f + Fraction(u) * Fraction(1, 2**53)
        e = np.array([float((Fraction(float(c + Y)) - (c + Y)) / Fraction(1, 2**53)) for Y in Ys])
        if e.std() == 0:
            continue
        rho = np.corrcoef(res, e)[0, 1]
        if rho > best[0]:
            best = (rho, u, np.polyfit(e, res, 1)[0])
    print(f"{blk:26s} sd r {res.std():6.2f}  S~{S0:+.3f} (ulp {per:g} units)  best rho {best[0]:+.2f} at u {best[1]:.2f} slope {best[2]:+.2f}")
