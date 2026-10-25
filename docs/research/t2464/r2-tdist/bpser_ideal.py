"""bpser at (n/2, 1/2, y) (T, t^2 > n, branch w_bpser) as an 'ideal' identity in the style of the bgrat result:
  I = RN( I_exact(y) * exp(a (LN(y) - ln y)) * EXP_xl(z)/exp(z) ),  z = fl64(a LN(y) - lnB) (extended)
i.e. exact everything but Excel's LN(y), and Excel's EXP of the extended exponent. Variants:
  noexp   without the EXP ratio;  z53   EXP of the stored double z;  w3   the incumbent (r2-bratio BEST)
Data: T tail cells and probe part D/C rows in branch w_bpser. usage: bpser_ideal.py"""
from collections import defaultdict

import mpmath

import p1
import tlib
import tscore

bx, em = tlib.bx, tlib.bx.em


def rows_all():
    rows = tscore.rows_L()
    for i, p, (t, n) in p1.META:
        if p in ("C", "D") and p1.excel(i) not in (None, 0.0):
            rows.append(dict(kind="rt", t=t, n=n, excel=p1.excel(i), hold=tlib.hold(p1.NAME, f"A{i}"), src="p1"))
    for r in rows:
        r.setdefault("src", "cells")
        r["cr"] = tlib.exact_cell(r)
    return rows


rows = rows_all()
vals, brs = tscore.evaluate(rows, p1.FINAL)
out = defaultdict(list)
for r, v, b in zip(rows, vals, brs):
    if b != "w_bpser" or r["n"] <= 2:
        continue
    t, n = abs(r["t"]), r["n"]
    tt, u, y, z = tlib.targs(t, n)
    a = n / 2
    sc = 1 if r["kind"] == "2t" else 0.5
    with mpmath.workdps(50):
        Y = mpmath.mpf(y)
        I = mpmath.betainc(a, 0.5, 0, Y, regularized=True)
        lny = mpmath.mpf(bx.LN(y))
        lnB = mpmath.log(mpmath.beta(a, 0.5))
        fac = mpmath.exp(a * (lny - mpmath.log(Y)))
        zx = em.fl64(a * lny - lnB)
        z53 = em.fl53(a * lny - lnB)
        c = {
            "ideal": I * fac * bx.EXPX(zx) / mpmath.exp(zx),
            "noexp": I * fac,
            "z53": I * fac * bx.EXP(z53) / mpmath.exp(mpmath.mpf(z53)),
            "exact_y": I,
        }
    for k, val in c.items():
        out[(k, r["src"])].append((float(val * sc), r["excel"]))
    out[("w3 (FINAL)", r["src"])].append((v, r["excel"]))
    out[("CR", r["src"])].append((r["cr"], r["excel"]))
for k in sorted(out, key=lambda k: (k[1], k[0])):
    print(tlib.line(str(k), out[k]))
