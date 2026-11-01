"""T-3568 r5-bup: two checks on the frozen model's misses (t3568-r5-bup-tune and -hold).

1. F rows split by whether F's y0 equals fl(1 - x0): the probe's companions are BETA.DIST(x0, a0, c, TRUE) and
   BETA.DIST(y0, c, a0, FALSE), which re-derive the other argument as 1 - (their x), so where y0 != fl(1 - x0) P is
   Excel's density at a neighbouring point, not at F's (x0, y0).
2. Excel's D = BETA.DIST(x0, a0, c, TRUE) returned as 0 where the true value is a normal double: counted by
   region and the D point's branch, with the true value's range. usage: fsplit.py"""
from __future__ import annotations

from collections import defaultdict

import mpmath

import model
import score as s
from common import sl

for half in ("tune", "hold"):
    g = defaultdict(list)
    zero = defaultdict(list)
    for r in s.load(half):
        if r["region"] == "f":
            key = ("f", "c=1" if r["c"] == 1.0 else "c=1/2", "y0 = fl(1-x0)" if r["y0"] == 1.0 - r["x0"] else "y0 != fl(1-x0)")
            g[key].append((model.cell(r), r["excel"]))
        if r["D"] == 0.0:
            t = float(mpmath.betainc(r["a0"], r["c"], 0, r["x0"], regularized=True))
            if t >= 2.2250738585072014e-308:
                zero[(r["region"], r["dbr"])].append(t)
    print(f"## {half}\n\n| F rows | cells | FROZEN exact/w4/w15 |\n|---|---|---|")
    for k in sorted(g):
        n, e, f, w = sl.tally(g[k])
        print(f"| {' '.join(k[1:])} | {n} | {e}/{f}/{w} |")
    print("\n| D = 0 with a normal true value: region, D branch | cells | true D range |\n|---|---|---|")
    for k in sorted(zero):
        v = zero[k]
        print(f"| {k[0]}, {k[1]} | {len(v)} | {min(v):.3g} to {max(v):.3g} |")
    print()
