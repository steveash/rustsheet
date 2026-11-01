"""T-3568 r5-bup: bup's terms as R_i X_i with x^i through Excel's EXP and LN; cell scores and residual sd.

Term i of S is R_i X_i. R_i: 'Rex' the exact product of TOMS's stored ratios fl(apb+l)/fl(ap1+l); 'Rd' that product
accumulated in doubles; 'Rld' in x87 extended. X_i: 'Xex' y0^i exact; 'Xd' EXP(fl(i LN(y0))) (Excel's EXP and LN,
r2-bratio); 'Xx' EXPX(fl64(i LN(y0))) (the product kept extended); 'Xe' exp(fl(i ln y0)) with exact exp and a correctly
rounded ln. Terms and the sum in doubles ('sd') or extended ('sx') or exact ('se'). W = x87(x87(P fl(xy)/c) S).
Cell = RN(W + D). usage: powforms2.py tune|hold"""
from __future__ import annotations

import statistics
import sys
from collections import defaultdict

import mpmath
import numpy as np

import score as s
from common import sl, xlk

bx = xlk.bx
LD = np.longdouble
M = mpmath.mpf
U = M(2) ** -53
k_terms = s.k_terms


def s_of(r, rform, xform, sform, brk=True):
    c, a0, y0, x0, n = r["c"], r["a0"], r["y0"], r["x0"], r["n"]
    apb, ap1 = c + a0, c + 1.0
    lnx = bx.LN(y0)
    lncr = float(mpmath.log(y0))
    k = k_terms(c, a0, y0, x0, n)
    R = M(1) if rform == "Rex" else (LD(1) if rform == "Rld" else 1.0)
    w = M(1) if sform == "se" else (LD(1) if sform == "sx" else 1.0)
    for i in range(1, n):
        l = i - 1
        q = (apb + l) / (ap1 + l)
        R = R * (M(q) if rform == "Rex" else (LD(q) if rform == "Rld" else q))
        if xform == "Xex":
            X = M(y0) ** i
        elif xform == "Xd":
            X = M(bx.EXP(i * lnx))
        elif xform == "Xx":
            X = M(bx.EXPX(bx.em.fl64(M(i) * M(lnx))))
        else:
            X = mpmath.exp(M(i * lncr))
        t = M(R) * X if rform == "Rex" else M(float(LD(R) * LD(float(X))))
        if sform == "se":
            w += t
        elif sform == "sx":
            w = w + LD(float(t))
        else:
            w = w + float(t)
        if brk and i - 1 >= k and float(t) <= s.EPS * float(w):
            break
    return w


rows = [r for r in s.load(sys.argv[1]) if r["P"] != 0 and r["region"] == "beta" and r["dbr"] == "bgrat"]
combos = [("Rex", "Xex", "se"), ("Rex", "Xe", "se"), ("Rex", "Xd", "se"), ("Rex", "Xx", "se"),
          ("Rd", "Xd", "sd"), ("Rd", "Xd", "sx"), ("Rld", "Xd", "sx"), ("Rd", "Xx", "sd"), ("Rld", "Xx", "sx"),
          ("Rd", "Xe", "sd")]
g = defaultdict(lambda: defaultdict(list))
res = defaultdict(lambda: defaultdict(list))
for r in rows:
    ret = s.ret_of(r, "x87")
    se = s.bup_sum(r["c"], r["a0"], r["y0"], r["x0"], r["n"], "ex")
    es = float(((M(r["excel"]) - r["D"]) / (M(float(ret)) * se) - 1) / U)
    nb = "n<=9" if r["n"] <= 9 else "n>=10"
    for cmb in combos:
        sv = s_of(r, *cmb)
        w = float(LD(ret) * LD(float(sv)))
        v = float(M(w) + r["D"])
        nm = "/".join(cmb)
        for key in (nb, "all"):
            g[key][nm].append((v, r["excel"]))
            res[key][nm].append(es - float((M(float(sv)) / se - 1) / U))
print("| R/X/sum | " + " | ".join(f"{b} exact/w4/w15" for b in sorted(g)) + " | " + " | ".join(f"{b} resid sd" for b in sorted(g)) + " |")
print("|---|" + "---|" * (2 * len(g)))
for cmb in combos:
    nm = "/".join(cmb)
    sc = []
    for b in sorted(g):
        n, e, f, w = sl.tally(g[b][nm])
        sc.append(f"{e}/{f}/{w} ({n})")
    print(f"| {nm} | " + " | ".join(sc) + " | " + " | ".join(f"{statistics.pstdev(res[b][nm]):.1f}" for b in sorted(g)) + " |")
