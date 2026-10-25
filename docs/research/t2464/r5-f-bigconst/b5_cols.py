"""b5: candidate error columns (units 2^-53, absolute in ln) per (shape, side) for the x = 1 constant; regress
need (b4's need_SET.tsv) on them. Columns use Excel's LN (slib.LN), Excel's log1p (slib.L_xl) and doubles as Excel
stores them.
Usage: b5_cols.py SET col,col,...   (cols: see COLS; 'side' = M-sd's bracket, 'one' = intercept)"""

from __future__ import annotations

import sys

import numpy as np

import blib as B

SL, mp, LDB, st = B.SL, B.mp, B.LDB, B.st
U = B.U


def ex(v):
    return B.C.ex(v)


def cols(m, n, side):
    a, b, h = m / 2, n / 2, (m + n) / 2
    w, o = (a, b) if side == 1 else (b, a)
    q = st(LDB(w) / LDB(o))
    Lq = SL.L_xl(q)
    lq_ex = mp.log1p(ex(q))
    l_ex = mp.log1p(mp.mpf(w) / o)
    out = {}
    out["eL"] = (ex(Lq) - lq_ex) * U  # Excel's log1p error at fl(w/o)
    out["er"] = (lq_ex - l_ex) * U  # rounding of w/o, through log1p
    out["heL"] = h * out["eL"]
    out["her"] = h * out["er"]
    for nm, v in (("w", w), ("o", o), ("h", h), ("2pi", 2 * float(mp.pi)), ("pi", float(mp.pi))):
        out["eLN" + nm] = (ex(SL.LN(v)) - mp.log(mp.mpf(v) if nm not in ("2pi", "pi") else mp.mpf(v))) * U
    out["e2pi"] = (mp.mpf(2 * float(mp.pi)) - 2 * mp.pi) / (2 * mp.pi) * U  # rel error of the double 2pi
    out["ehalfw2pi"] = (ex(SL.LN(st(LDB(w) / LDB(2 * float(mp.pi))))) - mp.log(mp.mpf(w) / (2 * mp.pi))) * U
    P = (LDB(h - 0.5) * Lq)
    out["pst"] = (mp.mpf(st(P)) - ex(P)) * U  # rounding of the stored product (h-1/2) L
    hL = LDB(h) * Lq
    out["hst"] = (mp.mpf(st(hL)) - ex(hL)) * U
    out["one"] = mp.mpf(1)
    out["inv_w"] = mp.mpf(1) / w
    return {k: float(v) for k, v in out.items()}


if __name__ == "__main__":
    d = np.genfromtxt(f"need_{sys.argv[1]}.tsv", names=True, delimiter="\t")
    names = sys.argv[2].split(",")
    X = []
    for m, n, s, sd in zip(d["d1"], d["d2"], d["side"], d["msd_side"]):
        c = cols(float(m), float(n), int(s))
        c["side"] = sd
        X.append([c[k] for k in names])
    X = np.array(X)
    y = d["need"]
    coef, *_ = np.linalg.lstsq(X, y, rcond=None)
    res = y - X @ coef
    print(dict(zip(names, np.round(coef, 3))), "rms", round(float(np.sqrt(np.mean(res**2))), 3),
          "|res|<1:", int(np.sum(abs(res) < 1)), "of", len(y))
