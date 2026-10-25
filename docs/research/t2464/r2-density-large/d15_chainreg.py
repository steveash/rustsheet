"""Which partial sums of E = C - b1 - b2 are stored?  Regress Excel's density error on the rounding errors (units
2^-53, absolute in E) of candidate partial sums, all from exact b1 = bd0(k, fl(n x)), b2 = bd0(n-k, fl(n fl(1-x)))
and C = ln(pdf) + b1 + b2 (exact):  X=-b1-b2, C-b1, C-b2, E, and with the Loader split C = L5 + Cs - h:
Cs-b1, Cs-b1-b2 (lc), lc-h, L5-h+... (K+lc).  Usage: d15_chainreg.py BLOCK ..."""
import sys
import numpy as np
import mpmath
import dlib as L
import model as M

U = mpmath.mpf(2) ** 53
rows = L.load()


def bd0x(k, M_):
    return k * mpmath.log(k / M_) + M_ - k


names = ["1", "X", "C-b1", "C-b2", "E", "Cs-b1", "lc", "lc-h", "Cs-b2"]
for blk in sys.argv[1:]:
    idx = L.blocks(rows)[blk]
    r0 = rows[idx[0]]
    a, b = r0["a"], r0["b"]
    k, n, nk, Cs, lf, L5 = M.shape_consts(a, b, {})
    k, n, nk = mpmath.mpf(k), mpmath.mpf(n), mpmath.mpf(nk)
    h = lf / 2
    F, Y = [], []
    for i in idx:
        r = rows[i]
        X_ = mpmath.mpf(r["x"])
        q = mpmath.mpf(1.0 - r["x"])
        b1 = bd0x(k, mpmath.mpf(L.D(n * X_)))
        b2 = bd0x(nk, mpmath.mpf(L.D(n * q)))
        E = mpmath.log(r["exact"])
        C = E + b1 + b2
        lc = Cs - b1 - b2
        rd = lambda v: (mpmath.mpf(L.D(v)) - v) * U  # noqa: E731
        f = [1, rd(-b1 - b2), rd(C - b1), rd(C - b2), rd(E), rd(Cs - b1), rd(lc), rd(lc - h), rd(Cs - b2)]
        F.append([float(v) for v in f])
        Y.append(L.err_units(r["excel"], r["exact"]))
    F, Y = np.array(F), np.array(Y)
    keep = [j for j in range(F.shape[1]) if j == 0 or F[:, j].std() > 1e-3]
    c, *_ = np.linalg.lstsq(F[:, keep], Y, rcond=None)
    res = Y - F[:, keep] @ c
    print(f"{blk} ({a:g},{b:g}) sd {Y.std():.2f}->{res.std():.2f}: "
          + ", ".join(f"{names[j]}={v:+.2f}({F[:, j].std():.2f})" for j, v in zip(keep, c)), flush=True)
