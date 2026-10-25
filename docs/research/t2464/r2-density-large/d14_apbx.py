"""Regress Excel's density error (units 2^-53 rel) on the absolute rounding error of fl((a+b)x) (units 2^-53) and of
fl((a+b)y), y = fl(1-x), plus the q-tie ey, per x-run block; compare with the sensitivities d lnf/d(np) = k/np - 1 and
d lnf/d(nq) = (n-k)/nq - 1 (Loader with np or nq formed from (a+b)x)."""
import numpy as np
import mpmath
import dlib as L

U = mpmath.mpf(2) ** 53
rows = L.load()
for blk, idx in sorted(L.blocks(rows).items()):
    if not blk.startswith("xruns"):
        continue
    r0 = rows[idx[0]]
    a, b = r0["a"], r0["b"]
    s = mpmath.mpf(L.D(mpmath.mpf(a) + b))
    F, Y = [], []
    for i in idx:
        r = rows[i]
        x = mpmath.mpf(r["x"])
        y = mpmath.mpf(1.0 - r["x"])
        sx, sy = s * x, s * y
        F.append([1.0, float((mpmath.mpf(L.D(sx)) - sx) * U), float((mpmath.mpf(L.D(sy)) - sy) * U),
                  float((y - (1 - x)) / (1 - x) * U)])
        Y.append(L.err_units(r["excel"], r["exact"]))
    F, Y = np.array(F), np.array(Y)
    keep = [j for j in range(4) if j == 0 or F[:, j].std() > 1e-9]
    c, *_ = np.linalg.lstsq(F[:, keep], Y, rcond=None)
    res = Y - F[:, keep] @ c
    x = r0["x"]
    k, n = a - 1, a + b - 2
    names = ["1", "e(sx)", "e(sy)", "ey"]
    print(f"{blk} ({a:g},{b:g}) x {x:.4f}: sd {Y.std():.2f}->{res.std():.2f} "
          + ", ".join(f"{names[j]}={v:+.2f}" for j, v in zip(keep, c))
          + f" | k/np-1={k / (n * x) - 1:+.2f} (n-k)/nq-1={(n - k) / (n * (1 - x)) - 1:+.2f}")
