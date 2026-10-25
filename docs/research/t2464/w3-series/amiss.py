"""a-sweep misses of the best model (fwd from 1/a x87, z53 k=-1/4): position vs a, E - model ulps, and regressions
of the residual (relative units) on candidate a-dependent rounding columns."""
import collections
from lib import *
from model import exponent
from series import accum2
M = mpmath.mpf
rows = sorted([r for r in load(cum=True, branch="direct") if r["block"] == "absweep:2"], key=lambda r: r["a"])
x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
z = exponent(x, a, b, big=8.0); zs = [M(math.ulp(float(v))) for v in z]
zz = np.array([fl53(v - M(1) / 4 * u) for v, u in zip(z, zs)])
P = xl("EXP", zz)
s1, _ = accum2(x, a, b, mode="x87", rec="R4", tf="T0", start="1/a")
m = mul(P, s1)
d = np.round((E - m) / np.array([math.ulp(v) for v in m])).astype(int)
print("E-model:", sorted(collections.Counter(d).items()))
print("miss index runs:", np.nonzero(d)[0].tolist())
I = [exact_cdf(xi, ai, bi) for xi, ai, bi in zip(x, a, b)]
r = np.array([float(M(e) / M(v) - 1) / U for e, v in zip(E, m)])
apb = add(a, b)
cols = {"e(a+b)psi": np.array([float(mpmath.digamma(M(s_)) * (M(s_) - (M(ai) + M(bi)))) / U for s_, ai, bi in zip(apb, a, b)]),
        "e GL(a+b)": np.array([float(M(g) - mpmath.loggamma(M(v))) / U for g, v in zip(xl("GAMMALN", apb), apb)]),
        "e fl(1/a)": np.array([float(M(v) * M(ai) - 1) / U for v, ai in zip(div(1.0, a), a)]),
        "e(alx)": np.array([float(M(p) - M(ai) * M(l)) / U for p, ai, l in zip(mul(a, xl("LN", x)), a, xl("LN", x))]),
        "e(z53)": np.array([float(M(v) - w) / U for v, w in zip(zz, z)]),
        "e EXP": np.array([float(M(e) / mpmath.exp(M(v)) - 1) / U for e, v in zip(P, zz)])}
for k, v in cols.items():
    print(f"  {k:12s} sd {v.std():.2f}  corr {np.corrcoef(v, r)[0,1]:+.3f}  slope {np.polyfit(v, r, 1)[0]:+.2f}")
eg = cols["e GL(a+b)"]
for i in list(range(286, 312)) + list(range(402, 416)):
    print(f"   i={i} a={a[i]!r} a+b={apb[i]!r} r={r[i]:+6.2f} eGL(a+b)={eg[i]:+6.2f} e(z53)={cols['e(z53)'][i]:+.2f}")
