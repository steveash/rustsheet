"""T-2512 r3-bgrat: one-term L140 bup+bgrat cells = RN(W + D) with Excel's own pieces (probes -split, -bupdens).

D = Excel's BETA.DIST(x, a, b0, TRUE) (its bgrat increment alone). W = bup's one term, r2-bratio's form
fl(fl(P * fl(x y)) / b0) with P one of: the model density (M0), Excel's BETA.DIST(x, a, b0, FALSE) (Pa) or
BETA.DIST(y, b0, a, FALSE) (Pb, bup's own orientation); plus x87 variants (the product chain kept extended, one
rounding) and RN(W_exact_from_P + D) with W and the sum exact; two ablations of the best form (exact F for D,
the model density r2-bratio uses for Pb). Scored against Excel's L140 cell, by hold-out half."""
from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path

import mpmath
import numpy as np

import bscore as score
import score_lib as sl

HERE = Path(__file__).resolve().parent
CASES = HERE.parents[3] / "eval/suites/probes-numeric/cases"
LD = np.longdouble


def sheet(name):
    return json.load(open(CASES / f"{name}.oracle.json"))["sheets"]["Calc"]


def num(c):
    return c["v"] if c and c.get("t") == "n" else None


s2, s3 = sheet("t2512-r3-bgrat-split"), sheet("t2512-r3-bgrat-bupdens")
D = {(a, b0, x): num(s2.get(f"A{k}")) for k, a, b0, x, _c, _e in json.load(open(HERE / "t2512-r3-bgrat-split.meta.json"))}
P = {(a, b0, x): (num(s3.get(f"A{k}")), num(s3.get(f"B{k}")))
     for k, a, b0, x in json.load(open(HERE / "t2512-r3-bgrat-bupdens.meta.json"))}
mpmath.mp.dps = 40
g = defaultdict(list)
sys.path.insert(0, str(HERE.parent / "r2-tdist"))
import branches  # noqa: E402
import xlk  # noqa: E402

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
print("| model | half | n | exact/w4/w15 |")
print("|---|---|---|---|")
for k in sorted(g):
    n, e, f, w = sl.tally(g[k])
    print(f"| {k[0]} | {k[1]} | {n} | {e}/{f}/{w} |")
