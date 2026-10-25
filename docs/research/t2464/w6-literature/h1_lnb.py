"""w6-literature H1: which library's ln B(a, b) does Excel's density use for min(a,b) < 1?

Literature candidates (structure only, coefficients from the sources):
  TOMS   : TOMS 708 betaln (Didonato & Morris 1992; R toms708.c): b0 < 8 -> GL form, else gamln(a0) + algdiv
           (w1's M1, with Excel GAMMALN/LN pieces).
  GL     : fl(GL(lo) + fl(GL(hi) - GL(a+b))) at every max (no algdiv).
  R      : R nmath lbeta.c: q < 10 -> log(gamma(p) * (gamma(q)/gamma(p+q))); q >= 10 ->
           lgammafn(p) + corr + p - p*log(p+q) + (q-0.5)*log1p(-p/(p+q)), corr = lgammacor(q) - lgammacor(p+q)
           (lgammafn(p) = log|gamma(p)| for p <= 10). Excel GAMMA and LN as gamma/log.
  NSWC   : Apache Commons Math Beta.logBeta (NSWC DBETLN): a < 1: b >= 10 -> logGamma(a) + algdiv(a, b),
           else log(gamma(a) * gamma(b) / gamma(a+b)).
  TOMS10 : TOMS betaln with the algdiv switch at 10 instead of 8 (R/NSWC cutoff, TOMS formulas).
  exact  : the correctly rounded ln B (control).
Everything else is w1's M1 (logs, stored products, extended exponent, Excel EXP V6). Scored per max-shape bin
and per hold-out half (even/odd row index).
"""

import collections
import math
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "w1copy"))
import mpmath  # noqa: E402

from forms import ingredients  # noqa: E402
from lib import D, R64, exp_v6, fmt, load_pdf, load_probe, mpf, score, xl  # noqa: E402
from toms_betaln import GL, LN, algdiv, betaln  # noqa: E402

ALGMCS = [
    +0.1666389480451863247205729650822e0,
    -0.1384948176067563840732986059135e-4,
    +0.9810825646924729426157171547487e-8,
    -0.1809129475572494194263306266719e-10,
    +0.6221098041892605227126015543416e-13,
]


def cheb(x, a):
    twox = x * 2
    b2 = b1 = b0 = 0.0
    for c in reversed(a):
        b2 = b1
        b1 = b0
        b0 = twox * b1 - b2 + c
    return (b0 - b2) * 0.5


def lgammacor(x):
    t = 10 / x
    return cheb(t * t * 2 - 1, ALGMCS) / x


def G(v):
    return xl("GAMMA", [v])[0]


def r_lbeta(a, b, lg="lnG"):
    p, q = min(a, b), max(a, b)
    if q >= 10:
        corr = lgammacor(q) - lgammacor(p + q)
        lp = LN(abs(G(p))) if lg == "lnG" else GL(p)
        return lp + corr + p - p * LN(p + q) + (q - 0.5) * math.log1p(-p / (p + q))
    return LN(G(p) * (G(q) / G(p + q)))


def nswc(a, b):
    p, q = min(a, b), max(a, b)
    if q >= 10:
        return GL(p) + algdiv(p, q, LN)
    return LN(G(p) * G(q) / G(p + q))


def toms10(a, b):
    p, q = min(a, b), max(a, b)
    if q < 10:
        return float(D(mpf(GL(p)) + D(mpf(GL(q)) - mpf(GL(a + b)))))
    return betaln(a, b)


def glform(a, b):
    lo, hi = min(a, b), max(a, b)
    return float(D(mpf(GL(lo)) + D(mpf(GL(hi)) - mpf(GL(a + b)))))


def toms(a, b):
    lo, hi = min(a, b), max(a, b)
    if hi <= 8.0:  # w1: 8 itself takes the GL form
        return glform(a, b)
    return betaln(a, b)


VARIANTS = {
    "TOMS(w1)": toms,
    "GL": glform,
    "R lbeta lnG": lambda a, b: r_lbeta(a, b, "lnG"),
    "R lbeta GL": lambda a, b: r_lbeta(a, b, "GL"),
    "NSWC/Apache": nswc,
    "TOMS@10": toms10,
    "exact": lambda a, b: float(mpmath.log(mpmath.beta(mpf(a), mpf(b)))),
}


def m1(rows, ing, lnb):
    out, cache = [], {}
    for r, g in zip(rows, ing):
        a, b, x = r["a"], r["b"], r["x"]
        if (a, b) not in cache:
            cache[(a, b)] = mpf(lnb(a, b))
        L = cache[(a, b)]
        ly = mpf(D(g["l1p"])) if x <= 0.375 else g["ly_yd"]
        tx = mpf(D(g["am1"] * g["LNd"]))
        ty = mpf(D(g["bm1"] * ly))
        E = mpf(D(tx - L)) if b == 1.0 else R64(tx + R64(ty - L))
        out.append(exp_v6(E))
    return out


def hibin(a, b):
    hi = max(a, b)
    if hi < 8:
        return "1) max<8"
    if hi == 8:
        return "2) max=8"
    if hi < 10:
        return "3) 8<max<10"
    if hi == 10:
        return "4) max=10"
    return "5) max>10"


def run(rows, label):
    rows = [r for r in rows if min(r["a"], r["b"]) < 1 and r["a"] != 1 and r["b"] != 1]
    ing = ingredients(rows)
    groups = collections.defaultdict(list)
    for i, r in enumerate(rows):
        groups[hibin(r["a"], r["b"])].append(i)
    print(f"== {label}: n={len(rows)} (min<1, no unit shape)")
    for name, f in VARIANTS.items():
        m = m1(rows, ing, f)
        parts = []
        for gk in sorted(groups):
            idx = groups[gk]
            ev = [i for i in idx if i % 2 == 0]
            od = [i for i in idx if i % 2 == 1]
            s = score(m, rows, idx)
            parts.append(
                f"{gk} {s[1]}/{s[0]} (e{score(m, rows, ev)[1]} o{score(m, rows, od)[1]}) w4 {s[2]} w15 {s[3]}"
            )
        print(f"  {name:14s} all {fmt(score(m, rows))}")
        for p in parts:
            print(f"      {p}")


if __name__ == "__main__":
    run(load_pdf(), "beta_exact pdf")
    pr = load_probe("probe1")
    run(pr, "w1 probe1 pdf")
