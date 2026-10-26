"""For one block and chain, the cells' admissible-C intervals' centres against the rounding errors (units) of
candidate stored intermediates (sums of C, h, b1, b2 and the bd0 pieces): correlation and the least-squares slope.
A missing stored intermediate shows as a slope near -1 or +1 with a high |rho|.  Usage: b7_corr.py <probe> <fn> <block>"""
import math
import sys

import numpy as np

import blib as B

LDB = B.LDB
name, fn, blk = sys.argv[1:4]
o = dict(mr="xx")
rows = B.load_probe(name)
idx = [i for i, r in enumerate(rows) if r["fn"] == fn and r["blk"] == blk]
f = B.fit_block_g(rows, idx, o, "R:eee")
C = f["C0"] + LDB(f["off_all"]) * B.R3.U53
L5, h = B.shape_Lh(rows[idx[0]], o)
H = LDB(h)
U = float(2.0**-53)


def err(v):
    return float(LDB(B.st(v)) - v) / U


cent, feats = [], {}
for i in idx:
    iv = f["ivs"][i]
    if not iv:
        continue
    cent.append((iv[0][0] + iv[0][1]) / 2 - f["off_all"])
    b1, b2, _ = f["X"][i]
    cand = {"P+Q": b1 + b2, "C-P": C - b1, "C-Q": C - b2, "C-P-Q": C - b1 - b2, "C-h": C - H, "h+P+Q": H + b1 + b2,
            "C-h-P": C - H - b1, "C-h-Q": C - H - b2, "E": C - H - b1 - b2, "P": b1, "Q": b2, "P+h": b1 + H}
    for k, v in cand.items():
        feats.setdefault(k, []).append(err(v))
cent = np.array(cent)
print(blk, "cells", len(cent), "sd of interval centres", cent.std())
for k, v in feats.items():
    v = np.array(v)
    if v.std() == 0:
        print(f"{k:8s} constant")
        continue
    rho = np.corrcoef(cent, v)[0, 1]
    slope = np.polyfit(v, cent, 1)[0]
    print(f"{k:8s} rho {rho:+.2f} slope {slope:+.2f} (feature sd {v.std():.2f})")
