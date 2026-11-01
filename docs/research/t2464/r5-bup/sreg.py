"""T-3568 r5-bup: regress Excel's implied bup sum error on stored-input rounding errors (BETA.DIST, D bgrat, P > 0).

eS = (Excel - D) / (x87(P fl(xy)/c) S_exact) - 1 in units of 2^-53, S_exact on TOMS's stored apb, ap1.
Regressors (units of 2^-53): eP (the density's own error against 50 digits), the derivative of ln S along
apb, ap1 and x times each input's rounding error for candidate stored forms, and n. usage: sreg.py tune|hold"""
from __future__ import annotations

import sys

import mpmath
import numpy as np

import score as s

M = mpmath.mpf
U = M(2) ** -53
X, Y, NS = [], [], []
for r in s.load(sys.argv[1]):
    if r["P"] == 0 or r["region"] != "beta" or r["dbr"] != "bgrat":
        continue
    c, a0, n, x0, y0 = r["c"], r["a0"], r["n"], r["x0"], r["y0"]
    se = s.bup_sum(c, a0, y0, x0, n, "ex")
    pre = M(float(s.ret_of(r, "x87")))
    es = float(((M(r["excel"]) - r["D"]) / (pre * se) - 1) / U)
    pt = M(y0) ** (c - 1) * M(x0) ** (a0 - 1) / mpmath.beta(c, a0)
    ep = float((r["P"] / pt - 1) / U)
    # d ln S / d ln(input), numerically
    h = M(10) ** -20

    def sens(apb_s=0, ap1_s=0, x_s=0):
        apb, ap1, xx = M(c + a0) * (1 + apb_s), M(c + 1.0) * (1 + ap1_s), M(y0) * (1 + x_s)
        d = w = M(1)
        for i in range(n - 1):
            d = d * (apb + i) / (ap1 + i) * xx
            w += d
        return w
    s0 = sens()
    da = float((sens(apb_s=h) / s0 - 1) / h)
    d1 = float((sens(ap1_s=h) / s0 - 1) / h)
    dx = float((sens(x_s=h) / s0 - 1) / h)
    # rounding errors of candidate stored inputs, relative, units of 2^-53
    e_apb = float((M(c + a0) - (M(c) + a0)) / (M(c) + a0) / U)
    e_ap1 = float((M(c + 1.0) - (M(c) + 1)) / (M(c) + 1) / U)
    b = r["b0"]
    e_c = float((M(b - n) - (M(b) - n)) / U)  # zero: c is exact
    X.append([ep, da * e_apb, d1 * e_ap1, dx, float(n), 1.0])
    Y.append(es)
    NS.append(n)
A, y = np.array(X), np.array(Y)
names = ["eP", "dlnS/dlnapb * e_apb", "dlnS/dlnap1 * e_ap1", "dlnS/dlnx (unit x error)", "n", "const"]
for sel, lab in ((np.array(NS) <= 9, "n<=9"), (np.array(NS) >= 10, "n>=10"), (np.ones(len(NS), bool), "all")):
    coef, *_ = np.linalg.lstsq(A[sel], y[sel], rcond=None)
    res = y[sel] - A[sel] @ coef
    print(f"{lab}: cells {sel.sum()}, sd eS {y[sel].std():.2f} -> residual sd {res.std():.2f}")
    for nm, cf in zip(names, coef):
        print(f"   {nm:28s} {cf:+.3f}")
