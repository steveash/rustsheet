"""b3: regress the per-cell needed constant (centre of the admissible interval, units) on rounding errors of the
x-side pieces, with one intercept per run: e_mx = rel. error of fl(m x), e_r = rel. error of r = fl(mx/n) given mx,
e_s = rel. error of s = fl(1 + r) given r, e_lx = abs. error of LN(x) (units of 2^-53), all in units.
Usage: b3_reg.py SET [run-prefix] [min_r]"""

from __future__ import annotations

import sys

import numpy as np

import blib as B

C, SL, mp = B.C, B.SL, B.mp


def regressors(x, m, n):
    X, M, N = mp.mpf(x), mp.mpf(m), mp.mpf(n)
    mx = SL.st(SL.LDB(m) * SL.LDB(x))
    side, r = SL.side_r(x, m, n)
    s = SL.st(SL.LDB(1) + SL.LDB(r))
    rex = mp.mpf(mx) / N if side == 1 else N / mp.mpf(mx)
    e_mx = (mp.mpf(mx) / (M * X) - 1) * B.U
    e_r = (mp.mpf(r) / rex - 1) * B.U
    e_s = (mp.mpf(s) / (1 + mp.mpf(r)) - 1) * B.U
    e_lx = (C.ex(SL.LN(x)) - mp.log(X)) * B.U
    return [float(e_mx), float(e_r), float(e_s), float(e_lx)]


def cells(rows, pref=None, min_r=0.0):
    out = {}
    for pc, E0, (lo, hi), r in C.run_cells(rows):
        if pref and not r["blk"].startswith(pref) or pc["r"] < min_r:
            continue
        K = C.K_exact(r["d1"], r["d2"], pc["side"])
        base = float((E0 - C.ex(pc["P"]) - C.ex(pc["H"]) - K) * B.U)
        out.setdefault((r["d1"], r["d2"], pc["side"], r["blk"][:2]), []).append(
            (base + (lo + hi) / 2, regressors(r["x"], r["d1"], r["d2"]), pc))
    return out


if __name__ == "__main__":
    g = cells(B.load(sys.argv[1]), sys.argv[2] if len(sys.argv) > 2 else None,
              float(sys.argv[3]) if len(sys.argv) > 3 else 0.0)
    keys = sorted(g)
    ys, X = [], []
    for j, k in enumerate(keys):
        for y, reg, _ in g[k]:
            row = [0.0] * len(keys)
            row[j] = 1.0
            ys.append(y)
            X.append(row + reg)
    ys, X = np.array(ys), np.array(X)
    coef, *_ = np.linalg.lstsq(X, ys, rcond=None)
    res = ys - X @ coef
    print("cells", len(ys), "runs", len(keys), "coef e_mx e_r e_s e_lx:", np.round(coef[-4:], 3), "rms", round(float(np.sqrt(np.mean(res**2))), 3))
    X0 = X[:, : len(keys)]
    c0, *_ = np.linalg.lstsq(X0, ys, rcond=None)
    print("intercepts only rms", round(float(np.sqrt(np.mean((ys - X0 @ c0) ** 2))), 3))
