"""Is Excel's pmf fl(EXP(D) * F) for a double D and one constant F per block?  For each cell and each double D
within +-J ulps of ln(y) - ln(F0), the admissible F form an interval; the deepest overlap across the block's cells
(one D per cell) gives the best F.  EXP's result is taken as a double ('x') or extended ('e').
Usage: b8_factor.py <probe> <fn> <block> <lnF0> [x|e] [J]"""
import math
import sys
from fractions import Fraction as Fr

import mpmath

import blib as B

name, fn, blk, lnF0 = sys.argv[1:5]
mode = sys.argv[5] if len(sys.argv) > 5 else "x"
J = int(sys.argv[6]) if len(sys.argv) > 6 else 12
lnF0 = float(eval(lnF0))
rows = [r for r in B.load_probe(name) if r["fn"] == fn and r["blk"] == blk]
F0 = math.exp(lnF0)
ivs = []
for r in rows:
    y = r["excel"]
    u = Fr(math.ulp(y)) / 2
    lo_y, hi_y = Fr(y) - u, Fr(y) + u
    d0 = float(mpmath.log(y)) - lnF0
    cell = []
    for j in range(-J, J + 1):
        D = B.R3.nextd(d0, j)
        e = B.R3.exp_v6(B.LDB(D), mode == "e")
        e = Fr(*B.LDB(e).as_integer_ratio())
        # F with lo_y <= e*F <= hi_y  (ties ignored), in units of 2^-53 relative to F0
        flo, fhi = lo_y / e, hi_y / e
        cell.append((float((flo / Fr(F0) - 1) * 2**53), float((fhi / Fr(F0) - 1) * 2**53)))
    ivs.append(cell)
best, pos = B.R3.best_overlap(ivs)
print(f"{blk} mode {mode}: best F overlap {best}/{len(rows)} at F = F0*(1 + {pos:.3f}*2^-53)")

# per-cell admissible D at the best F, and D + X (X = b1 + b2 from the r3 model) in units of 2^-53
if "--cells" in sys.argv:
    o = dict(mr="xx")
    Ff = Fr(F0) * (1 + Fr(pos) / 2**53)
    for r, cell in zip(rows, ivs):
        js = [j for j, (lo, hi) in zip(range(-J, J + 1), cell) if lo <= pos <= hi]
        d0 = float(mpmath.log(r["excel"])) - lnF0
        Ds = [B.R3.nextd(d0, j) for j in js]
        b1, b2, br = B.xpart_v(r["p"], r["k"], r["n"], o)
        X = b1 + b2
        print(r["p"], br, [f"{float((B.LDB(D) + X - B.LDB(Ds[0]) - X) / B.R3.U53):+.1f}" for D in Ds],
              "D+X", [f"{(float(B.LDB(D) + X)):.17f}" for D in Ds], f"X={float(X):.17f}")
