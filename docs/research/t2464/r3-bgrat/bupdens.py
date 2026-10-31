"""T-2512 r3-bgrat: one-term L140 bup+bgrat cells = RN(W + D) with Excel's own pieces (probes -split, -bupdens).

D = Excel's BETA.DIST(x, a, b0, TRUE) (its bgrat increment alone). W = bup's one term, r2-bratio's form
fl(fl(P * fl(x y)) / b0) with P one of: the model density (M0), Excel's BETA.DIST(x, a, b0, FALSE) (Pa) or
BETA.DIST(y, b0, a, FALSE) (Pb, bup's own orientation); plus x87 variants (the product chain kept extended, one
rounding) and RN(W_exact_from_P + D) with W and the sum exact; two ablations of the best form (exact F for D,
the model density r2-bratio uses for Pb). Scored against Excel's L140 cell, by hold-out half."""
from __future__ import annotations

from collections import defaultdict

import mpmath
import numpy as np

import bscore as score
from common import bupdens_p, split_d, table, xlk

LD = np.longdouble
D, P = split_d(), bupdens_p()
mpmath.mp.dps = 40
g = defaultdict(list)
import branches  # noqa: E402

xlk.run([(b0, a, 1.0 - x, x) for (a, b0, x) in P], branches.FINAL)
K = xlk.bx._KERNELS[branches.FINAL]
for c in score.cells():
    k = (c["a"], c["b"], c["x"])
    if c["br"] != "L140 bup+bgrat" or k not in P or D.get(k) is None or None in P[k]:
        continue
    r = c["row"]
    a, b0, x = k
    y = 1.0 - x
    dx = D[k]
    half = "hold" if r["hold"] else "tune"
    xy = x * y
    cands = {"M0 (model bup + model bgrat)": c["fn"](c["g"]), "model W + D": float(mpmath.mpf(c["w"]) + dx)}
    for nm, p in (("Pa", P[k][0]), ("Pb", P[k][1])):
        cands[f"fl(fl({nm}*fl(xy))/b0) + D"] = float(mpmath.mpf((p * xy) / b0) + dx)
        cands[f"x87({nm}*fl(xy)/b0) + D"] = float(mpmath.mpf(float(LD(p) * LD(xy) / LD(b0))) + dx)
        cands[f"x87({nm}*x*y/b0) + D"] = float(mpmath.mpf(float(LD(p) * LD(x) * LD(y) / LD(b0))) + dx)
        cands[f"x87(({nm}*fl(xy)/b0) + D)"] = float(LD(p) * LD(xy) / LD(b0) + LD(dx))
        cands[f"exact {nm}*x*y/b0 + D"] = float(mpmath.mpf(p) * x * y / b0 + dx)
    pb = P[k][1]
    pm, _ = K.density(y, b0, a)
    cands["ablate: x87(Pb*fl(xy)/b0) + F (exact increment)"] = float(mpmath.mpf(float(LD(pb) * LD(xy) / LD(b0))) + c["F"])
    cands["ablate: x87(Pmodel*fl(xy)/b0) + D (model density)"] = float(mpmath.mpf(float(LD(pm) * LD(xy) / LD(b0))) + dx)
    for m, v in cands.items():
        g[(m, half)].append((v, r["excel"]))
        g[(m, "ALL")].append((v, r["excel"]))
table(g, ("model", "half"))
