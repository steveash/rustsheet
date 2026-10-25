"""Which double exponent drives T.DIST(t, n, FALSE) at large |E|: for each candidate E_c (a double, built with
Excel's LN and stored x87 ops from the double t), the predicted relative error is (E_c - E_exact) in units of
2^-53 (E_exact = -(n+1)/2 ln(1 + t^2/n) at the exact t). Excel's error is regressed on each, per |E| band.
usage: pdf_ereg.py"""
import math

import mpmath
import numpy as np

import tlib
import xlk

bx, sx = tlib.bx, tlib.bx.sx
U = 2.0**-53


def cands(t, n):
    h = (n + 1) / 2
    tt = t * t
    u = n + tt
    q = tt / n
    y = n / u
    l1p = xlk.excel_log1p.log1p_series(q) if q <= 0.375 else bx.LN(1.0 + q)
    with mpmath.workdps(40):
        lnn = mpmath.log(n)
        c = {
            "h*LN(u)-": -sx("*", h, bx.LN(u)) + h * lnn,  # E relative to the exact h ln n (constant)
            "h*LN(1+q)": -sx("*", h, bx.LN(1.0 + q)),
            "h*l1p(q)": -sx("*", h, l1p),
            "h*LN(y)": sx("*", h, bx.LN(y)),
            "n/2*LN(1+q)+LN/2": -(sx("*", n / 2, bx.LN(1.0 + q)) + 0.5 * bx.LN(1.0 + q)),
            "h*(LN(u)-LN(n))": -sx("*", h, sx("-", bx.LN(u), bx.LN(n))),
            "exactL_fl(h*L)": -float(h * mpmath.log1p(mpmath.mpf(tt) / n)),
        }
    return c


rows = [r for r in tlib.load(("pdf",)) if r["t"] != 0 and r["excel"] != 0 and r["n"] <= 1e6]
data = []
for r in rows:
    t, n = abs(r["t"]), r["n"]
    with mpmath.workdps(40):
        T = mpmath.mpf(t)
        Ex = -(n + 1) / 2 * mpmath.log1p(T * T / n)
        c = cands(t, n)
        pred = {k: float((mpmath.mpf(v) - Ex)) / U if not k.startswith("h*LN(u)") else
                float(mpmath.mpf(v) - Ex) / U for k, v in c.items()}
    ex = tlib.exact_pdf(t, n)
    data.append((float(abs(Ex)), (r["excel"] - ex) / ex / U, pred))
names = list(data[0][2])
for lo, hi in ((0, 1), (1, 3), (3, 6), (6, 12), (12, 1e9)):
    sel = [d for d in data if lo <= d[0] < hi]
    if len(sel) < 20:
        continue
    Y = np.array([d[1] for d in sel])
    print(f"|E| in [{lo},{hi}) n={len(sel)} excel err sd {Y.std():.2f}")
    for k in names:
        X = np.array([d[2][k] for d in sel])
        A = np.column_stack([X, np.ones(len(X))])
        coef, *_ = np.linalg.lstsq(A, Y, rcond=None)
        res = Y - A @ coef
        print(f"   {k:22} slope {coef[0]:+.3f} corr {np.corrcoef(X, Y)[0, 1]:+.3f} resid sd {res.std():.2f}"
              f"  sd(pred - excel) {np.std(Y - X):.2f}")
