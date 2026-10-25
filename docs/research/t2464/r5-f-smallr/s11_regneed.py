"""Grid search: which linear combination alpha*eP + beta*er + gamma*eLx (+ free offset) falls inside the most
needed-shift intervals of s10_need (the shift of C each cell needs).  Usage: s11_regneed.py SET tag chain"""
import itertools
import sys

import numpy as np

import r5lib as R
import s10_need as N

src, tag, ch = sys.argv[1:4]
for blk, rs, m, n in R.runs(src, lambda m, n, rs: tag in rs[0]["blk"]):
    sc, iv = N.need(rs, m, n, ch)
    rg = np.array(N.regressors(rs, m, n))
    lo = np.array([v[0] for v in iv])
    hi = np.array([v[1] for v in iv])
    good = ~np.isnan(lo)
    best = []
    for al, be, ga in itertools.product(np.arange(-1.5, 1.51, 0.125), np.arange(-1.5, 1.51, 0.125),
                                        np.arange(-100, 101, 5.0)):
        z = al * rg[:, 2] + be * rg[:, 0] + ga * rg[:, 1]
        cnt = 0
        for o in np.arange(-32, 32.1, 1.0):
            c = int(np.sum(good & (lo <= z + o) & (z + o <= hi)))
            cnt = max(cnt, c)
        best.append((cnt, al, be, ga))
    best.sort(reverse=True)
    print(blk, ch, "base", sc, "of", len(rs), "best:", best[:6])
