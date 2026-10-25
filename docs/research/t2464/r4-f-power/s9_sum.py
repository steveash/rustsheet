"""s9: is the x-power product summed with h*L(r) before a store?  Regressors per run (each alone with eLx, eL):
  rx   : own rounding of fl(c*LN(x))
  rPH  : own rounding of fl(P + H) with P = c*LN(x), H = h*L both extended (sign as in lnF)
  rPxH : own rounding of fl(fl(P) + H)
  rPHx : own rounding of fl(P + fl(H))
Usage: s9_sum.py SRC MINMAX"""
import collections
import sys

import numpy as np

import cands as C
import plib as P
import s2_reg

LDB, S, ex = C.LDB, C.S, C.ex
U = P.U


def feats(x, m, n):
    s, _ = C.pieces(x, m, n)
    a, b, h = s["a"], s["b"], s["h"]
    side1 = s["mx"] <= n
    c, sg = (a, 1) if side1 else (b, -1)
    L = s["L1p2"] if side1 else s["L1p1"]
    Pp = LDB(c) * s["Lx"]
    H = LDB(h) * L
    tot = sg * Pp - H

    def rho(v):
        return float((ex(S(v)) - ex(v)) * U)

    return {"rx": sg * rho(Pp), "rPH": rho(tot), "rPxH": rho(sg * S(Pp) - H), "rPHx": rho(sg * Pp - S(H))}


src, minmax = sys.argv[1], float(sys.argv[2])
tot = collections.defaultdict(list)
for blk, rs, m, n in P.runs(src, minmax):
    D = np.array([P.D_units(r) for r in rs])
    F0 = np.array([[s2_reg.feats(r["x"], m, n)[0][k] for k in ("eLx", "eL")] for r in rs])
    E = [feats(r["x"], m, n) for r in rs]
    line = []
    for k in E[0]:
        e = np.array([d[k] for d in E])
        A = np.column_stack([F0, e, np.ones(len(D))])
        cf, *_ = np.linalg.lstsq(A, D, rcond=None)
        sd = (D - A @ cf).std()
        tot[k].append(sd)
        line.append(f"{k} {sd:6.1f} ({cf[2]:+.2f})")
    print(f"{blk:20s} Dsd {D.std():6.1f} | " + " | ".join(line))
print({k: round(float(np.median(v)), 2) for k, v in tot.items()})
