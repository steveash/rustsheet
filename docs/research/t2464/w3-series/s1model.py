"""Models where the first partial sum q1 = 1 + a*w1 is stored and the rest added: S = fl(q1 + a*R), R = sum_{n>=2}
w_n (exact or double), times P = EXP(z)/a (stored or extended)."""
import sys
from lib import *
M = mpmath.mpf
blk = sys.argv[1]; k = int(sys.argv[2])
rows = sorted([r for r in load(cum=True, branch="direct") if r["block"] == blk], key=lambda r: r["x"])
x = np.array([r["x"] for r in rows]); A = rows[0]["a"]; B = rows[0]["b"]
E = np.array([r["excel"] for r in rows]); mA, mB = M(A), M(B)
I = [exact_cdf(xi, A, B) for xi in x]; Bt = mpmath.beta(mA, mB)
err = rel_units(E, I)
X = [M(v) for v in x]
Sx = [i / (v ** mA / (mA * Bt)) for i, v in zip(I, X)]
lnB0 = float(mpmath.log(Bt))
w1 = [(1 - mB) * v / (mA + 1) for v in X]
q1 = np.array([float(1 + mA * w) for w in w1])
R = [(s - 1) / mA - w for s, w in zip(Sx, w1)]
Rd = np.array([float(v) for v in R])
for kk in range(k - 2, k + 3):
    L = lnB0 + kk * math.ulp(lnB0)
    z = sub(mul(A, xl("LN", x)), L); ez = xl("EXP", z); P = div(ez, A)
    S1 = add(q1, mul(A, Rd))
    models = {"P*fl(q1 + a*R)": mul(P, S1), "P*(q1 + a*R) ext": s(np.asarray(P, dtype=LD) * (np.asarray(q1, dtype=LD) + LD(A) * np.asarray(Rd, dtype=LD))),
              "P*q1 + P*a*R": add(mul(P, q1), mul(P, mul(A, Rd))),
              "ext P*(q1+aR) exactR": np.array([fl53(M(p) * (M(q) + mA * r)) for p, q, r in zip(P, q1, R)])}
    for nm, m in models.items():
        me = rel_units(m, I)
        print(f"  k{kk:+d} {nm:24s} exact {int((m == E).sum()):4d} corr {np.corrcoef(me, err)[0,1]:+.3f} resid {(err - me).std():.3f}")
