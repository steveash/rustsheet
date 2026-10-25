"""Exhaustive summation-tree scan for E = C - h - b1 - b2 (free C per block, x part = r3 model) on chosen blocks:
all-cell best-overlap count per tree.  Usage: b6_trees.py <probe> <fn> <block substrings comma> [top]"""
import sys

import blib as B

LDB = B.LDB
name, fn, subs = sys.argv[1:4]
top = int(sys.argv[4]) if len(sys.argv) > 4 else 15
o = dict(mr="xx")
rows = B.load_probe(name)
bl = {k: v for k, v in B.blocks(rows, fn).items() if any(s in k[1] for s in subs.split(","))}
pre = {}
for key, idx in bl.items():
    r0 = rows[idx[0]]
    L5, h = B.shape_Lh(r0, o)
    Xs = {i: B.xpart_v(rows[i]["p"], rows[i]["k"], rows[i]["n"], o) for i in idx}
    i0 = idx[len(idx) // 2]
    C0 = LDB(float(B.exact_ln(rows[i0]))) + Xs[i0][0] + Xs[i0][1] - LDB(L5) + LDB(h)
    pre[key] = (idx, L5, h, Xs, C0)
res = []
specs = list(B.tree_specs())
print(len(specs), "trees", len(bl), "blocks")
for t in specs:
    tot = n = 0
    per = []
    for key, (idx, L5, h, Xs, C0) in sorted(pre.items()):
        ivs = []
        for i in idx:
            b1, b2, _ = Xs[i]

            def F(u, b1=b1, b2=b2):
                E = B.tree_eval(t, dict(C=C0 + LDB(u) * B.R3.U53, H=-LDB(h), P=-b1, Q=-b2))
                if L5:
                    E = LDB(L5) + E
                return B.R3.exp_v6(E)

            y = rows[i]["excel"]
            iv = B.R3.off_interval(F, y, y, 256.0, 1e-3)
            ivs.append([iv] if iv and F(iv[0]) == y else [])
        c, _ = B.R3.best_overlap(ivs)
        tot += c
        n += len(idx)
        per.append(c)
    res.append((tot, B.tree_name(t), per))
res.sort(reverse=True)
for tot, nm, per in res[:top]:
    print(f"{tot}/{n} {nm:40s} {per}")
print("blocks:", sorted(pre))
