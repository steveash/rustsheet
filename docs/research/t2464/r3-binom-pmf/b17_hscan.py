"""Fixed C (stirlerr table/series, R order) and a tree; scan h = h_exact + dh units: exact count vs dh.
Usage: b17_hscan.py <probe> <fn> <block> <tree> [stir] [cs x|e] [span] [step]"""
import sys

import mpmath
import numpy as np

import blib as B

LDB = B.LDB
name, fn, blk, tree = sys.argv[1:5]
sm = sys.argv[5] if len(sys.argv) > 5 else "R"
csm = sys.argv[6] if len(sys.argv) > 6 else "e"
span = float(sys.argv[7]) if len(sys.argv) > 7 else 8
step = float(sys.argv[8]) if len(sys.argv) > 8 else 0.125
rows = [r for r in B.load_probe(name) if r["fn"] == fn and r["blk"] == blk]
k, n = rows[0]["k"], rows[0]["n"]
o = dict(mr="xx")
X = [B.xpart_v(r["p"], k, n, o) for r in rows]
t = B._tree_by_name(tree)
C = B._s(B._s(B.stirlerr(n, sm) - B.stirlerr(k, sm), csm) - B.stirlerr(n - k, sm), csm)
h0 = B.R3._ld_of(mpmath.log(2 * mpmath.pi * k * (n - k) / n) / 2)
for lf in ("LF0", "LF1"):
    _, hm = B.shape_Lh(rows[0], dict(lf=lf))
    print(lf, "h - h_exact =", float((LDB(hm) - h0) / B.R3.U53), "units")
out = []
for dh in np.arange(-span, span + 1e-9, step):
    h = h0 + LDB(dh) * B.R3.U53
    hit = sum(B.R3.exp_v6(B.tree_eval(t, dict(C=C, H=-h, P=-b1, Q=-b2))) == r["excel"] for r, (b1, b2, _) in zip(rows, X))
    out.append((dh, hit))
print(" ".join(f"{dh:+.2f}:{h}" for dh, h in out if h >= max(x[1] for x in out) * 0.8))
