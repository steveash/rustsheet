"""Regress Excel's density error on the individual roundings of Loader's bd0 log form x ln(x/np) + np - x (stored
x87 ops, Excel LN), both sides (p: bd0(k, n x), q: bd0(n-k, n fl(1-x))). Coefficient 1 = that rounding is present."""
import sys
import numpy as np
import mpmath
import dlib as L

U = mpmath.mpf(2) ** 53
rows = L.load()


def feats(k, M):
    k, M = mpmath.mpf(k), mpmath.mpf(M)
    r = mpmath.mpf(L.D(k / M))
    l = mpmath.mpf(L.LN(float(r)))
    t = mpmath.mpf(L.D(k * l))
    s = mpmath.mpf(L.D(t + M))
    u = mpmath.mpf(L.D(s - k))
    return [-k * (r - k / M) / (k / M), -k * (l - mpmath.log(r)), -(t - k * l), -(s - (t + M)), -(u - (s - k))]


names = ["1", "p.div", "p.ln", "p.mul", "p.add", "p.sub", "q.div", "q.ln", "q.mul", "q.add", "q.sub"]
for blk in sys.argv[1:]:
    idx = L.blocks(rows)[blk]
    F, Y = [], []
    for i in idx:
        r = rows[i]
        a, b, x = r["a"], r["b"], r["x"]
        k, n = a - 1, a + b - 2
        q = 1.0 - x
        f = [1.0] + [float(v * U) for v in feats(k, L.D(mpmath.mpf(n) * mpmath.mpf(x)))] + \
            [float(v * U) for v in feats(n - k, L.D(mpmath.mpf(n) * mpmath.mpf(q)))]
        F.append(f)
        Y.append(L.err_units(r["excel"], r["exact"]))
    F, Y = np.array(F), np.array(Y)
    keep = [j for j in range(F.shape[1]) if j == 0 or np.std(F[:, j]) > 1e-6]
    c, *_ = np.linalg.lstsq(F[:, keep], Y, rcond=None)
    res = Y - F[:, keep] @ c
    print(f"{blk}: sd {Y.std():.2f} -> {res.std():.2f}; " + ", ".join(f"{names[j]}={v:+.2f}" for j, v in zip(keep, c)))
    print("    feature sds: " + ", ".join(f"{names[j]}={F[:, j].std():.2f}" for j in keep[1:]))
