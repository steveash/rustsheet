"""T.DIST(t, n, FALSE): compositions RN(K_exact * EXP_xl(E)) with Excel's EXP (model V6, w1) and an exact constant.
Forms (h = (n+1)/2, tt = fl(t t)):
  lnu     E = fl(h * LN(fl(n + tt))) negated,      K = Gamma(h) n^(n/2) / (Gamma(n/2) sqrt(pi))
  lnu_x   E = -h * LN(fl(n + tt)) in extended (EXP of a 64-bit argument), same K
  l1p     E = -fl(h * log1p_xl(fl(tt / n))),       K = Gamma(h) / (Gamma(n/2) sqrt(n pi))
  ln1p    E = -fl(h * LN(fl(1 + fl(tt / n)))),     same K
  ln1p_x  extended product
  ratio   E = -fl(h * LN(fl(fl(n + tt) / n)))
  lny(_x) E = fl(h * LN(y)), y = fl(n / fl(n + tt)) (the tails' y), same K
  m1, m1b w1's beta-density composition: EXP of the extended fl(h LN y) - lnB(n/2, 1/2) - LN(n)/2 (two groupings)
  CR      the exact density
usage: pdf_lab.py [max_cells]"""
import math
import sys
from collections import defaultdict

import mpmath

import tlib
import xlk

bx = tlib.bx
rows = tlib.load(("pdf",))
if len(sys.argv) > 1:
    rows = rows[: int(sys.argv[1])]
for r in rows:
    r["cr"] = tlib.exact_pdf(r["t"], r["n"])
tlib.save_exact()
KC: dict = {}


def K(n, form):
    k = (n, form)
    if k not in KC:
        with mpmath.workdps(40):
            N = mpmath.mpf(n)
            c = mpmath.exp(mpmath.loggamma((N + 1) / 2) - mpmath.loggamma(N / 2)) / mpmath.sqrt(N * mpmath.pi)
            if form.startswith("lnu"):
                c *= mpmath.power(N, (N + 1) / 2)
            KC[k] = c
    return KC[k]


def model(r, form):
    t, n = r["t"], r["n"]
    h = (n + 1) / 2
    tt = t * t
    if form == "CR":
        return r["cr"]
    if form.startswith("lnu"):
        L = bx.LN(n + tt)
    elif form == "l1p":
        q = tt / n
        L = xlk.excel_log1p.log1p_series(q) if q <= 0.375 else bx.LN(1.0 + q)
    elif form.startswith("ln1p"):
        L = bx.LN(1.0 + tt / n)
    elif form == "ratio":
        L = bx.LN((n + tt) / n)
    elif form.startswith("lny"):
        L = -bx.LN(n / (n + tt))
    elif form.startswith("m1"):  # beta-density style: E = fl(h LN y) - lnB(n/2, 1/2) - LN(n)/2, extended sum
        y = n / (n + tt)
        tx = bx.sx("*", h, bx.LN(y))
        lnb = mpmath.mpf(KER.lnb_m1(n / 2, 0.5))
        e = mpmath.mpf(tx) - lnb - mpmath.mpf(bx.LN(n)) / 2 if form == "m1" else \
            mpmath.mpf(tx) - (lnb + mpmath.mpf(bx.sx("*", 0.5, bx.LN(n))))
        return bx.EXPX(bx.em.fl64(e))
    if form.endswith("_x"):
        e = bx.EXPX(bx.em.fl64(-mpmath.mpf(h) * mpmath.mpf(L)))
    else:
        e = bx.EXP(-bx.sx("*", h, L))
    with mpmath.workdps(40):
        return float(K(n, form) * e)


FORMS = ["lnu", "lnu_x", "l1p", "ln1p", "ln1p_x", "ratio", "lny", "lny_x", "m1", "m1b", "CR"]
KER = bx.Kernel(bx.BEST)
out = defaultdict(list)
for r in rows:
    if r["t"] == 0 or r["excel"] == 0:
        continue
    for f in FORMS:
        v = model(r, f)
        out[(f, "all")].append((v, r["excel"]))
        nb = "n=%g" % r["n"] if r["n"] <= 10 else "n<=100" if r["n"] <= 100 else "n>100"
        out[(f, nb)].append((v, r["excel"]))
for k in sorted(out, key=lambda k: (k[1], k[0])):
    print(tlib.line(str(k), out[k]))
