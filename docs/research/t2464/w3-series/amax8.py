"""a < 1 < 8 < b cells (beta_exact): series variants x prefactor forms with the exact lnB (no fitting)."""
import sys
from lib import *
from series import bpser_sum, cephes_pseries_sum
M = mpmath.mpf
rows = [r for r in load(cum=True, branch="direct") if r["a"] < 1 and r["b"] > 8]
x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
I = [exact_cdf(xi, ai, bi) for xi, ai, bi in zip(x, a, b)]
lnBe = [mpmath.log(mpmath.beta(M(ai), M(bi))) for ai, bi in zip(a, b)]
Sx = [i / (M(xi) ** M(ai) / M(ai) / mpmath.exp(l)) for i, xi, ai, l in zip(I, x, a, lnBe)]
Lex = np.array([float(l) for l in lnBe])
alx = mul(a, xl("LN", x))
ez = xl("EXP", sub(alx, Lex)); P = div(ez, a)
print("n", len(rows), "groups", sorted({(r['a'], r['b']) for r in rows}))
print("CR", int((np.array([float(v) for v in I]) == E).sum()), " P*S_exact", int((np.array([float(M(p) * s) for p, s in zip(P, Sx)]) == E).sum()))
for mode in ("dbl", "x87", "ext"):
    for stop in ("unchanged", "eps_a", "rel"):
        for var in ("toms", "simple", "nb"):
            S, sm = bpser_sum(x, a, b, mode=mode, tol_kind=stop, eps=1e-15 if stop == "eps_a" else 2.0 ** -53, variant=var, nmax=20000)
            m1 = mul(P, S); m2 = mul(ez, add(div(1.0, a), sm))
            print(f"  bpser {var:6s} {mode} {stop:9s}: P*(1+a*sum) {int((m1 == E).sum()):2d}   EXP(z)*(1/a+sum) {int((m2 == E).sum()):2d}")
    sC = cephes_pseries_sum(x, a, b, mode=mode, nmax=20000)
    print(f"  cephes {mode}: EXP(z)*s {int((mul(ez, sC) == E).sum())}")
for i, r in enumerate(rows):
    S, _ = bpser_sum(x[i:i+1], a[i:i+1], b[i:i+1], mode="ext", tol_kind="unchanged", nmax=20000)
    m = mul(P[i:i+1], S)[0]
    print(f"   a={r['a']} b={r['b']} x={r['x']:.4g}  excel-model {ulps(m, r['excel']) if False else (r['excel']-m)/math.ulp(r['excel']):+.0f} ulp  excel-CR {(r['excel']-float(I[i]))/math.ulp(r['excel']):+.0f}")
