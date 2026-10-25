"""Break tails.pkl scores down by branch x (t <= 1) x df band; usage: brk.py [variant-for-branch]"""
import pickle, sys
from collections import defaultdict
import tlib
rows, vals, br = pickle.load(open(tlib.HERE / "tails.pkl", "rb"))
kv = sys.argv[1] if len(sys.argv) > 1 else "yz"
def band(n):
    return "n<=2" if n <= 2 else "n<=10" if n <= 10 else "n<=30" if n <= 30 else "n<=100" if n <= 100 else "n<=1000" if n <= 1000 else "n>1000"
g = defaultdict(list)
for i, r in enumerate(rows):
    up = (r["kind"] == "cdf") == (r["t"] > 0) and r["kind"] != "2t"
    if up:
        continue
    g[(br[kv][i], "t<=1" if abs(r["t"]) <= 1 else "t>1", band(r["n"]))].append(i)
for k in sorted(g, key=str):
    idx = g[k]
    s = " | ".join(f"{v} {'/'.join(map(str, tlib.TALLY((vals[v][i], rows[i]['excel']) for i in idx)[1:]))}" for v in vals)
    c = '/'.join(map(str, tlib.TALLY((rows[i]['cr'], rows[i]['excel']) for i in idx)[1:]))
    print(k, len(idx), s, "| CR", c)
