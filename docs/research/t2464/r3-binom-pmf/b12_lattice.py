"""Model-free lattice test: E = K + s*D with D a free double per cell and K one real constant per block, for three
decompositions of Excel's exponent E (EXP_V6 of an extended E):
  lc : D = lc (stored, ~ ln y + h),          K = -h            (s = +1)
  X  : D = b1 + b2 (stored, ~ K0 - ln y),     K = Cs - h        (s = -1)
  E  : D = E itself (stored): K = 0 -- the eset test.
K is reported as its offset from the exact value in 2^-53 units (mod the D lattice period).
Usage: b12_lattice.py <probe> <fn> [block substring] [decomps]"""
import math
import sys

import mpmath

import blib as B

LDB = B.LDB
name, fn = sys.argv[1:3]
sub = sys.argv[3] if len(sys.argv) > 3 else ""
decs = (sys.argv[4] if len(sys.argv) > 4 else "lc,X").split(",")
rows = B.load_probe(name)
J = 8
for (f_, blk), idx in sorted(B.blocks(rows, fn).items()):
    if sub not in blk:
        continue
    rs = [rows[i] for i in idx]
    k, n = rs[0]["k"], rs[0]["n"]
    h0 = mpmath.log(2 * mpmath.pi * k * (n - k) / n) / 2
    cs0 = B.stirlerr_exact(n) - B.stirlerr_exact(k) - B.stirlerr_exact(n - k)
    if f_ == "BETA":
        h0 = h0 - mpmath.log(n + 1)
    line = [f"{blk:24s}"]
    for dec in decs:
        K0, s = (-h0, 1) if dec == "lc" else (cs0 - h0, -1)
        ivs = []
        per = None
        for r in rs:
            y = r["excel"]
            d0 = float(s * (mpmath.log(y) - K0))
            per = per or math.ulp(d0) / 2**-53
            cell = []
            for j in range(-J, J + 1):
                D = B.R3.nextd(d0, j)

                def F(u, D=D):
                    return B.R3.exp_v6(B.R3._ld_of(K0) + LDB(u) * B.R3.U53 + s * LDB(D))

                iv = B.R3.off_interval(F, y, y, 4 * per + 64, 1e-3)
                if iv and F(iv[0]) == y:
                    cell.append(iv)
            ivs.append(cell)
        best, pos = B.R3.best_overlap(ivs)
        line.append(f"{dec}: {best}/{len(rs)} K-off {pos % per:7.3f} (per {per:g})")
    print("  ".join(line))
