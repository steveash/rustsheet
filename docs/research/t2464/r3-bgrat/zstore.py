"""T-2512 r3-bgrat: does the stored z = fl(-nu lnx) show in Excel's direct bgrat cells (probe -split)?

D_model = RN(F * Q(b0, z_s) / Q(b0, z_e)): Q exact, z_s the stored double, z_e unrounded (-nu lnx exact, with nu
exact or the double fl(a + 0.5 (b0 - 1))). Also z_s from x87: fl53(fl64(nu * lnx)). Hold-out halves reported."""
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
NAME = "t2512-r3-bgrat-split"
out = json.load(open(HERE.parents[3] / "eval/suites/probes-numeric/cases" / f"{NAME}.oracle.json"))["sheets"]["Calc"]
meta = json.load(open(HERE / f"{NAME}.meta.json"))
C = {(c["a"], c["b"], c["x"]): c for c in score.cells()}
mpmath.mp.dps = 40
g = defaultdict(list)


def Qx(b, z):
    return mpmath.gammainc(b, z, mpmath.inf, regularized=True)


for k, a, b0, x, case, _cell in meta:
    v = out.get(f"A{k}")
    if not v or v.get("t") != "n":
        continue
    c = C[(a, b0, x)]
    dx, F, lnx = v["v"], c["F"], c["lnx"]
    half = "hold" if c["row"]["hold"] else "tune"
    nu_d = a + 0.5 * (b0 - 0.5 - 0.5)
    nu_e = mpmath.mpf(a) + (mpmath.mpf(b0) - 1) / 2
    z_d = c["z"]
    z_87 = float(-(np.longdouble(nu_d) * np.longdouble(lnx)))
    cands = {
        "CR": F,
        "zs/ze(nu exact)": F * Qx(b0, z_d) / Qx(b0, -nu_e * lnx),
        "zs/ze(nu double)": F * Qx(b0, z_d) / Qx(b0, -mpmath.mpf(nu_d) * lnx),
        "z87/ze(nu double)": F * Qx(b0, z_87) / Qx(b0, -mpmath.mpf(nu_d) * lnx),
    }
    for m, val in cands.items():
        g[(m, half)].append((float(val), dx))
        g[(m, "ALL")].append((float(val), dx))
print("| model | half | n | exact/w4/w15 |")
print("|---|---|---|---|")
for k in sorted(g):
    n, e, f, w = sl.tally(g[k])
    print(f"| {k[0]} | {k[1]} | {n} | {e}/{f}/{w} |")
