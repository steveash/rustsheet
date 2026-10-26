"""T.DIST(t, n, FALSE) as TOMS brcomp(n/2, 1/2, y, z) / |t| (f_t = y^(n/2) z^(1/2) / (B(n/2, 1/2) |t|)), with
y = fl(n/u), z = fl(tt/u) the tails' arguments. "Ideal" variants: every operation exact except the named stored
pieces (Excel's LN, Excel's log1p series for alnrel, the double y and z):
  brc_ideal   RN(exp(a lnx + b lny) / B / |t|), lnx, lny by brcomp's 0.375 switch with Excel LN and log1p
  brc_ideal_exactlogs  the same with exact logs of the double y, z (only y, z rounding)
  brc_ideal_tt  exact logs of y = n/u, z = tt/u with tt, u doubles but y, z not rounded
  CR          exact density
usage: pdf_brc.py"""
from collections import defaultdict

import mpmath

import tlib
import xlk

bx = tlib.bx


def logs(y, z, mode):
    def LN(v):
        return mpmath.log(mpmath.mpf(v)) if mode == "exact" else mpmath.mpf(bx.LN(v))

    def L1(v):  # ln(1 + v)
        return mpmath.log1p(mpmath.mpf(v)) if mode == "exact" else mpmath.mpf(xlk.excel_log1p.log1p_series(v))
    if y <= 0.375:
        return LN(y), L1(-y)
    if z > 0.375:
        return LN(y), LN(z)
    return L1(-z), LN(z)


rows = [r for r in tlib.load(("pdf",)) if r["t"] != 0 and r["excel"] != 0]
out = defaultdict(list)
for r in rows:
    t, n = abs(r["t"]), r["n"]
    tt, u, y, z = tlib.targs(t, n)
    a, b = n / 2, 0.5
    with mpmath.workdps(40):
        lnB = mpmath.log(mpmath.beta(mpmath.mpf(a), mpmath.mpf(b)))
        res = {}
        for mode in ("xl", "exact"):
            lx, ly = logs(y, z, mode)
            res["brc_ideal" if mode == "xl" else "brc_ideal_exactlogs"] = float(mpmath.exp(a * lx + b * ly - lnB) / t)
        Y = mpmath.mpf(n) / mpmath.mpf(u)
        Z = mpmath.mpf(tt) / mpmath.mpf(u)
        res["brc_ideal_tt"] = float(mpmath.exp(a * mpmath.log(Y) + b * mpmath.log(Z) - lnB) / t)
        res["CR"] = tlib.exact_pdf(t, n)
    nb = "n=%g" % n if n <= 10 else "n<=100" if n <= 100 else "n>100"
    for k, v in res.items():
        out[(k, "all")].append((v, r["excel"]))
        out[(k, nb)].append((v, r["excel"]))
        if r["hold"]:
            out[(k, "hold")].append((v, r["excel"]))
for k in sorted(out, key=lambda k: (k[1], k[0])):
    print(tlib.line(str(k), out[k]))
