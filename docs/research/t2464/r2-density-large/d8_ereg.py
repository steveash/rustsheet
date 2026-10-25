"""Observed E (midpoint of the real interval whose stored double maps to Excel's density through EXP_V6) minus the
model C + X, regressed on the rounding errors of candidate stored intermediates (units 2^-53, signed as they enter E):
bd0 results, X, lc = Cs + X, lc - h, and the log-form internals.  Usage: d8_ereg.py BLOCK 'knobs'"""
import math
import sys
import numpy as np
import mpmath
import dlib as L
import loader as LD
import s2_scan as S

sys.path.append(str(L.T / "w6-literature"))
import h2_loader as H  # noqa: E402

U = mpmath.mpf(2) ** 53
blk, spec = sys.argv[1], sys.argv[2]
best, *_ = S.run(blk, spec, quiet=True)
C = best[1]
kw = S.parse(spec)
kw.pop("comb", None)
rows = S.rows
idx = L.blocks(rows)[blk]
r0 = rows[idx[0]]
a, b = r0["a"], r0["b"]
k, n = a - 1, a + b - 2
A = H.Arith("D")
Cs = A.stirlerr(n) - A.stirlerr(k) - A.stirlerr(n - k)
h = C - Cs  # E = lc + h, so h = K
F, Y = [], []
names = ["1", "D(b1)", "D(b2)", "D(X)", "D(lc)", "D(lc+K)", "D(np-k)", "D(nq-nk)"]
for i in idx:
    r = rows[i]
    X, b1, b2 = LD.xpart(r["x"], a, b, **kw)
    e0 = float(mpmath.log(r["excel"]))
    u = math.ulp(e0)
    ok = [j for j in range(-8, 9) if L.exp_v6(mpmath.mpf(e0 + j * u)) == r["excel"]]
    if not ok:
        continue
    lo, hi = e0 + (min(ok) - 0.5) * u, e0 + (max(ok) + 0.5) * u
    obs = (mpmath.mpf(lo) + mpmath.mpf(hi)) / 2
    lc = Cs + X
    npx = mpmath.mpf(L.D(mpmath.mpf(n) * mpmath.mpf(r["x"])))
    nqx = mpmath.mpf(L.D(mpmath.mpf(n) * mpmath.mpf(1.0 - r["x"])))
    f = [1.0]
    for v, sgn in ((b1, -1), (b2, -1), (X, 1), (lc, 1), (lc + h, 1), (npx - k, -1), (nqx - (n - k), -1)):
        f.append(float(sgn * (mpmath.mpf(L.D(v)) - v) * U))
    F.append(f)
    Y.append(float((obs - (C + X)) * U))
F, Y = np.array(F), np.array(Y)
keep = [j for j in range(F.shape[1]) if j == 0 or F[:, j].std() > 1e-6]
c, *_ = np.linalg.lstsq(F[:, keep], Y, rcond=None)
res = Y - F[:, keep] @ c
print(f"{blk} ({a:g},{b:g}) [{spec}] n={len(Y)} sd {Y.std():.3f} -> {res.std():.3f}; "
      + ", ".join(f"{names[j]}={v:+.2f}(sd {F[:, j].std():.2f})" for j, v in zip(keep, c)))
