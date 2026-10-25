"""Error statistics (units of 2^-53 relative, model - excel) per branch, for tails.pkl variants."""
import pickle, statistics, sys
from collections import defaultdict
import tlib
rows, vals, br = pickle.load(open(tlib.HERE / "tails.pkl", "rb"))
g = defaultdict(list)
for i, r in enumerate(rows):
    up = (r["kind"] == "cdf") == (r["t"] > 0) and r["kind"] != "2t"
    if up or r["excel"] == 0:
        continue
    g[br["yz"][i]].append(i)
for k, idx in sorted(g.items()):
    out = [f"{k} n={len(idx)}"]
    for v in list(vals) + ["cr"]:
        e = [((vals[v][i] if v != "cr" else rows[i]["cr"]) - rows[i]["excel"]) / abs(rows[i]["excel"]) / 2**-53 for i in idx]
        out.append(f"{v}: med {statistics.median(e):+.1f} sd {statistics.pstdev(e):.1f}")
    print(" | ".join(out))
