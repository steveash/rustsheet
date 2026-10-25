"""Scan the double factor: out = fl(EXP_ext(D) * F) or fl(EXP_ext(D) / G) over doubles F (G) near the exact
1/sqrt(f) (sqrt(f)), f = 2 pi k (n-k)/n, with D = fl(C - X) for a free C (X = b1 + b2, r3 model) -- or D free per
cell (--free).  Prints the best doubles and, for the best, the candidate formulas that produce it.
Usage: b9_fscan.py <probe> <fn> <block> [--free]"""
import math
import sys
from fractions import Fraction as Fr

import mpmath

import blib as B

LDB = B.LDB
name, fn, blk = sys.argv[1:4]
free = "--free" in sys.argv
rows = [r for r in B.load_probe(name) if r["fn"] == fn and r["blk"] == blk]
k, n = rows[0]["k"], rows[0]["n"]
f_ex = 2 * mpmath.pi * k * (n - k) / n
F0, G0 = float(1 / mpmath.sqrt(f_ex)), float(mpmath.sqrt(f_ex))
o = dict(mr="xx")
Xs = [B.xpart_v(r["p"], k, n, o) for r in rows]
J = 6


def count(op, V):
    """best count over a free C (or per-cell free D) for out = fl(e op V)."""
    ivs = []
    for r, (b1, b2, _) in zip(rows, Xs):
        y = r["excel"]
        X = b1 + b2
        d0 = float(mpmath.log(y)) + (0.5 * math.log(float(f_ex)))
        cell = []
        for j in range(-J, J + 1):
            D = B.R3.nextd(d0, j)
            e = B.R3.exp_v6(LDB(D), True)
            out = B.st(e * LDB(V)) if op == "*" else B.st(e / LDB(V))
            if out == y:
                if free:
                    cell.append((j - 0.25, j + 0.25))
                else:  # C values with fl(C - X) == D: C in D + X +- half ulp(D) (units 2^-53, relative to 0)
                    h = math.ulp(D) / 2 / 2**-53
                    c = float((LDB(D) + X) / B.R3.U53)
                    cell.append((c - h, c + h))
        ivs.append(cell)
    if free:  # any D per cell: count cells with a hit
        return sum(bool(c) for c in ivs)
    return B.R3.best_overlap(ivs)[0]


res = []
for op, V0 in (("*", F0), ("/", G0)):
    for j in range(-12, 13):
        V = B.R3.nextd(V0, j)
        res.append((count(op, V), op, j, V))
res.sort(reverse=True)
print(blk, "n cells", len(rows), "F0", F0, "G0", G0)
for c, op, j, V in res[:8]:
    print(f"  {c}/{len(rows)}  op {op}  V = exact{j:+d} ulp = {V!r}")
# candidate formulas
cand = {}
X = lambda v: LDB(B.st(v))  # noqa: E731
K, NK, N, P2 = LDB(k), LDB(n - k), LDB(n), LDB(B.M_2PI)
f1 = X(X(X(P2 * K) * NK) / N)
cand["sqrt(fl R f)"] = B.st(LDB(math.sqrt(float(f1))))
cand["1/sqrt(fl R f)"] = B.st(LDB(1) / LDB(math.sqrt(float(f1))))
cand["1/fsqrt(f) ext"] = B.st(LDB(1) / __import__("numpy").sqrt(f1))
cand["sqrt(n/(2pi k nk))"] = B.st(__import__("numpy").sqrt(X(N / X(X(P2 * K) * NK))))
cand["sqrt(f) via f=2pi*(k*nk/n)"] = B.st(__import__("numpy").sqrt(X(P2 * X(X(K * NK) / N))))
cand["sqrt(f) f = k*nk*2pi/n"] = B.st(__import__("numpy").sqrt(X(X(X(K * NK) * P2) / N)))
for nm, v in cand.items():
    print(f"  {nm:28s} {v!r}")
