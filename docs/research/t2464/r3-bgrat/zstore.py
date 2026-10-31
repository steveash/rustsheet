"""T-2512 r3-bgrat: does the stored z = fl(-nu lnx) show in Excel's direct bgrat cells (probe -split)?

D_model = RN(F * Q(b0, z_s) / Q(b0, z_e)): Q exact, z_s the stored double, z_e unrounded (-nu lnx exact, with nu
exact or the double fl(a + 0.5 (b0 - 1))). Also z_s from x87: fl53(fl64(nu * lnx)). Hold-out halves reported."""
from __future__ import annotations

from collections import defaultdict

import mpmath
import numpy as np

import bscore as score
from common import split_cells, table

C = {(c["a"], c["b"], c["x"]): c for c in score.cells()}
mpmath.mp.dps = 40
g = defaultdict(list)


def Qx(b, z):
    return mpmath.gammainc(b, z, mpmath.inf, regularized=True)


for a, b0, x, _case, dx in split_cells():
    c = C[(a, b0, x)]
    F, lnx = c["F"], c["lnx"]
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
table(g, ("model", "half"))
