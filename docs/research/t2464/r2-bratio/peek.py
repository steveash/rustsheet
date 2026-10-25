"""Per-block error summary of chosen configs from results.pkl: peek.py <set> [branch-substring]."""
import pickle
import statistics
import sys
from collections import defaultdict

import score_lib as sl

rows, res = pickle.load(open(sl.HERE / "results.pkl", "rb"))
want_set = sys.argv[1]
want_br = sys.argv[2] if len(sys.argv) > 2 else ""
names = sys.argv[3].split(";") if len(sys.argv) > 3 else list(res)
g = defaultdict(list)
for i, r in enumerate(rows):
    br = res[names[-1]][i][2]
    if r["set"] == want_set and want_br in br:
        g[(r["blk"], round(r["a"], 4), round(r["b"], 4), br)].append(i)
for k, idx in sorted(g.items(), key=str):
    line = [f"{k} n={len(idx)}"]
    for n in names + ["CR"]:
        v = [rows[i]["cr"] if n == "CR" else sl.side_value(rows[i], res[n][i]) for i in idx]
        e = [(m - rows[i]["excel"]) / max(abs(rows[i]["excel"]), 1e-300) / 2**-53 for m, i in zip(v, idx)]
        ex = sum(m == rows[i]["excel"] for m, i in zip(v, idx))
        line.append(f"{n}: ex {ex} med {statistics.median(e):+.1f} sd {statistics.pstdev(e):.1f}")
    print(" | ".join(line))
