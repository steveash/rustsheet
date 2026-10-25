"""Scan accumulation recurrences x term forms x arithmetic x stopping rule on a block (or group), free L (k ulps),
P = EXP(fl(fl(a*LN x) - L))/a, I = fl(P*S)."""
import sys, itertools
from lib import *
from series import accum2
M = mpmath.mpf
blk = sys.argv[1]; K = int(sys.argv[2]); Lmode = sys.argv[3] if len(sys.argv) > 3 else "ex"
rows = sorted([r for r in load(cum=True, branch="direct") if r["block"] == blk], key=lambda r: r["x"] + r["a"] + r["b"])
x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
if Lmode == "gl":
    ga, gb, gab = xl("GAMMALN", a), xl("GAMMALN", b), xl("GAMMALN", add(a, b))
    lnB0 = add(ga, sub(gb, gab))
else:
    lnB0 = np.array([float(mpmath.log(mpmath.beta(M(ai), M(bi)))) for ai, bi in zip(a, b)])
alx = mul(a, xl("LN", x))
Ps = []
for k in range(-K, K + 1):
    L = lnB0 + k * np.array([math.ulp(v) for v in lnB0])
    Ps.append((k, div(xl("EXP", sub(alx, L)), a)))
res = []
for mode, rec, tf, stop in itertools.product(("dbl", "x87"), ("R1", "R2", "R3", "R4", "R5", "R6", "R7"), ("T1", "T2", "T3", "T4"), ("unchanged", "rel")):
    S, nt = accum2(x, a, b, mode=mode, rec=rec, tf=tf, stop=stop)
    best = max((int((mul(P, S) == E).sum()), k) for k, P in Ps)
    res.append((best[0], best[1], mode, rec, tf, stop, nt.mean()))
res.sort(reverse=True)
print(blk, len(E), "a", a[0], "b", b[0])
for r in res[:20]:
    print(f"  {r[0]:5d} k={r[1]:+d} {r[2]} {r[3]} {r[4]} {r[5]}  terms {r[6]:.1f}")
print("  worst:", res[-1][:6])
