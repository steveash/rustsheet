"""Recurrence / term-form / stopping scan for the forward accumulation from 1/a (x87), prefactor fixed per block:
z53 with offset k53/4 ulp(z) or z64 with k64/16 ulp(z)."""
import sys, itertools
from lib import *
from model import exponent
from series import accum2
M = mpmath.mpf
blk, zm, k = sys.argv[1], sys.argv[2], int(sys.argv[3])
rows = [r for r in load(cum=True, branch="direct") if r["block"] == blk]
x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
z = exponent(x, a, b, big=8.0); zs = [M(math.ulp(float(v))) for v in z]
if zm == "z53":
    P = xl("EXP", np.array([fl53(v + M(k) / 4 * u) for v, u in zip(z, zs)]))
else:
    P = np.array([fl53(mpmath.exp(fl64(v + M(k) / 16 * u))) for v, u in zip(z, zs)])
res = []
for mode, rec, stop in itertools.product(("x87", "dbl"), ("R1", "R2", "R3", "R4", "R5", "R6", "R7"), ("unchanged", "rel", "abs")):
    s1, nt = accum2(x, a, b, mode=mode, rec=rec, tf="T0", start="1/a", stop=stop, eps=2.0 ** -53)
    res.append((int((mul(P, s1) == E).sum()), mode, rec, stop, nt.mean()))
res.sort(reverse=True)
for r in res[:10]: print("  %4d %s %s %-9s terms %.1f" % r)
print("  ... worst", res[-1])
