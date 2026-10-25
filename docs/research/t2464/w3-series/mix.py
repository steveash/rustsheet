"""Mixture test: model A = fl(fl(EXP(z))/a * S_exact), model B = fl(exp(z)/a*S_exact) (unrounded exp), cross-tab."""
import sys, collections
from lib import *
M = mpmath.mpf
blk = sys.argv[1]; Ld = int(sys.argv[2])
rows = [r for r in load(cum=True, branch="direct") if r["block"] == blk]
x = np.array([r["x"] for r in rows]); A = rows[0]["a"]; B = rows[0]["b"]
E = np.array([r["excel"] for r in rows]); mA, mB = M(A), M(B)
I = [exact_cdf(xi, A, B) for xi in x]; Bt = mpmath.beta(mA, mB)
Sx = [i / (M(xi) ** mA / (mA * Bt)) for i, xi in zip(I, x)]
lnB0 = float(mpmath.log(Bt))
for Ld in range(Ld - 3, Ld + 4):
    L = lnB0 + Ld * math.ulp(lnB0)
    z = sub(mul(A, xl("LN", x)), L)
    ez = xl("EXP", z)
    mAo = np.array([float(M(e) / mA * s) for e, s in zip(ez, Sx)])
    mBo = np.array([float(mpmath.exp(M(zz)) / mA * s) for zz, s in zip(z, Sx)])
    a_ok, b_ok = mAo == E, mBo == E
    print(Ld, "A", a_ok.sum(), "B", b_ok.sum(), "both", (a_ok & b_ok).sum(), "either", (a_ok | b_ok).sum())
