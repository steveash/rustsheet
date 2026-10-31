"""T-2512 r3-bgrat: probe t2512-r3-bgrat-split read against the L140 bup+bgrat cells it came from.

D = Excel's BETA.DIST(x, a, b0) (b0 = frac(b)): the bgrat increment Excel computes with w = 0, rounded once.
Scores (exact / within 4 ulp / w15):
  direct cells  : M0 (TOMS bgrat in doubles, Excel pieces, r2-tdist FINAL) and CR (exact F at the double lnx) vs D
  L140 cells    : M0; RN(w_M0 + D) (model bup sum plus Excel's own increment); RN(w_M0 + F) (EXQ);
                  RN(w_exact + D) with w_exact = exact I_x(a, b) - exact I_x(a, b0) (exact bup, Excel's increment)
Split by md5 hold-out half (nothing here is fitted)."""
from __future__ import annotations

import pickle
from collections import defaultdict
from pathlib import Path

import mpmath

import bscore as score
from common import split_d, table

HERE = Path(__file__).resolve().parent
D = split_d()
cs = [c for c in score.cells() if c["br"].startswith("L140 bup+bgrat")]
EXF = HERE / "exact_l140.pkl"
exl = pickle.load(open(EXF, "rb")) if EXF.exists() else {}
mpmath.mp.dps = 40


def exact_I(a, b, x):
    k = (a, b, x)
    if k not in exl:
        exl[k] = mpmath.betainc(a, b, 0, x, regularized=True)
    return exl[k]


g = defaultdict(list)
for c in cs:
    k = (c["a"], c["b"], c["x"])
    if k not in D:
        continue
    r = c["row"]
    dx = D[k]
    F_dbl = float(c["F"])
    ident = c["fn"](c["g"]) == c["g"]
    # the cell's own (a, b, x) and the exact bup sum w = I_x(a, b_cell) - I_x(a, b0) on the bgrat orientation
    bc = c["b"] + round(r["b"] - c["b"]) if ident else c["b"] + round(r["a"] - c["b"])
    wex = exact_I(c["a"], bc, c["x"]) - exact_I(c["a"], c["b"], c["x"])
    vals = {
        "M0": c["fn"](c["g"]),
        "RN(w_M0+D)": c["fn"](float(mpmath.mpf(c["w"]) + mpmath.mpf(dx))),
        "RN(w_M0+F)": c["fn"](float(mpmath.mpf(c["w"]) + c["F"])),
        "RN(w_ex+D)": c["fn"](float(wex + mpmath.mpf(dx))),
        "RN(w_ex+F)": c["fn"](float(wex + c["F"])),
    }
    half = "hold" if r["hold"] else "tune"
    g[("direct: D vs CR(F)", half)].append((F_dbl, dx))
    for m, v in vals.items():
        g[("L140: " + m, half)].append((v, r["excel"]))
pickle.dump(exl, open(EXF, "wb"))
table(g, ("comparison", "half"))
