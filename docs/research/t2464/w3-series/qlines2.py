"""Spectral line of the double rounding of each series-internal quantity (exact values) on an x-run block:
bpser terms c_n, w_n, partial sums, a*sum, and Cephes t_n, v_n, s partials."""
import sys
from lib import *
M = mpmath.mpf
blk = sys.argv[1]; target = [int(v) for v in sys.argv[2].split(",")] if len(sys.argv) > 2 else []
rows = sorted([r for r in load(cum=True, branch="direct") if r["block"] == blk], key=lambda r: r["x"])
x = [M(r["x"]) for r in rows]; A = M(rows[0]["a"]); B = M(rows[0]["b"])
def peak(vals):
    e = np.array([float(M(float(v)) / v - 1) / U if v != 0 else 0 for v in vals])
    w = e - e.mean(); F = np.abs(np.fft.rfft(w)) ** 2
    t = int(np.argmax(F[1:]) + 1); return t, F[t] / F.mean(), [int(F[k] / F.mean()) for k in target]
N = 12
c = [M(1)] * len(x); sm = [M(0)] * len(x)
qs = {}
for n in range(1, N + 1):
    c = [ci * (1 - B / n) * xi for ci, xi in zip(c, x)]
    w = [ci / (A + n) for ci in c]
    sm = [s + wi for s, wi in zip(sm, w)]
    qs[f"c{n}"] = c; qs[f"w{n}"] = w; qs[f"sum{n}"] = sm
    qs[f"a*sum{n}"] = [A * s for s in sm]; qs[f"1+a*sum{n}"] = [1 + A * s for s in sm]
    qs[f"1/a+sum{n}"] = [1 / A + s for s in sm]
for nm, v in qs.items():
    t, p, tp = peak(v)
    print(f"  {nm:12s} peak {t:4d} ({p:.0f})  power at {target}: {tp}")
