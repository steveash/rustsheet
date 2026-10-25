"""Breakdown of a scored pickle (vscore.py output): by source, and by shape for shapes with >= 200 cells."""
import pickle, sys, math, os
from collections import defaultdict
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from vscore import w15
U = pickle.load(open(sys.argv[1], "rb"))
def t(rs, k):
    return (sum(r[k] == r["excel"] for r in rs), sum(abs(r[k]-r["excel"]) <= 4*math.ulp(r["excel"]) for r in rs), sum(w15(r[k], r["excel"]) for r in rs))
g = defaultdict(list); s = defaultdict(list)
for r in U:
    g[r["src"].split(":")[0] if not r["src"].startswith("w2") else r["src"]].append(r)
    s[(r["src"] == "p1", r["a"], r["b"])].append(r)
for k, rs in sorted(g.items()):
    print(f"{k:40s} n={len(rs):6d} model {t(rs,'model')} CR {t(rs,'cr')}")
small = []
for k in sorted(s):
    rs = s[k]
    if len(rs) >= 200:
        print(("p1 " if k[0] else "r1 ") + f"({k[1]:g},{k[2]:g}) n={len(rs)} model {t(rs,'model')} CR {t(rs,'cr')}")
    else:
        small += rs
print(f"shapes with <200 cells: n={len(small)} model {t(small,'model')} CR {t(small,'cr')}")
