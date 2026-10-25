"""With the accumulated Cephes-style series s (x87, R4, start 1/a) and I = EXP(fl(fl(a LN x) - L)) * s, which lnB
form L reproduces each block with NO free offset? Also prints the best free offset for reference."""
import sys, itertools
from lib import *
from series import accum2
M = mpmath.mpf
blocks = sys.argv[1:] or ["xruns-1:3", "absweep:2", "absweep:6", "xruns-2:6", "xruns-1:6"]
R = load(cum=True, branch="direct")
for blk in blocks:
    rows = [r for r in R if r["block"] == blk]
    x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
    E = np.array([r["excel"] for r in rows])
    s1, _ = accum2(x, a, b, mode="x87", rec="R4", tf="T0", start="1/a")
    apb = add(a, b); ga, gb, gab = xl("GAMMALN", a), xl("GAMMALN", b), xl("GAMMALN", apb)
    Ga, Gb, Gab = xl("GAMMA", a), xl("GAMMA", b), xl("GAMMA", apb)
    lae = np.array([float(mpmath.loggamma(M(v))) for v in a]); lbe = np.array([float(mpmath.loggamma(M(v))) for v in b])
    labe = np.array([float(mpmath.loggamma(M(v))) for v in apb])
    lnBe = np.array([float(mpmath.log(mpmath.beta(M(ai), M(bi)))) for ai, bi in zip(a, b)])
    p, q = np.minimum(a, b), np.maximum(a, b)
    Gp, Gq = xl("GAMMA", p), xl("GAMMA", q)
    forms = {
        "lnB exact": lnBe,
        "GLa+(GLb-GLab)": add(ga, sub(gb, gab)), "(GLa+GLb)-GLab": sub(add(ga, gb), gab), "GLb+(GLa-GLab)": add(gb, sub(ga, gab)),
        "la_ex+(GLb-GLab)": add(lae, sub(gb, gab)), "(la_ex+GLb)-GLab": sub(add(lae, gb), gab),
        "ex+ex-ex stored": sub(add(lae, lbe), labe), "la+(lb-lab) ex stored": add(lae, sub(lbe, labe)),
        "LN(Ga*Gb/Gab)": xl("LN", div(mul(Ga, Gb), Gab)), "LN(Ga*(Gb/Gab))": xl("LN", mul(Ga, div(Gb, Gab))),
        "LN(Gp*(Gq/Gpq))": xl("LN", mul(Gp, div(Gq, Gab))), "LN(Gq*(Gp/Gpq))": xl("LN", mul(Gq, div(Gp, Gab))),
        "LN(Ga/Gab*Gb)": xl("LN", mul(div(Ga, Gab), Gb)),
    }
    alx = mul(a, xl("LN", x))
    print(f"== {blk} n={len(E)} a={a[0]:g} b={b[0]:g}")
    out = []
    for nm, L in forms.items():
        c0 = int((mul(xl("EXP", sub(alx, L)), s1) == E).sum())
        bk = max((int((mul(xl("EXP", sub(alx, L + k * np.array([math.ulp(v) for v in L]))), s1) == E).sum()), k) for k in range(-6, 7))
        out.append((c0, nm, bk))
    for c0, nm, bk in sorted(out, reverse=True):
        print(f"   {c0:5d}  {nm:24s} (best free offset {bk[0]} at k={bk[1]:+d})")
