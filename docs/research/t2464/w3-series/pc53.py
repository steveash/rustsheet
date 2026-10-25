"""Hypothesis: the beta code's exp runs with the x87 precision control at 53 bits (PC53): every step of the EXP V6
recipe rounds to a double. Compare P = exp_pc53(z)/a vs Excel EXP(z)/a in P*S_exact on direct x-runs and sweeps."""
import sys
from lib import *
M = mpmath.mpf
L2E53 = fl53(1 / mpmath.log(2))
L2E64 = fl64(1 / mpmath.log(2))
def exp_pc(z, bits):
    rnd = fl53 if bits == 53 else (lambda v: fl64(v))
    t = M(rnd(M(z) * (M(L2E53) if bits == 53 else L2E64)))
    n = int(mpmath.nint(t)); f = t - n
    u = M(rnd(M(2) ** abs(f) - 1)); w = M(rnd(1 + u))
    if f < 0: w = M(rnd(1 / w))
    return w * M(2) ** n
blk = sys.argv[1]; Ld = int(sys.argv[2]) if len(sys.argv) > 2 else None
rows = [r for r in load(cum=True, branch="direct") if r["block"] == blk]
x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
I = [exact_cdf(xi, ai, bi) for xi, ai, bi in zip(x, a, b)]
Sx = [i / (M(xi) ** M(ai) / (M(ai) * mpmath.beta(M(ai), M(bi)))) for i, xi, ai, bi in zip(I, x, a, b)]
ga, gb, gab = xl("GAMMALN", a), xl("GAMMALN", b), xl("GAMMALN", add(a, b))
lnB0 = np.array([float(mpmath.log(mpmath.beta(M(ai), M(bi)))) for ai, bi in zip(a, b)])
alx = mul(a, xl("LN", x))
best = {}
for k in range(-20, 21):
    L = lnB0 + k * np.array([math.ulp(v) for v in lnB0])
    z = sub(alx, L)
    for nm, P in (("Excel EXP", xl("EXP", z)), ("PC53 exp", np.array([float(exp_pc(v, 53)) for v in z])),
                  ("ext exp (no store)", None)):
        if P is None:
            m = np.array([fl53(fl64(fl64(exp_pc(v, 64) / M(ai)) * s)) for v, ai, s in zip(z, a, Sx)])
        else:
            m = np.array([float(M(p) / M(ai) * s) for p, ai, s in zip(P, a, Sx)])
        c = int((m == E).sum())
        if c > best.get(nm, (0,))[0]: best[nm] = (c, k)
print(blk, len(E), best)
