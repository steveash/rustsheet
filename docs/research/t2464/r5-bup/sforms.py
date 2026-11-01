"""T-3568 r5-bup: which bup sum S makes Excel's W? sd of eW - eS over BETA.DIST cells (D by bgrat, P > 0).

e = (Excel - D) / (P_xl fl(xy)/c S) - 1 in units of 2^-53, for S in exact arithmetic from different inputs:
  stored     apb = fl(c + a0), ap1 = fl(c + 1) (TOMS), terms (apb+l)/(ap1+l) x
  exactapb   apb = c + a0 exactly
  b0form     terms from b0 directly: (a0 + b0 - n + l)/(b0 - n + 1 + l) with b0 - n stored
  rev        terms counted down from b0: the recurrence in the caller's orientation, I(a0, b0) = I(a0, b0-1) + ...
             (same math; stored b0 - j)
and the arithmetic forms of score.bup_sum (d, ld, w4). usage: sforms.py tune|hold"""
from __future__ import annotations

import statistics
import sys
from collections import defaultdict

import mpmath

import score as s

U = mpmath.mpf(2) ** -53
M = mpmath.mpf


def s_exact(c, a0, x, n, apb, ap1):
    d = w = M(1)
    for i in range(n - 1):
        d = d * (M(apb) + i) / (M(ap1) + i) * x
        w += d
    return w


def s_rev(a0, b0, x, n):
    """Sum the terms from the top shape down: t_top = 1, t_{j-1}/t_j = (b0 - j)/(a0 + b0 - j - 1) / x ..."""
    # the same ratios as bup's, inverted and with the shapes stored from b0 (b0 - j in double)
    terms = [M(1)]
    for j in range(1, n):
        bj = b0 - j  # double, exact for these b0
        terms.append(terms[-1] * M(bj) / (M(a0 + bj - 1.0 + 1.0) - 1) / x * 1)
    # rescale so the lowest term is 1 (bup's d_0)
    return sum(terms) / terms[-1]


g = defaultdict(lambda: defaultdict(list))
for r in s.load(sys.argv[1]):
    if r["P"] == 0 or r["region"] != "beta" or r["dbr"] != "bgrat":
        continue
    c, a0, n, b0 = r["c"], r["a0"], r["n"], r["b0"]
    x = M(r["y0"])
    pre = M(r["P"]) * (r["y0"] * r["x0"]) / c
    wx = M(r["excel"]) - r["D"]
    forms = {
        "stored apb (TOMS), exact sum": s_exact(c, a0, x, n, c + a0, c + 1.0),
        "exact apb": s_exact(c, a0, x, n, M(c) + a0, M(c) + 1),
        "sum d (doubles)": M(s.bup_sum(c, a0, r["y0"], r["x0"], n, "d")),
        "sum ld (x87)": M(float(s.bup_sum(c, a0, r["y0"], r["x0"], n, "ld"))),
        "sum w4": M(s.bup_sum(c, a0, r["y0"], r["x0"], n, "w4")),
    }
    nb = "n<=9" if n <= 9 else "n>=10"
    for f, sv in forms.items():
        e = float((wx / (pre * sv) - 1) / U)
        g[nb][f].append(e)
        g["all"][f].append(e)
print("| sum form | " + " | ".join(sorted(g)) + " |")
print("|---|" + "---|" * len(g))
for f in next(iter(g.values())):
    print(f"| {f} | " + " | ".join(f"median {statistics.median(g[k][f]):.1f}, sd {statistics.pstdev(g[k][f]):.1f} ({len(g[k][f])})"
                                    for k in sorted(g)) + " |")
