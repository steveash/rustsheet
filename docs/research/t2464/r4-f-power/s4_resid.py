"""s4: after the rx/eLx/eL fit (s2), correlate the per-run residual with each cands.py stored term's own-rounding
residue (stored - fl64 product of its stored inputs) and total error.  Usage: s4_resid.py SRC MINMAX"""
import sys

import numpy as np

import cands as C
import plib as P
import s2_reg

src, minmax = sys.argv[1], float(sys.argv[2])
for blk, rs, m, n in P.runs(src, minmax):
    D = np.array([P.D_units(r) for r in rs])
    F0 = np.array([[s2_reg.feats(r["x"], m, n)[0][k] for k in ("rx", "eLx", "eL")] for r in rs])
    ok = [j for j in range(3) if F0[:, j].std() > 0.3]
    A = np.column_stack([F0[:, ok], np.ones(len(D))])
    cf, *_ = np.linalg.lstsq(A, D, rcond=None)
    res = D - A @ cf
    E = [C.errs(r["x"], m, n) for r in rs]
    out = []
    for k in E[0]:
        e = np.array([d[k] for d in E])
        if e.std() < 1:
            continue
        c = np.corrcoef(res, e)[0, 1]
        out.append((abs(c), k, c, np.polyfit(e, res, 1)[0]))
    out.sort(reverse=True)
    print(f"{blk:20s} res {res.std():6.1f} | " + " | ".join(f"{k} r={c:+.2f} sl={s:+.2f}" for _, k, c, s in out[:3]))
