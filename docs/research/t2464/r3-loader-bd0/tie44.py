"""(4,4) q-tie response: r2's estimator (regress the error on the relative rounding error of fl(1-x)) applied to
Excel and to the x87 Loader model's own error dX (chain-free).  Usage: tie44.py BLOCKREGEX"""
import re
import sys

import mpmath
import numpy as np

import r3lib as L
import rmsfit as RF
import scan as S

rows = S.load("old")
pat = re.compile(sys.argv[1])
for blk, idx in sorted(L.blocks(rows).items()):
    if not pat.fullmatch(blk):
        continue
    Ds = RF.block_D(rows, idx)
    Y, Ym, F = [], [], []
    for i, (d, xe) in zip(idx, Ds):
        r = rows[i]
        x = mpmath.mpf(r["x"])
        b1, b2, _ = L.xpart(r["x"], r["a"], r["b"], {})
        xm = L.frac(-b1 - b2)
        Y.append(d)
        Ym.append(float((mpmath.mpf(xm.numerator) / xm.denominator - xe) * RF.U))
        F.append([1.0, float((mpmath.mpf(1.0 - r["x"]) - (1 - x)) / (1 - x) * RF.U)])
    F = np.array(F)
    for nm, y in (("excel", np.array(Y)), ("model", np.array(Ym))):
        c, *_ = np.linalg.lstsq(F, y, rcond=None)
        print(f"{blk} ({rows[idx[0]]['a']:g},{rows[idx[0]]['b']:g}) {nm}: q-tie coefficient {c[1]:+.2f}")
