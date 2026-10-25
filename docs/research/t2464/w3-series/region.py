"""Region map: power-series candidates for Excel's direct-branch cdf on every direct cdf cell (beta_exact + w2 probes),
scored per region and per tune/hold-out half. Closed forms (a = 1 or b = 1) are excluded.

Candidates (P = prefactor, S = series; x87 stored ops unless noted, Excel's LN/EXP/GAMMALN/GAMMA):
  exactS:<P>        P * S_exact (S exact, one rounding of the product): isolates the prefactor
  bpser:<P>:<mode>:<stop>   TOMS 708 bpser loop (c *= (0.5 - b/n + 0.5)*x; w = c/(a+n)), ans = P * (a*sum + 1)
  cephes:<P'>:<mode>        Cephes pseries s = 1/a + sum t_n/(a+n), ans = s * x^a/B
P forms: z = fl(fl(a*LN x) - lnB); EXP(z)/a. lnB = GLa+(GLb-GLab), (GLa+GLb)-GLab, GLb+(GLa-GLab), exact.
P' forms (Cephes): EXP(z); EXP(fl(a LN x))*GAMMA(a+b)/(GAMMA(a)*GAMMA(b)).
Run: python region.py  (writes region.out)
"""

import collections
import sys

from lib import *
from series import bpser_sum, cephes_pseries_sum

M = mpmath.mpf
rows = [r for r in load(cum=True, branch="direct") if r["a"] != 1.0 and r["b"] != 1.0]
tune, hold = split_half(rows)
tune_ids = {id(r) for r in tune}


def region(r):
    a, b, x = r["a"], r["b"], r["x"]
    if min(a, b) < 1:
        return "a<1" if a < 1 else "b<1"
    if b * x <= 0.7:
        return "a,b>1 bx<=0.7"
    return "a,b>1 bx>0.7"


x = np.array([r["x"] for r in rows])
a = np.array([r["a"] for r in rows])
b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
I = [exact_cdf(xi, ai, bi) for xi, ai, bi in zip(x, a, b)]
save_exact()
lnBex = np.array([float(mpmath.log(mpmath.beta(M(ai), M(bi)))) for ai, bi in zip(a, b)])
Sx = [
    i / (M(xi) ** M(ai) / (M(ai) * mpmath.exp(mpmath.log(mpmath.beta(M(ai), M(bi))))))
    for i, xi, ai, bi in zip(I, x, a, b)
]
apb = add(a, b)
ga, gb, gab = xl("GAMMALN", a), xl("GAMMALN", b), xl("GAMMALN", apb)
lnx = xl("LN", x)
alx = mul(a, lnx)
lnBs = {
    "GLa+(GLb-GLab)": add(ga, sub(gb, gab)),
    "(GLa+GLb)-GLab": sub(add(ga, gb), gab),
    "GLb+(GLa-GLab)": add(gb, sub(ga, gab)),
    "exact": lnBex,
}
models = {}
models["correctly rounded"] = np.array([float(v) for v in I])
P = {}
for nm, L in lnBs.items():
    z = sub(alx, L)
    ez = xl("EXP", z)
    P[nm] = div(ez, a)
    models[f"exactS:EXP(z)/a [{nm}]"] = np.array([float(M(p) * s) for p, s in zip(P[nm], Sx)])
    P["ez " + nm] = ez
bp = {}
for mode in ("dbl", "x87", "ext"):
    for stop in ("eps_a", "unchanged"):
        S, sm = bpser_sum(x, a, b, mode=mode, tol_kind=stop, eps=1e-15, nmax=20000)
        bp[(mode, stop)] = (S, sm)
        for nm in lnBs:
            models[f"bpser:{nm}:{mode}:{stop}"] = mul(P[nm], S)
    sC = cephes_pseries_sum(x, a, b, mode=mode, nmax=20000)
    for nm in lnBs:
        models[f"cephes:EXP(z) [{nm}]:{mode}"] = mul(P["ez " + nm], sC)
    gr = div(xl("GAMMA", apb), mul(xl("GAMMA", a), xl("GAMMA", b)))
    xa = xl("EXP", alx)
    models[f"cephes:x^a*Gratio:{mode}"] = mul(mul(sC, gr), xa)
regs = sorted({region(r) for r in rows})
out = []
hdr = f"{'model':52s} " + " ".join(f"{g:>22s}" for g in regs) + f" {'all tune':>16s} {'all hold':>16s}"
out.append(hdr)
for nm, m in models.items():
    cols = []
    for g in regs:
        idx = [i for i, r in enumerate(rows) if region(r) == g]
        sc = score(m[idx], E[idx])
        cols.append(f"{sc[0]:5d}/{sc[1]:5d}/{sc[2]:5d}")
    it = [i for i, r in enumerate(rows) if id(r) in tune_ids]
    ih = [i for i, r in enumerate(rows) if id(r) not in tune_ids]
    st, sh = score(m[it], E[it]), score(m[ih], E[ih])
    out.append(
        f"{nm:52s} "
        + " ".join(f"{c:>22s}" for c in cols)
        + f" {st[0]:5d}/{st[2]:5d}     {sh[0]:5d}/{sh[2]:5d}"
    )
ns = {g: sum(region(r) == g for r in rows) for g in regs}
out.append(
    "n per region: "
    + str(ns)
    + f"; tune {len(tune)} hold {len(hold)}; cells: exact/within4ulp/within15digits"
)
txt = "\n".join(out)
print(txt)
open(HERE / "region.out", "w").write(txt + "\n")
