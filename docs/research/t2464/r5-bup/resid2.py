"""T-3568 r5-bup: is the n >= 2 W error Excel's density error? Binned by a0 and n (BETA.DIST, D by bgrat, non-zero P).

eW = (Excel - D)/W_true - 1 and eP = P_xl/P_true - 1, both in units of 2^-53, with W_true = P_true fl(xy)/c S_exact
and P_true the 50-digit density at (y0; c, a0). If W = P_xl fl(xy)/c S (a few roundings), eW - eP is small.
usage: resid2.py tune|hold"""
from __future__ import annotations

import statistics
import sys
from collections import defaultdict

import mpmath

import score as s

U = mpmath.mpf(2) ** -53
g = defaultdict(list)
for r in s.load(sys.argv[1]):
    if r["P"] == 0 or r["region"] not in ("beta", "f") or r["dbr"] != "bgrat":
        continue
    c, a0, y, x = r["c"], r["a0"], mpmath.mpf(r["y0"]), mpmath.mpf(r["x0"])
    pt = y ** (c - 1) * x ** (a0 - 1) / mpmath.beta(c, a0)
    se = s.bup_sum(c, a0, r["y0"], r["x0"], r["n"], "ex")
    wt = pt * (r["y0"] * r["x0"]) / c * se
    ew = float(((mpmath.mpf(r["excel"]) - r["D"]) / wt - 1) / U)
    ep = float((r["P"] / pt - 1) / U)
    ab = "a0<100" if a0 < 100 else "a0 100-500" if a0 < 500 else "a0>=500"
    nb = "n<=9" if r["n"] <= 9 else "n>=10"
    g[(r["region"], ab, nb)].append((ew, ep))
print("| region | a0 | n | cells | sd eW | sd eP | sd eW-eP | corr(eW, eP) |")
print("|---|---|---|---|---|---|---|---|")
for k in sorted(g):
    v = g[k]
    if len(v) < 3:
        continue
    ew, ep = [a for a, _ in v], [b for _, b in v]
    dd = [a - b for a, b in v]
    try:
        co = statistics.correlation(ew, ep)
    except statistics.StatisticsError:
        co = float("nan")
    print(f"| {k[0]} | {k[1]} | {k[2]} | {len(v)} | {statistics.pstdev(ew):.1f} | {statistics.pstdev(ep):.1f} | "
          f"{statistics.pstdev(dd):.1f} | {co:.2f} |")
