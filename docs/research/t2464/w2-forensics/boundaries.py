"""Straddle runs (xruns-2 blocks 3-7, centred on a boundary double): Excel's error statistics below vs above."""
import csv, collections, math
import numpy as np
rows = [r for r in csv.DictReader(open("exact_w2f.tsv"), delimiter="\t") if r["case"] == "t2464-w2f-xruns-2"]
names = {3: ("b*x=1", lambda a, b: 1 / b), 4: ("x=0.95", lambda a, b: 0.95), 5: ("mean", lambda a, b: a / (a + b)),
         6: ("mean", lambda a, b: a / (a + b)), 7: ("x=0.5", lambda a, b: 0.5)}
g = collections.defaultdict(list)
for r in rows:
    i = int(r["cell"].split("!A")[1]) - 1
    g[(i // 2048, r["cum"])].append(r)
for (blk, cum), rs in sorted(g.items()):
    if blk not in names: continue
    a, b = float(rs[0]["a"]), float(rs[0]["b"]); nm, f = names[blk]; c = f(a, b)
    lo = [r for r in rs if float(r["x"]) <= c]; hi = [r for r in rs if float(r["x"]) > c]
    def st(s):
        e = np.array([float(r["rel"]) for r in s]); u = np.array([float(r["err_ulp"]) for r in s])
        return f"n={len(s)} mean {e.mean():+.2f} sd {e.std():.2f} exact {np.mean(u == 0):.2f}"
    # nearest 64 each side
    lo64, hi64 = lo[-64:], hi[:64]
    print(f"{nm:7s} a={a} b={b} {'cdf' if cum=='1' else 'pdf'} | below: {st(lo)} | above: {st(hi)} | 64 nearest: {st(lo64)} || {st(hi64)}")
