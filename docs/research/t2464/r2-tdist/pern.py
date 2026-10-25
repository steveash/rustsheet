"""Per-df error table (units 2^-53, model - excel) for one spec on one branch: pern.py BRANCHSUB spec"""
import statistics
import sys
from collections import defaultdict

import tlib
import tscore

rows = tscore.rows_L()
vals, brs = tscore.evaluate(rows, sys.argv[2])
g = defaultdict(list)
for i, r in enumerate(rows):
    if sys.argv[1] in brs[i]:
        g[r["n"]].append(i)
for n in sorted(g):
    idx = g[n]
    e = sorted((vals[i] - rows[i]["excel"]) / abs(rows[i]["excel"]) / 2**-53 for i in idx)
    ex = sum(vals[i] == rows[i]["excel"] for i in idx)
    print(f"n={n:g} cnt={len(idx)} exact={ex} med={statistics.median(e):+.2f} sd={statistics.pstdev(e):.2f} "
          f"min={e[0]:+.1f} max={e[-1]:+.1f}")
