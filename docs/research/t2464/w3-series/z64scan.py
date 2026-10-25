"""z64 (extended EXP argument) with a fine offset on L (steps of 2^-62 absolute), new series s: can a precise
extended L make z64 fit the x-runs as well as z53 does?"""
import sys
from lib import *
from model import exponent
from series import accum2
M = mpmath.mpf
blk = sys.argv[1]; lo, hi, st = (float(v) for v in sys.argv[2:5])
rows = [r for r in load(cum=True, branch="direct") if r["block"] == blk]
x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
s1, _ = accum2(x, a, b, mode="x87", rec="R4", tf="T0", start="1/a")
z = exponent(x, a, b, big=8.0)
best = []
for k in np.arange(lo, hi + 1e-9, st):
    off = M(k) * M(2) ** -62
    e = np.array([fl53(mpmath.exp(fl64(v + off))) for v in z])
    best.append((int((mul(e, s1) == E).sum()), k))
best.sort(reverse=True)
print(blk, "z64 best offsets (count, k*2^-62):", best[:6])
e53 = xl("EXP", np.array([fl53(v) for v in z]))
print("z53:", int((mul(e53, s1) == E).sum()))
