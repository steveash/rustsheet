"""Positive/negative control for the series discrimination: synthetic 'Excel' values from three known series
(fwd from 1/a x87, fwd from 1 x87, exact S), same prefactor P = EXP(z53), on the real a-sweep and (1.5,0.5) x-run
points; score each candidate against each synthetic truth. A usable discriminator gives a dominant diagonal."""
from lib import *
from model import exponent
from series import accum2
M = mpmath.mpf
R = load(cum=True, branch="direct")
for blk in ("absweep:2", "xruns-1:6", "xruns-1:3"):
    rows = [r for r in R if r["block"] == blk]
    x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
    E = np.array([r["excel"] for r in rows])
    z = exponent(x, a, b, big=8.0)
    P = xl("EXP", np.array([fl53(v) for v in z]))
    I = [exact_cdf(xi, ai, bi) for xi, ai, bi in zip(x, a, b)]
    Sx = [i / (M(xi) ** M(ai) / M(ai) / mpmath.beta(M(ai), M(bi))) for i, xi, ai, bi in zip(I, x, a, b)]
    s1a = accum2(x, a, b, mode="x87", rec="R4", tf="T0", start="1/a")[0]
    s1 = accum2(x, a, b, mode="x87", rec="R4", tf="T1", start=1.0)[0]
    cands = {"fwd 1/a": mul(P, s1a), "fwd 1": mul(div(P, a), s1),
             "exact S": np.array([float(M(p) / M(ai) * s) for p, ai, s in zip(P, a, Sx)])}
    print(f"== {blk} (rows: synthetic truth; cols: candidate)   real Excel row last")
    for tn, tv in list(cands.items()) + [("REAL EXCEL", E)]:
        print(f"   {tn:11s} " + "  ".join(f"{cn}: {int((cv == tv).sum()):4d}" for cn, cv in cands.items()) + f"   of {len(E)}")
