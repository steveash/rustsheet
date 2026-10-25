"""Density-level regression (Excel error, units 2^-53) on candidate roundings for the Loader chain, computed from exact
values: tp/tq = rounding of fl(k ln(k/np)) / fl((n-k) ln((n-k)/nq)) (a stored log-form product with an accurate log),
b1/b2 = rounding of the stored bd0 values, lc = fl(Cs + X), g = fl(lc - h) (h = h0 - 4 ulp), E = fl(L5 + g),
sm = rounding of fl(k + np) (series variable).  Usage: d11_reg2.py BLOCK ..."""
import math
import sys
import numpy as np
import mpmath
import dlib as L
import s4_chain as C4

U = mpmath.mpf(2) ** 53
rows = C4.rows
for blk in sys.argv[1:]:
    idx = L.blocks(rows)[blk]
    r0 = rows[idx[0]]
    a, b = r0["a"], r0["b"]
    k, n = mpmath.mpf(a - 1), mpmath.mpf(a + b - 2)
    nk = n - k
    Cs = C4.A.stirlerr(float(n)) - C4.A.stirlerr(float(k)) - C4.A.stirlerr(float(nk))
    h0 = float(C4.lf_ideal(k, n) / 2)
    h = mpmath.mpf(h0 - 4 * math.ulp(h0))
    L5 = mpmath.mpf(L.LN(a + b - 1))
    F, Y = [], []
    for i in idx:
        r = rows[i]
        x = mpmath.mpf(r["x"])
        q = mpmath.mpf(1.0 - r["x"])
        npx, nqx = mpmath.mpf(L.D(n * x)), mpmath.mpf(L.D(n * q))
        tp, tq = k * mpmath.log(k / npx), nk * mpmath.log(nk / nqx)
        b1, b2 = tp + npx - k, tq + nqx - nk
        lc = Cs - b1 - b2
        g = lc - h
        E = L5 + mpmath.mpf(L.D(g))
        rd = lambda v: mpmath.mpf(L.D(v)) - v  # noqa: E731
        f = [1 / U, -rd(tp), -rd(tq), -rd(b1), -rd(b2), rd(lc), rd(g), rd(E), -rd(npx + k) / (npx + k) * b1,
             -rd(nqx + nk) / (nqx + nk) * b2]
        F.append([float(v * U) for v in f])
        Y.append(L.err_units(r["excel"], r["exact"]))
    names = ["1", "tp", "tq", "b1", "b2", "lc", "g", "E", "smp", "smq"]
    F, Y = np.array(F), np.array(Y)
    keep = [j for j in range(F.shape[1]) if j == 0 or F[:, j].std() > 1e-3]
    c, *_ = np.linalg.lstsq(F[:, keep], Y, rcond=None)
    res = Y - F[:, keep] @ c
    print(f"{blk} ({a:g},{b:g}) sd {Y.std():.2f} -> {res.std():.2f}; "
          + ", ".join(f"{names[j]}={v:+.2f}(sd{F[:, j].std():.2f})" for j, v in zip(keep, c)), flush=True)
