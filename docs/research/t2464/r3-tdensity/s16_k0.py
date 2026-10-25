"""T.DIST(0, n, FALSE) = K_n: enumerate x87 compositions of Excel pieces (GAMMALN, GAMMA, EXP, LN, SQRT, PI) for
K = Gamma(h) / (Gamma(n/2) sqrt(n pi)) and count matches with Excel over the 8 dfs of m9-t-dist-pdf (s15_t0.XL).
usage: s16_k0.py"""
import itertools
import math

import mpmath

import tdlib as TD
from s15_t0 import XL

L, st = TD.LDB, TD.st
PI = math.pi
ns = [n for n in XL if n >= 3]
hs = [(n + 1) / 2 for n in ns] + [n / 2 for n in ns]
GLv = dict(zip(hs, TD.xl_fn("GAMMALN", hs)))
GAv = dict(zip(hs, TD.xl_fn("GAMMA", [h if h < 171 else 1.0 for h in hs])))


def X(op, a, b):
    a, b = L(a), L(b)
    return st({"+": a + b, "-": a - b, "*": a * b, "/": a / b}[op])


def EXP(v):
    return TD.exp_v6(L(v))


def SQ(v):
    return st(np_sqrt(L(v)))


np_sqrt = __import__("numpy").sqrt


def pieces(n):
    h, k = (n + 1) / 2, n / 2
    r = {}
    r["E(GLh-GLk)"] = EXP(X("-", GLv[h], GLv[k]))
    if h < 171:
        r["Gh/Gk"] = X("/", GAv[h], GAv[k])
    r["E(GLh-GLk-LN(n)/2)"] = EXP(X("-", X("-", GLv[h], GLv[k]), X("/", TD.LN(n), 2.0)))
    r["E(GLh-GLk-.5LN(nPI))"] = EXP(X("-", X("-", GLv[h], GLv[k]), X("*", 0.5, TD.LN(X("*", n, PI)))))
    r["E(GLh-(GLk+.5LN(nPI)))"] = EXP(X("-", GLv[h], X("+", GLv[k], X("*", 0.5, TD.LN(X("*", n, PI))))))
    lnb = X("+", TD.GL(0.5), X("-", GLv[k], GLv[h])) if k <= 8 else None
    if lnb is not None:
        r["E(-lnB)"] = EXP(-lnb)
        r["E(-lnB-LN(n)/2)"] = EXP(X("-", -lnb, X("/", TD.LN(n), 2.0)))
    return r


dens = {"SQ(n*PI)": lambda n: SQ(X("*", n, PI)), "SQ(n)*SQ(PI)": lambda n: X("*", SQ(n), SQ(PI)),
        "SQ(PI*n)": lambda n: SQ(X("*", PI, n)), "1": lambda n: 1.0, "SQ(n)": lambda n: SQ(n)}
res = {}
for n in ns:
    P = pieces(n)
    for (pn, pv), (dn, df) in itertools.product(P.items(), dens.items()):
        if "LN" in pn and dn != "1":
            continue
        if "lnB" in pn and dn not in ("SQ(n)", "1"):
            continue
        for mode in ("/", "*inv"):
            d = df(n)
            v = X("/", pv, d) if mode == "/" else X("*", pv, X("/", 1.0, d))
            key = f"{pn} {mode} {dn}"
            res.setdefault(key, []).append((n, round((v - XL[n]) / math.ulp(XL[n]))))
for k, v in sorted(res.items(), key=lambda kv: -sum(e == 0 for _, e in kv[1])):
    print(f"{sum(e == 0 for _, e in v)}/{len(v)} {k}: " + " ".join(f"{n:g}:{e:+d}" for n, e in v))
print("CR:", " ".join(f"{n:g}:{round((float(mpmath.exp(TD.exact_logpdf(0.0, n))) - XL[n]) / math.ulp(XL[n])):+d}" for n in ns))
