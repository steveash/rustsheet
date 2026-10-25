"""Positive control for regress2: regress SYNTHETIC errors (model P*fl(S), P = EXP(z)/a) on the same columns."""
import sys
from lib import *
M = mpmath.mpf
blk = sys.argv[1]; k = int(sys.argv[2])
rows = sorted([r for r in load(cum=True, branch="direct") if r["block"] == blk], key=lambda r: r["x"])
x = np.array([r["x"] for r in rows]); A = rows[0]["a"]; B = rows[0]["b"]
E = np.array([r["excel"] for r in rows]); mA, mB = M(A), M(B)
I = [exact_cdf(xi, A, B) for xi in x]; Bt = mpmath.beta(mA, mB)
Sx = [i / (M(xi) ** mA / (mA * Bt)) for i, xi in zip(I, x)]
lnB0 = float(mpmath.log(Bt)); L = lnB0 + k * math.ulp(lnB0)
lnx = xl("LN", x); alx = mul(A, lnx); z = sub(alx, L); ez = xl("EXP", z)
eln = np.array([float((M(l) - mpmath.log(M(xi)))) * A / U for l, xi in zip(lnx, x)])
eexp = np.array([float(M(e) / mpmath.exp(M(zz)) - 1) / U for e, zz in zip(ez, z)])
eS = np.array([float(M(float(s)) / s - 1) / U for s in Sx])
Sd = np.array([float(s) for s in Sx])
P = div(ez, A)
syn = {"P*fl(S)": mul(P, Sd), "P*S exact": np.array([float(M(p) * s) for p, s in zip(P, Sx)]), "excel": E}
X = np.column_stack([eln, eexp, eS, np.ones_like(eln)])
for nm, m in syn.items():
    er = rel_units(m, I)
    c, *_ = np.linalg.lstsq(X, er, rcond=None)
    print(f"{nm:12s} coef ln {c[0]:+.2f} exp {c[1]:+.2f} S {c[2]:+.2f} const {c[3]:+.2f}  resid sd {(er - X @ c).std():.3f}")
