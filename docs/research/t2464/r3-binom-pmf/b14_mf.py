"""Matched filter on one block: Excel's exponent error D (2^-53 units) minus the r3 x-part model's error dX (exact
pieces, extended chain) against the rounding errors of candidate intermediates (each in units of its own ulp).
Usage: b14_mf.py <probe> <fn> <block>"""
import sys

import mpmath
import numpy as np

import blib as B
from fractions import Fraction
B.Fr = Fraction

LDB = B.LDB
name, fn, blk = sys.argv[1:4]
rows = [r for r in B.load_probe(name) if r["fn"] == fn and r["blk"] == blk]
k, n = rows[0]["k"], rows[0]["n"]
U = mpmath.mpf(2) ** -53
o = dict(mr="xx")


def exact_X(p):
    P = mpmath.mpf(p)
    tot = 0
    for kk, M in ((k, n * P), (n - k, n * (1 - P))):
        tot += kk * mpmath.log(kk / M) + M - kk
    return tot


def rerr(v_ext, v_exact):  # rounding error of storing in units of ulp
    d = float(v_ext)
    return float((mpmath.mpf(d) - v_exact) / mpmath.mpf(np.spacing(d)))


res, feats = [], {}
for r in rows:
    p = r["p"]
    P = mpmath.mpf(p)
    b1, b2, _ = B.xpart_v(p, k, n, o)
    Xm = mpmath.mpf(B.Fr(*LDB(b1 + b2).as_integer_ratio()).numerator) / B.Fr(*LDB(b1 + b2).as_integer_ratio()).denominator
    dX = (Xm - exact_X(p)) / U
    Dv = (mpmath.log(r["excel"]) - B.exact_ln(r)) / U
    res.append(float(Dv + dX))  # E = C - X: Excel error minus model error (model error enters with minus)
    q = B.st(1 - LDB(p))
    Q = mpmath.mpf(q)
    npd = B.st(LDB(n) * LDB(p))
    nqd = B.st(LDB(n) * LDB(q))
    cand = {
        "fl(np)": (n * P), "fl(nq)": (n * Q), "fl(1-p)": 1 - P, "fl(n-np)": n - mpmath.mpf(npd),
        "fl(k/np)": k / mpmath.mpf(npd), "fl(nk/nq)": (n - k) / mpmath.mpf(nqd), "fl(k-np)": k - mpmath.mpf(npd),
        "fl(np-k)": mpmath.mpf(npd) - k, "fl(nk-nq)": (n - k) - mpmath.mpf(nqd), "fl(np/k)": mpmath.mpf(npd) / k,
        "fl(np*q)": mpmath.mpf(npd) * Q, "fl(p*q)": P * Q, "fl(k/n/p)": mpmath.mpf(B.st(LDB(k) / LDB(n))) / P,
        "fl(p/k)": P / k, "fl(k*q)": k * Q, "fl(n*p - k)ext": n * P - k,
    }
    for nm, ex in cand.items():
        feats.setdefault(nm, []).append(rerr(ex, ex) if False else float((mpmath.mpf(float(ex)) - ex) / mpmath.mpf(np.spacing(float(ex)))))
res = np.array(res)
res -= res.mean()
print(blk, "sd(D + dX) =", res.std())
for nm, v in feats.items():
    v = np.array(v)
    if v.std() < 1e-9:
        print(f"  {nm:14s} constant")
        continue
    rho = np.corrcoef(res, v)[0, 1]
    print(f"  {nm:14s} rho {rho:+.2f}  slope {np.polyfit(v, res, 1)[0]:+8.2f} units per ulp")
