"""T-3568 r5-bup: Excel's implied bup sum against the exact one, in units of 2^-53, by region and n.

rel = (Excel - D) / (P fl(x0 y0)/c * S_exact) - 1, with Excel's P and D (probe columns C, B). If Excel's W were
P fl(xy)/c S rounded a few times, rel would sit within a few units at every n; a drift with n points at the sum or
at a different prefactor. usage: resid.py tune|hold"""
from __future__ import annotations

import statistics
import sys
from collections import defaultdict

import mpmath

import score as s

U = mpmath.mpf(2) ** -53
rows = s.load(sys.argv[1])
g = defaultdict(list)
for r in rows:
    se = s.bup_sum(r["c"], r["a0"], r["y0"], r["x0"], r["n"], "ex")
    w = mpmath.mpf(r["P"]) * (r["y0"] * r["x0"]) / r["c"] * se
    if w == 0:
        g[("P = 0", r["region"], "")].append(0.0)
        continue
    rel = float(((mpmath.mpf(r["excel"]) - r["D"]) / w - 1) / U)
    nb = "n<=5" if r["n"] <= 5 else "n 6-15" if r["n"] <= 15 else "n 16-39"
    g[(r["region"], r["dbr"], nb)].append(rel)
    g[(r["region"], "all", "all")].append(rel)
print("| region | D branch | n | cells | median | sd | min | max |")
print("|---|---|---|---|---|---|---|---|")
for k in sorted(g):
    v = g[k]
    sd = statistics.pstdev(v) if len(v) > 1 else 0.0
    print(f"| {k[0]} | {k[1]} | {k[2]} | {len(v)} | {statistics.median(v):.2f} | {sd:.2f} | {min(v):.1f} | {max(v):.1f} |")
