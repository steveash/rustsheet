"""Shape-constant pieces on the sweeps: regress need (c3 method) on the rounding errors of candidate shape-only pieces
(units 2^-53 absolute in E, signed as they would enter E = LN(a+b-1) + Cs - lf/2):
  dS   : fl(a+b) - (a+b), propagated through the exact constant: dC/d(n) * dS  (n = fl(a+b) - 2 used everywhere)
  L5   : LN(fl(a+b)-1) error;  Lk: -LN(k)/2 error; Lnk: -LN(n-k)/2 error; Ln: +LN(n)/2 error
  GLs  : -(GL(a)+GL(b)-GL(a+b)) error (Excel GAMMALN in a beta-function constant)"""
import sys
import numpy as np
import mpmath
import dlib as L
import loader as LD
import model as M

U = mpmath.mpf(2) ** 53
rows = L.load()


def Cfun(A, B, k, n):
    nk = n - k
    return mpmath.log(A + B - 1) + mpmath.loggamma(n + 1) - mpmath.loggamma(k + 1) - mpmath.loggamma(nk + 1) \
        - (n + 0.5) * mpmath.log(n) + (k + 0.5) * mpmath.log(k) + (nk + 0.5) * mpmath.log(nk) \
        - 0.5 * (mpmath.log(2 * mpmath.pi) + mpmath.log(k) + mpmath.log(nk) - mpmath.log(n))


names = ["1", "dS", "L5", "Lk", "Lnk", "Ln", "GLs"]
for blk in sys.argv[1:]:
    idx = L.blocks(rows)[blk]
    F, Y = [], []
    for i in idx:
        r = rows[i]
        a, b = r["a"], r["b"]
        A, B = mpmath.mpf(a), mpmath.mpf(b)
        k, n = A - 1, A + B - 2
        nk = n - k
        Cex = -mpmath.log(mpmath.beta(A, B)) + k * mpmath.log(k / n) + nk * mpmath.log(nk / n)
        X = LD.xpart(r["x"], a, b, **M.BEST_X)[0]
        Y.append(L.err_units(r["excel"], r["exact"]) - L.err_units(L.exp_v6(mpmath.mpf(L.D(Cex + X))), r["exact"]))
        S = mpmath.mpf(L.D(A + B))
        dC = (Cfun(A, B, k, S - 2) - Cfun(A, B, k, n)) * U
        lnv = lambda v: (mpmath.mpf(L.LN(float(v))) - mpmath.log(v)) * U  # noqa: E731
        gls = (-(mpmath.mpf(L.GL(a)) + L.GL(b) - L.GL(float(S))) + mpmath.log(mpmath.beta(A, B))) * U
        F.append([1.0, float(dC), float(lnv(S - 1)), float(-lnv(k) / 2), float(-lnv(S - 2 - k) / 2), float(lnv(S - 2) / 2),
                  float(gls)])
    F, Y = np.array(F), np.array(Y)
    keep = [j for j in range(F.shape[1]) if j == 0 or F[:, j].std() > 1e-6]
    c, *_ = np.linalg.lstsq(F[:, keep], Y, rcond=None)
    res = Y - F[:, keep] @ c
    print(f"{blk}: need sd {Y.std():.2f} -> {res.std():.2f}; "
          + ", ".join(f"{names[j]}={v:+.2f}(sd{F[:, j].std():.2f})" for j, v in zip(keep, c)), flush=True)
