"""Per-block summary of Excel's error on the w2f probes (block = 1024 cells of one config)."""
import csv, collections, math
import numpy as np
rows = list(csv.DictReader(open("exact_w2f.tsv"), delimiter="\t"))
def blk(r):
    i = int(r["cell"].split("!A")[1]) - 1
    if r["case"] == "t2464-w2f-unit-shape": return None
    size = 2048 if "xruns" in r["case"] else 1024
    return (r["case"].replace("t2464-w2f-", ""), i // size)
g = collections.defaultdict(list)
for r in rows:
    k = blk(r)
    if k: g[k + (r["cum"],)].append(r)
for k in sorted(g):
    rs = g[k]; e = np.array([float(r["rel"]) for r in rs]); u = np.array([float(r["err_ulp"]) for r in rs])
    r0 = rs[0]; x = float(r0["x"]); a = float(r0["a"]); b = float(r0["b"])
    print(f"{k[0]:9s} {k[1]} {'cdf' if k[2]=='1' else 'pdf'} a={a:<7.4g} b={b:<6.4g} x={x:<9.5g} m={a/(a+b):.3f} n={len(rs)} "
          f"mean={e.mean():7.2f} sd={e.std():6.2f} exact={np.mean(u==0):.2f} |u|<=1 {np.mean(abs(u)<=1):.2f} range[{e.min():.1f},{e.max():.1f}]")
