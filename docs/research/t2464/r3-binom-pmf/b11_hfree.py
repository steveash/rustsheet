"""Model-free test of out = EXP_V6(D - H) with D a free double per cell (stored lc) and H one real constant per
block (extended subtraction).  Reports the best overlap and H's offset from lf/2 (exact) modulo the D lattice.
Usage: b11_hfree.py <probe> <fn> [block substring] [min period]"""
import math
import sys

import mpmath

import blib as B

LDB = B.LDB
name, fn = sys.argv[1:3]
sub = sys.argv[3] if len(sys.argv) > 3 else ""
minper = float(sys.argv[4]) if len(sys.argv) > 4 else 4
rows = B.load_probe(name)
J = 10
for (f_, blk), idx in sorted(B.blocks(rows, fn).items()):
    if sub not in blk:
        continue
    rs = [rows[i] for i in idx]
    k, n = rs[0]["k"], rs[0]["n"]
    h0 = float(mpmath.log(2 * mpmath.pi * k * (n - k) / n) / 2)
    d00 = float(mpmath.log(rs[0]["excel"])) + h0
    per = math.ulp(d00) / 2**-53
    if per < minper:
        continue
    ivs = []
    for r in rs:
        y = r["excel"]
        d0 = float(mpmath.log(y)) + h0
        cell = []
        for j in range(-J, J + 1):
            D = B.R3.nextd(d0, j)

            def F(u, D=D):  # increasing in u: E = D - (h0 - u*2^-53)
                return B.R3.exp_v6(LDB(D) - (LDB(h0) - LDB(u) * B.R3.U53))

            iv = B.R3.off_interval(F, y, y, 4 * per + 64, 1e-3)
            if iv and F(iv[0]) == y:
                cell.append(iv)
        ivs.append(cell)
    best, pos = B.R3.best_overlap(ivs)
    print(f"{blk:26s} best {best}/{len(rs)}  H = h_exact - {pos:.3f} units (mod period {per:g}: {pos % per:.3f})")
    out = []
    for lf in ("LF0", "LF1", "ex"):
        _, hm = B.shape_Lh(rs[0], dict(lf=lf))
        u0 = -float((mpmath.mpf(hm) - mpmath.mpf(h0)) * 2**53)  # pos value that corresponds to H = hm
        best_m = max(B.R3.hits_at(ivs, u0 + m * per) for m in range(-int(600 / per) - 2, int(600 / per) + 3))
        out.append(f"{lf} {best_m}")
    print("      at model H:", "  ".join(out))
