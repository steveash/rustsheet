"""Model-free test of out = fl(EXP_ext(D) * F) per block: D a free double per cell, F one real factor per block.
Reports the best overlap, F's offset from the exact 1/sqrt(f) (f = 2 pi k (n-k)/n) in 2^-53 units modulo the D
lattice period (ulp(D)/2^-53), and the offsets of candidate double factors.
Usage: b10_fres.py <probe> <fn> [block substring] [x|e]"""
import math
import sys
from fractions import Fraction as Fr

import mpmath
import numpy as np

import blib as B

LDB = B.LDB
name, fn = sys.argv[1:3]
sub = sys.argv[3] if len(sys.argv) > 3 else ""
mode = sys.argv[4] if len(sys.argv) > 4 else "e"
rows = B.load_probe(name)
J = 10


def fit(rs, lnF0, F0):
    ivs = []
    for r in rs:
        y = r["excel"]
        u = Fr(math.ulp(y)) / 2
        d0 = float(mpmath.log(y)) - lnF0
        cell = []
        for j in range(-J, J + 1):
            D = B.R3.nextd(d0, j)
            e = Fr(*LDB(B.R3.exp_v6(LDB(D), mode == "e")).as_integer_ratio())
            cell.append((float(((Fr(y) - u) / e / Fr(F0) - 1) * 2**53), float(((Fr(y) + u) / e / Fr(F0) - 1) * 2**53)))
        ivs.append(cell)
    return B.R3.best_overlap(ivs), d0


def cands(k, n):
    X = lambda v: LDB(B.st(v))  # noqa: E731
    K, NK, N, P2 = LDB(k), LDB(n - k), LDB(n), LDB(B.M_2PI)
    f1 = X(X(X(P2 * K) * NK) / N)
    return {"1/fl(sqrt f)": 1 / LDB(B.st(np.sqrt(f1))), "fl(1/sqrt f)": X(1 / np.sqrt(f1)),
            "1/sqrt f (ext)": 1 / np.sqrt(f1), "fl(sqrt(n/(2pi k nk)))": X(np.sqrt(X(N / X(X(P2 * K) * NK))))}


for (f_, blk), idx in sorted(B.blocks(rows, fn).items()):
    if sub not in blk:
        continue
    rs = [rows[i] for i in idx]
    k, n = rs[0]["k"], rs[0]["n"]
    fx = 2 * mpmath.pi * k * (n - k) / n
    F0 = float(1 / mpmath.sqrt(fx))
    (best, pos), d0 = fit(rs, math.log(F0), F0)
    per = math.ulp(d0) / 2**-53
    cs = {nm: float((Fr(*LDB(v).as_integer_ratio()) / Fr(F0) - 1) * 2**53) % per for nm, v in cands(k, n).items()}
    print(f"{blk:26s} best {best}/{len(rs)}  F-res {pos % per:6.3f} (period {per:g})  " +
          "  ".join(f"{nm} {v:6.3f}" for nm, v in cs.items()))
