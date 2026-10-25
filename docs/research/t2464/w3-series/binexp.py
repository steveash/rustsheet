"""Excel err - a*e(LN) binned by EXP(z)'s rounding error (relative units) and by the fractional position of
exp(z) in ulps: is the EXP(z) rounding inherited linearly?"""
import sys
from lib import *
M = mpmath.mpf
blk = sys.argv[1]; k = int(sys.argv[2])
rows = sorted([r for r in load(cum=True, branch="direct") if r["block"] == blk], key=lambda r: r["x"])
x = np.array([r["x"] for r in rows]); A = rows[0]["a"]; B = rows[0]["b"]
E = np.array([r["excel"] for r in rows]); mA, mB = M(A), M(B)
I = [exact_cdf(xi, A, B) for xi in x]; Bt = mpmath.beta(mA, mB)
err = rel_units(E, I)
lnB0 = float(mpmath.log(Bt)); L = lnB0 + k * math.ulp(lnB0)
lnx = xl("LN", x); alx = mul(A, lnx); z = sub(alx, L); ez = xl("EXP", z)
eln = np.array([float((M(l) - mpmath.log(M(v)))) * mA / U for l, v in zip(lnx, x)])
ph = np.array([float(mpmath.frac(mpmath.exp(M(zz)) / M(math.ulp(e)))) for zz, e in zip(z, ez)])  # frac position of exact exp
ph = np.where(ph > 0.5, ph - 1, ph)  # in (-0.5, 0.5]: exp_exact = fl + (ph)*ulp  -> rounding error = -ph ulp
r = err - eln
print(blk, "bins of exp(z) fractional position (rounding error of EXP in ulps of exp = -pos)")
for lo in np.arange(-0.5, 0.5, 0.1):
    s = (ph >= lo) & (ph < lo + 0.1)
    print(f"  pos [{lo:+.1f},{lo+0.1:+.1f}) n {s.sum():4d}  mean r {r[s].mean():+.3f}  sd {r[s].std():.3f}")
