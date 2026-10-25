"""Forward-accumulated series S (start at 1, add a*c_n/(a+n)) in dbl/x87/ext x term forms, times P = EXP(z)/a, free L."""
import sys
from lib import *
from series import accum_sum, bpser_sum
M = mpmath.mpf
blk = sys.argv[1]; K = int(sys.argv[2])
rows = sorted([r for r in load(cum=True, branch="direct") if r["block"] == blk], key=lambda r: r["x"])
x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
I = [exact_cdf(xi, ai, bi) for xi, ai, bi in zip(x, a, b)]
err = rel_units(E, I)
lnB0 = np.array([float(mpmath.log(mpmath.beta(M(ai), M(bi)))) for ai, bi in zip(a, b)])
alx = mul(a, xl("LN", x))
Ss = {}
for mode in ("dbl", "x87", "ext"):
    for term in ("ratio", "c", "c2", "c3"):
        Ss[f"accum {term} {mode}"] = accum_sum(x, a, b, mode=mode, term=term)
Ss["bpser ext"] = bpser_sum(x, a, b, mode="ext", tol_kind="unchanged")[0]
res = []
for k in range(-K, K + 1):
    L = lnB0 + k * np.array([math.ulp(v) for v in lnB0])
    ez = xl("EXP", sub(alx, L)); P = div(ez, a)
    for nm, S in Ss.items():
        m = mul(P, S)
        me = rel_units(m, I) if k == 0 else None
        res.append((int((m == E).sum()), nm, k))
best = {}
for c, nm, k in res:
    if c > best.get(nm, (0,))[0]: best[nm] = (c, k)
for nm, v in sorted(best.items(), key=lambda t: -t[1][0]):
    print(f"  {nm:20s} {v[0]:5d}/{len(E)} at k={v[1]:+d}")
