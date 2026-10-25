"""Regress Excel's density error (units 2^-53 rel) on candidate intermediate rounding errors, per x-run block.
Features: eLNx = LN(x)-ln x; ey = rel err of fl(1-x); elog1p = Excel log1p(-x) - log1p(-x); eLNy = LN(fl(1-x)) -
ln(fl(1-x)); exy = rel err of fl(x*fl(1-x)); all in units of 2^-53."""
import math
import numpy as np
import mpmath
import dlib as L

U = mpmath.mpf(2) ** 53
rows = L.load()
for blk, idx in sorted(L.blocks(rows).items()):
    if not blk.startswith("xruns"):
        continue
    F, Y = [], []
    for i in idx:
        r = rows[i]
        x = mpmath.mpf(r["x"])
        y = 1.0 - r["x"]
        Y.append(L.err_units(r["excel"], r["exact"]))
        f = [1.0]
        f.append(float((L.LN(r["x"]) - mpmath.log(x)) * U))
        f.append(float((mpmath.mpf(y) - (1 - x)) / (1 - x) * U))
        f.append(float((L.log1p_xl(-r["x"]) - mpmath.log1p(-x)) * U) if r["x"] <= 0.375 else 0.0)
        f.append(float((L.LN(y) - mpmath.log(mpmath.mpf(y))) * U))
        xy = L.D(x * mpmath.mpf(y))
        f.append(float((mpmath.mpf(xy) - x * mpmath.mpf(y)) / (x * mpmath.mpf(y)) * U))
        F.append(f)
    F, Y = np.array(F), np.array(Y)
    keep = [j for j in range(F.shape[1]) if np.std(F[:, j]) > 1e-9 or j == 0]
    c, res, *_ = np.linalg.lstsq(F[:, keep], Y, rcond=None)
    pred = F[:, keep] @ c
    names = ["1", "eLNx", "ey", "elog1p", "eLNy", "exy"]
    r0 = rows[idx[0]]
    print(f"{blk} ({r0['a']:g},{r0['b']:g}) x~{r0['x']:.4f} sd {np.std(Y):.2f} -> resid sd {np.std(Y - pred):.2f}: "
          + ", ".join(f"{names[k]}={v:+.2f}" for k, v in zip(keep, c)))
