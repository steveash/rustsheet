"""y-tie response of Excel's density: regress error (units 2^-53) on ey = rel err of fl(1-x), with standard error,
vs the predictions: Loader bd0(n-k, n*q) with q = fl(1-x): (b-1) - (a+b-2) y; log form (b-1)*ln(fl(1-x)): b-1;
brcomp/(x*y) with lny = log1p(-x) and the divide by fl(y): -1; 0 for no y."""
import numpy as np
import mpmath
import dlib as L

U = mpmath.mpf(2) ** 53
rows = L.load()
print("block (a,b) x : coef(se) | Loader  logform  brcomp/(xy)")
for blk, idx in sorted(L.blocks(rows).items()):
    if not blk.startswith("xruns"):
        continue
    Y, F = [], []
    for i in idx:
        r = rows[i]
        x = mpmath.mpf(r["x"])
        y = 1.0 - r["x"]
        Y.append(L.err_units(r["excel"], r["exact"]))
        F.append([1.0, float((mpmath.mpf(y) - (1 - x)) / (1 - x) * U)])
    F, Y = np.array(F), np.array(Y)
    if np.std(F[:, 1]) < 1e-9:
        continue
    c, *_ = np.linalg.lstsq(F, Y, rcond=None)
    res = Y - F @ c
    se = np.sqrt(np.sum(res**2) / (len(Y) - 2) / np.sum((F[:, 1] - F[:, 1].mean()) ** 2))
    r0 = rows[idx[0]]
    a, b, x = r0["a"], r0["b"], r0["x"]
    print(f"{blk} ({a:g},{b:g}) {x:.4f}: {c[1]:+6.2f}({se:.2f}) | {(b-1)-(a+b-2)*(1-x):+6.2f} {b-1:+6.2f} {-1:+6.2f}")
