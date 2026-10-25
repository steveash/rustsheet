"""a-sweep: Cephes-style accumulation s = 1/a + sum c_n/(a+n) (start at 1/a), I = EXP(z) * s (no division by a),
versus S = 1 + sum a c_n/(a+n), I = EXP(z)/a * S. la exact-rounded or GL(a); L = la + (GLb - GLab)."""
import sys, itertools
from lib import *
from series import accum2
M = mpmath.mpf
blk = sys.argv[1] if len(sys.argv) > 1 else "absweep:2"
Lmode = sys.argv[2] if len(sys.argv) > 2 else "gl"
rows = sorted([r for r in load(cum=True, branch="direct") if r["block"] == blk], key=lambda r: r["a"] + r["b"] + r["x"])
x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
apb = add(a, b); gb, gab, ga = xl("GAMMALN", b), xl("GAMMALN", apb), xl("GAMMALN", a)
lae = np.array([float(mpmath.loggamma(M(v))) for v in a])
alx = mul(a, xl("LN", x))
lnBe = np.array([float(mpmath.log(mpmath.beta(M(ai), M(bi)))) for ai, bi in zip(a, b)])
res = []
for mode, rec in itertools.product(("dbl", "x87"), ("R1", "R2", "R4", "R6")):
    s1, _ = accum2(x, a, b, mode=mode, rec=rec, tf="T0", start="1/a")
    S, _ = accum2(x, a, b, mode=mode, rec=rec, tf="T1")
    for lan, la in (("exact", lae), ("GL", ga), ("lnB exact", None)):
        L0 = add(la, sub(gb, gab)) if la is not None else lnBe
        for k in range(-4, 5):
            L = L0 + k * np.array([math.ulp(v) for v in L0])
            ez = xl("EXP", sub(alx, L))
            res.append((int((mul(ez, s1) == E).sum()), "EXP(z)*s, s=1/a+sum", mode, rec, lan, k))
            res.append((int((mul(div(ez, a), S) == E).sum()), "EXP(z)/a*S", mode, rec, lan, k))
            res.append((int((div(mul(ez, S), a) == E).sum()), "(EXP(z)*S)/a", mode, rec, lan, k))
res.sort(reverse=True)
print(blk, len(E))
seen = set()
for r in res:
    key = r[1:5]
    if key in seen: continue
    seen.add(key)
    print("  %4d %-22s %s %s la=%s k=%+d" % r)
    if len(seen) >= 14: break
