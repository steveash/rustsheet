"""Two free constants per block: C (stirlerr part, inside the stored lc) and an offset dh (units 2^-53) on
h = LF0/2, for E = fl(C - b1 - b2) - (h + dh) extended (chain R:exe).  Grid over dh; prints the best count per
block and the dh range that reaches it.  Usage: b13_ch.py <probe> <fn> <block substring> [step] [span] [spec]"""
import sys

import numpy as np

import blib as B

name, fn, sub = sys.argv[1:4]
step = float(sys.argv[4]) if len(sys.argv) > 4 else 0.125
span = float(sys.argv[5]) if len(sys.argv) > 5 else 4.0
spec = sys.argv[6] if len(sys.argv) > 6 else "R:exe"
rows = B.load_probe(name)
o = dict(mr="xx")
orig = B.shape_Lh
for (f_, blk), idx in sorted(B.blocks(rows, fn).items()):
    if sub not in blk:
        continue
    res = []
    for dh in np.arange(-span, span + 1e-9, step):
        B.shape_Lh = lambda r, o_, dh=dh: (lambda L5h: (L5h[0], float(B.LDB(L5h[1]) + B.LDB(dh) * B.R3.U53)))(orig(r, o_))
        f = B.fit_block_g(rows, idx, o, spec)
        res.append((f["all"], dh))
    B.shape_Lh = orig
    best = max(c for c, _ in res)
    good = [dh for c, dh in res if c == best]
    print(f"{blk:26s} best {best}/{len(idx)} at dh in [{min(good):+.3f}, {max(good):+.3f}]  "
          f"(dh=0: {dict((round(d, 3), c) for c, d in res)[0.0]})")
    if "--c" in sys.argv:  # C offset from R's stirlerr constant at the best dh values
        k, n = rows[idx[0]]["k"], rows[idx[0]]["n"]
        cR = B._s(B.stirlerr(n, "R") - B.stirlerr(k, "R"), "e") - B.stirlerr(n - k, "R")
        for c, dh in res:
            if c == best:
                B.shape_Lh = lambda r, o_, dh=dh: (lambda L5h: (L5h[0], float(B.LDB(L5h[1]) + B.LDB(dh) * B.R3.U53)))(orig(r, o_))
                f = B.fit_block_g(rows, idx, o, spec)
                B.shape_Lh = orig
                C = f["C0"] + B.LDB(f["off_all"]) * B.R3.U53
                print(f"    dh {dh:+.3f}: C - C_R = {float((C - cR) / B.R3.U53):+.3f} units")
