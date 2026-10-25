"""Fixed C = R's stirlerr constant; E = C - fl(h' + b1 + b2) (tree T:(C+(H+(P+Q)e)x)e by default); scan
h' = h_exact + d over one lattice period of S = h' + X: the plateau of d with the most exact cells, and where
the candidate h formulas fall (mod period).  Usage: b20_hplat.py <probe> <fn> <block substring> [tree] [step]"""
import math
import sys

import mpmath
import numpy as np

import blib as B

LDB = B.LDB
name, fn, sub = sys.argv[1:4]
tree = sys.argv[4] if len(sys.argv) > 4 else "(C+(H+(P+Q)e)x)e"
step = float(sys.argv[5]) if len(sys.argv) > 5 else 0.1
t = B._tree_by_name(tree)
rows = B.load_probe(name)
o = dict(mr="xx")
for (f_, blk), idx in sorted(B.blocks(rows, fn).items()):
    if sub not in blk:
        continue
    rs = [rows[i] for i in idx]
    k, n = rs[0]["k"], rs[0]["n"]
    X = [B.xpart_v(r["p"], k, n, o) for r in rs]
    C = B._s(B.stirlerr(n, "R") - B.stirlerr(k, "R"), "e") - B.stirlerr(n - k, "R")
    h0m = mpmath.log(2 * mpmath.pi * k * (n - k) / n) / 2
    h0 = B.R3._ld_of(h0m)
    per = math.ulp(float(h0 + X[0][0] + X[0][1])) / 2**-53
    out = []
    for d in np.arange(-per / 2, per / 2, step):
        h = h0 + LDB(d) * B.R3.U53
        out.append((sum(B.R3.exp_v6(B.tree_eval(t, dict(C=C, H=-h, P=-b1, Q=-b2))) == r["excel"]
                        for r, (b1, b2, _) in zip(rs, X)), d))
    mx = max(c for c, _ in out)
    ds = [d for c, d in out if c == mx]
    cands = {nm: (float((mpmath.mpf(float(v)) - h0m) * 2**53) + per / 2) % per - per / 2
             for nm, v in B.h_variants(k, n).items()}
    print(f"{blk:24s} {X[0][2]} max {mx}/{len(rs)} at d in [{min(ds):+.2f},{max(ds):+.2f}] (per {per:g})  cands " +
          " ".join(f"{nm[:6]}{v:+.2f}" for nm, v in cands.items()))
