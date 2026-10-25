"""bgrat at b = 1/2: per-df offset of Excel against the model, against candidate forms of the constant factor
C(n) = EXP(-(algdiv(1/2, a) + LN(nu)/2)), a = n/2, nu = a - 1/4. Prints per-n: Excel median offset (model - excel,
units 2^-53) and each candidate's predicted shift (cand - model, units) so that a right candidate gives
med + shift ~ 0 ... wait: shift is how much the model moves; Excel-matching means med + shift = 0."""
import statistics
import sys
from collections import defaultdict

import mpmath

import tlib
import tscore
import xlk

bx = tlib.bx
SPEC = sys.argv[1] if len(sys.argv) > 1 else "args=yz,alnrel=xl,erfc=xl,bg_lnx=cut0.01"
rows = tscore.rows_L()
vals, brs = tscore.evaluate(rows, SPEC)
_, cfg = tscore.parse(SPEC)
K = bx._KERNELS[cfg]
g = defaultdict(list)
for i, r in enumerate(rows):
    if brs[i] == "bgrat/sw":
        g[r["n"]].append(i)


def cands(a):
    b = 0.5
    nu = a - 0.25
    ad_t = K.algdiv(b, a, True)
    lnu = bx.LN(nu)
    gl = bx.sx("-", bx.GAMMALN(a), bx.GAMMALN(a + b))
    with mpmath.workdps(40):
        ad_x = float(mpmath.loggamma(a) - mpmath.loggamma(a + b))
        exact = mpmath.exp(-(mpmath.loggamma(a) - mpmath.loggamma(a + b)) - mpmath.log(mpmath.mpf(nu)) / 2)
    out = {
        "toms": bx.EXP(-(ad_t + b * lnu)),
        "toms_xsum": bx.EXPX(-bx.em.fl64(mpmath.mpf(ad_t) + mpmath.mpf(b * lnu))),
        "gl": bx.EXP(-(gl + b * lnu)),
        "gl_x": bx.EXPX(-bx.em.fl64(mpmath.mpf(gl) + mpmath.mpf(b * lnu))),
        "exact_ad": bx.EXP(-(ad_x + b * lnu)),
        "exp2": bx.EXP(-ad_t) * bx.EXP(-(b * lnu)),
        "CR": float(exact),
    }
    return out


print("n cnt med(model-excel) | shift per candidate (cand - toms), units; resid = med + shift")
tot = defaultdict(float)
for n in sorted(g):
    idx = g[n]
    e = [(vals[i] - rows[i]["excel"]) / abs(rows[i]["excel"]) / 2**-53 for i in idx]
    med = statistics.median(e)
    c = cands(n / 2)
    base = c["toms"]
    parts = []
    for k, v in c.items():
        sh = (v - base) / base / 2**-53
        parts.append(f"{k} {med + sh:+.1f}")
        tot[k] += abs(med + sh) * len(idx)
    print(f"{n:g} {len(idx)} {med:+.2f} | " + " ".join(parts))
print("weighted |resid| sum:", {k: round(v) for k, v in tot.items()})
