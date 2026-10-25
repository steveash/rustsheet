"""b9: at the x = 1 cells, regress Excel's E error (centre of the admissible interval minus the exact ln F(1), units)
on the rounding errors of candidate stored doubles (fl(v) - v, units, exact v): E itself, A = -1/2 log1p(w/o),
B = 1/2 ln(w/2pi), A + B, ln w, ln o, ln h, 1/2 ln(w o / h), ln(w o/(2 pi h)), and Excel's LN errors at w, o, h.
Usage: b9_x1reg.py SET col,col,..."""

from __future__ import annotations

import sys

import numpy as np

import b6_x1 as X
import blib as B

mp, U = B.mp, B.U


def fe(v):
    return float((mp.mpf(float(v)) - v) * U)


def columns(m, n):
    a, b, h, side, w, o = X.shape(m, n)
    A = -mp.log1p(mp.mpf(w) / o) / 2
    Bv = mp.log(mp.mpf(w) / (2 * mp.pi)) / 2
    D = X.stx(h) - X.stx(a) - X.stx(b)
    E = A + Bv + D
    c = {"eE": fe(E), "eA": fe(A), "eB": fe(Bv), "eAB": fe(A + Bv), "elw": fe(mp.log(w)), "elo": fe(mp.log(o)),
         "elh": fe(mp.log(h)), "ehwoh": fe(mp.log(mp.mpf(w) * o / h) / 2), "ewoh2pi": fe(mp.log(mp.mpf(w) * o / h / (2 * mp.pi))),
         "eF": float((mp.mpf(float(mp.exp(E))) / mp.exp(E) - 1) * U), "one": 1.0}
    for nm, v in (("LNw", w), ("LNo", o), ("LNh", h)):
        c["x" + nm] = float((B.C.ex(X.LN(v)) - mp.log(v)) * U)
    return c, E


if __name__ == "__main__":
    names = sys.argv[2].split(",")
    ys, Xs = [], []
    for r in B.load(sys.argv[1]):
        if r["x"] != 1.0:
            continue
        t = X.target(r)
        if t is None:
            continue
        c, E = columns(r["d1"], r["d2"])
        ys.append(float(((t[0] + t[1]) / 2 - E) * U))
        Xs.append([c[k] for k in names])
    y, M = np.array(ys), np.array(Xs)
    coef, *_ = np.linalg.lstsq(M, y, rcond=None)
    res = y - M @ coef
    print(dict(zip(names, np.round(coef, 3))), "rms", round(float(np.sqrt(np.mean(res**2))), 3), "raw rms",
          round(float(np.sqrt(np.mean(y**2))), 3), "n", len(y))
