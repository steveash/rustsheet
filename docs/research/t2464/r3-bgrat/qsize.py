"""T-2512 r3-bgrat: the 1 - P identity test split by the size of Q(b, z), where 1 - P loses no more than
ulp(P)/Q relative precision (Q >= 0.25: under 4 units), and by bgrat's b."""
import pickle
import sys
from collections import defaultdict

import bscore as score
import score_lib as sl

cs = score.cells(sys.argv[1] if len(sys.argv) > 1 else "all")
qx = pickle.load(open("qxl.pkl", "rb"))
g = defaultdict(list)
for c in cs:
    k = ("1-P", c["b"], c["z"])
    if c["b"] == 0.5 or k not in qx:
        continue
    band = "Q>=0.25" if c["Q"] >= 0.25 else ("Q>=0.01" if c["Q"] >= 0.01 else "Q<0.01")
    bb = "b=1" if c["b"] == 1 else ("b<1")
    g[(c["br"].replace("/sw", ""), bb, band)].append(c)
print("| branch | b | Q band | n | M0 | EXQ | XLQ:1-P |")
print("|---|---|---|---|---|---|---|")
for k in sorted(g):
    v = g[k]
    m0 = sl.tally((c["fn"](c["g"]), c["row"]["excel"]) for c in v)
    e = sl.tally((score.model(c), c["row"]["excel"]) for c in v)
    x = sl.tally((score.model(c, qx[("1-P", c["b"], c["z"])]), c["row"]["excel"]) for c in v)
    print(f"| {k[0]} | {k[1]} | {k[2]} | {len(v)} | {m0[1]}/{m0[2]}/{m0[3]} | {e[1]}/{e[2]}/{e[3]} | {x[1]}/{x[2]}/{x[3]} |")
