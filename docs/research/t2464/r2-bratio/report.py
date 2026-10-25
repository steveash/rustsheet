"""Compact tables from results.pkl (written by score.py): per bratio branch and per data set, for chosen configs,
with hold-out exact counts, and the list of branches not at visible parity (cells failing w15) for the last config.
usage: report.py "R_LIKE;TOMS_XL;BEST(x29=0.29)" """
import pickle
import sys
from collections import defaultdict

import score_lib as sl
from score import readme_branch

rows, res = pickle.load(open(sl.HERE / "results.pkl", "rb"))
names = sys.argv[1].split(";") if len(sys.argv) > 1 else list(res)
vals = {n: [sl.side_value(r, v) for r, v in zip(rows, res[n])] for n in names}
best = names[-1]
for i, r in enumerate(rows):
    r["branch"] = res[best][i][2].split("/")[0]
sl.table(rows, vals, lambda r: r["branch"], f"per bratio branch (as fired in {best}; swapped and unswapped pooled)")
sl.table(rows, vals, lambda r: (r["set"], readme_branch(r)), "per data set (be/w2f/w4p1: README branch)")
print(f"\n## branches not at visible parity in {best} (cells failing w15; CR's count for comparison)")
print("| branch | n | model fails w15 | CR fails w15 | model fails w15, hold-out half |")
print("|---|---|---|---|---|")
g = defaultdict(list)
for i, r in enumerate(rows):
    g[r["branch"]].append(i)
for b, idx in sorted(g.items(), key=lambda kv: -sum(not sl.w15(vals[best][i], rows[i]["excel"]) for i in kv[1])):
    m = sum(not sl.w15(vals[best][i], rows[i]["excel"]) for i in idx)
    c = sum(not sl.w15(rows[i]["cr"], rows[i]["excel"]) for i in idx)
    h = sum(not sl.w15(vals[best][i], rows[i]["excel"]) for i in idx if rows[i]["hold"])
    print(f"| {b} | {len(idx)} | {m} | {c} | {h} |")
