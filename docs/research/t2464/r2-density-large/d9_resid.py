"""Residual of observed E (interval midpoint) minus model C + X along a run, with the fitted C (s2_scan), plus the
model's own stored-intermediate rounding errors, to eyeball unmodelled structure.  Usage: d9_resid.py BLOCK knobs N"""
import math
import sys
import numpy as np
import mpmath
import dlib as L
import loader as LD
import s2_scan as S

U = mpmath.mpf(2) ** 53
blk, spec = sys.argv[1], sys.argv[2]
best, *_ = S.run(blk, spec, quiet=True)
C = best[1]
kw = S.parse(spec)
kw.pop("comb", None)
rows = S.rows
idx = sorted(L.blocks(rows)[blk], key=lambda i: rows[i]["x"])
res = []
for i in idx:
    r = rows[i]
    X, b1, b2 = LD.xpart(r["x"], r["a"], r["b"], **kw)
    e0 = float(mpmath.log(r["excel"]))
    u = math.ulp(e0)
    ok = [j for j in range(-8, 9) if L.exp_v6(mpmath.mpf(e0 + j * u)) == r["excel"]]
    lo, hi = e0 + (min(ok) - 0.5) * u, e0 + (max(ok) + 0.5) * u
    res.append((float((mpmath.mpf(lo) - (C + X)) * U), float((mpmath.mpf(hi) - (C + X)) * U)))
N = int(sys.argv[3]) if len(sys.argv) > 3 else 64
for j, (lo, hi) in enumerate(res[:N]):
    m = int(math.frexp(rows[idx[j]]["x"])[0] * 2**53) & 15
    print(f"{j:4d} low4={m:2d} [{lo:+6.2f},{hi:+6.2f}] {'*' if lo > 0 or hi < 0 else ''}")
mid = np.array([(a + b) / 2 for a, b in res])
for lag in (1, 2, 3, 4, 8, 16):
    print("autocorr lag", lag, np.corrcoef(mid[:-lag], mid[lag:])[0, 1].round(3))
