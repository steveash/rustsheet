"""T.DIST pdf: Excel's per-df constant offset (mean error on the exact-input runs of t0969-t-df-ratio, and the
median over all pdf cells of that df) against the error of candidate constants K(n) = Gamma(h)/(Gamma(n/2) sqrt(n pi)).
usage: pdf_const.py"""
import statistics
from collections import defaultdict

import mpmath

import tlib

bx, sx, em = tlib.bx, tlib.bx.sx, tlib.bx.em
U = 2.0**-53
rows = [r for r in tlib.load(("pdf",)) if r["t"] != 0 and r["excel"] != 0]
g = defaultdict(list)
for r in rows:
    ex = tlib.exact_pdf(r["t"], r["n"])
    if abs(r["t"]) < 3:
        g[r["n"]].append((r["excel"] - ex) / ex / U)
ns = sorted(n for n in g if len(g[n]) >= 4 and n <= 1000)
GA = dict(zip([v for n in ns for v in (n / 2, (n + 1) / 2)], em.excel("GAMMA", [v for n in ns for v in (n / 2, (n + 1) / 2)])))
PI = 3.141592653589793
print("n cnt excel_med | candidate errors (cand - exact K, units)")
for n in ns:
    h, a = (n + 1) / 2, n / 2
    with mpmath.workdps(40):
        K = mpmath.exp(mpmath.loggamma(h) - mpmath.loggamma(a)) / mpmath.sqrt(n * mpmath.pi)
        c = {
            "EXP(GL-GL)/SQRT(n*PI)": sx("/", bx.EXP(sx("-", bx.GAMMALN(h), bx.GAMMALN(a))), (n * PI) ** 0.5),
            "EXP(ext GL-GL-LN(nPI)/2)": bx.EXPX(em.fl64(mpmath.mpf(bx.GAMMALN(h)) - mpmath.mpf(bx.GAMMALN(a))
                                                     - mpmath.mpf(bx.LN(n * PI)) / 2)),
            "GAMMA/GAMMA/SQRT": sx("/", sx("/", GA[h], GA[a]), (n * PI) ** 0.5),
            "GAMMA/(GAMMA*SQRT)": sx("/", GA[h], sx("*", GA[a], (n * PI) ** 0.5)),
            "1/(sqrt n B), B=EXP(lnb_m1)": sx("/", 1.0, sx("*", n ** 0.5, bx.EXP(bx.Kernel(bx.BEST).lnb_m1(a, 0.5)))),
        }
        errs = {k: float((mpmath.mpf(v) - K) / K) / U for k, v in c.items()}
    print(f"{n:g} {len(g[n])} {statistics.median(g[n]):+.2f} | " + " ".join(f"{k} {v:+.2f}" for k, v in errs.items()))
