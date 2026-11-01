"""T-3568 r5-bup: does eW - eP scale with the density's exponent |ln P| or with n? (BETA.DIST, D by bgrat, P > 0)

eW, eP as in resid2.py (units of 2^-53). A different exponent rounding in bup's prefactor than in BETA.DIST's density
gives an independent error of up to ulp(|ln W|)/2 relative, so its sd grows with |ln P|, not with n.
usage: expbins.py tune|hold"""
from __future__ import annotations

import statistics
import sys
from collections import defaultdict

import mpmath

import score as s

U = mpmath.mpf(2) ** -53
M = mpmath.mpf
g = defaultdict(list)
for r in s.load(sys.argv[1]):
    if r["P"] == 0 or r["region"] != "beta" or r["dbr"] != "bgrat":
        continue
    c, a0, n = r["c"], r["a0"], r["n"]
    y, x = M(r["y0"]), M(r["x0"])
    pt = y ** (c - 1) * x ** (a0 - 1) / mpmath.beta(c, a0)
    se = s.bup_sum(c, a0, r["y0"], r["x0"], n, "ex")
    wt = pt * (r["y0"] * r["x0"]) / c * se
    ew = float(((M(r["excel"]) - r["D"]) / wt - 1) / U)
    ep = float((r["P"] / pt - 1) / U)
    le = abs(float(mpmath.log(pt)))
    eb = "|lnP|<8" if le < 8 else "|lnP| 8-64" if le < 64 else "|lnP| 64-256" if le < 256 else "|lnP|>=256"
    lw = abs(float(mpmath.log(wt)))
    wb = "|lnW|<8" if lw < 8 else "|lnW| 8-64" if lw < 64 else "|lnW| 64-256" if lw < 256 else "|lnW|>=256"
    nb = "n<=9" if n <= 9 else "n>=10"
    g[("by lnP", eb, nb)].append(ew - ep)
    g[("by lnW", wb, nb)].append(ew - ep)
print("| key | bin | n | cells | median | sd |")
print("|---|---|---|---|---|---|")
for k in sorted(g):
    v = g[k]
    print(f"| {k[0]} | {k[1]} | {k[2]} | {len(v)} | {statistics.median(v):.1f} | {statistics.pstdev(v):.1f} |")
