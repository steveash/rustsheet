"""No-free-parameter scan: E = tree(C, -h, -b1, -b2) with C = stirlerr(n) - stirlerr(k) - stirlerr(n-k) (R order,
stored 'x' or extended 'e'), h = lf/2 (LF0, LF1, exact), stirlerr by mode, x part = r3 model; out = EXP_V6(E).
Counts exact cells per block group.  Usage: b16_full.py <probe> <fn> <block substring> [stir modes] [top]"""
import sys

import blib as B

LDB = B.LDB
name, fn, sub = sys.argv[1:4]
stirs = (sys.argv[4] if len(sys.argv) > 4 else "R,ex").split(",")
top = int(sys.argv[5]) if len(sys.argv) > 5 else 12
rows = [r for r in B.load_probe(name) if r["fn"] == fn and sub in r["blk"]]
o = dict(mr="xx")
X = [B.xpart_v(r["p"], r["k"], r["n"], o) for r in rows]
trees = list(B.tree_specs())
res = []
for sm in stirs:
    for csm in "xe":
        for lf in ("LF0", "LF1", "ex"):
            for t in trees:
                hit = 0
                for r, (b1, b2, _) in zip(rows, X):
                    k, n = r["k"], r["n"]
                    s1 = B._s(B.stirlerr(n, sm) - B.stirlerr(k, sm), csm)
                    C = B._s(s1 - B.stirlerr(n - k, sm), csm)
                    _, h = B.shape_Lh(r, dict(lf=lf))
                    E = B.tree_eval(t, dict(C=C, H=-LDB(h), P=-b1, Q=-b2))
                    hit += B.R3.exp_v6(E) == r["excel"]
                res.append((hit, sm, csm, lf, B.tree_name(t)))
res.sort(reverse=True)
print(len(rows), "cells")
for x in res[:top]:
    print(*x)
