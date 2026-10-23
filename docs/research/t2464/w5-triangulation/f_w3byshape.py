import math, sys
from collections import defaultdict
import numpy as np
sys.path.insert(0, '.'); sys.path.insert(0, 'w3copy')
import w5lib as L
from dispatch import bratio_branch
import model as W3
rows = []
for c in L.cells({"F.DIST", "F.DIST.RT", "FDIST"}):
    a_ = c["args"]
    if c["fn"] == "F.DIST" and a_[3] is not True: continue
    x, d1, d2 = a_[0], float(math.trunc(a_[1])), float(math.trunc(a_[2]))
    if not (x > 0 and d1 >= 1 and d2 >= 1) or d1 > 1e10 or d2 > 1e10: continue
    if c["excel"] in (0.0, 1.0): continue
    a, b = d1 / 2, d2 / 2
    p = d1 * x; w = 1 / (1 + d2 / p); v = 1 / (1 + p / d2)
    tail = "lower" if c["fn"] == "F.DIST" else "upper"
    br, ind, comp = bratio_branch(a, b, w, v)
    if tail != comp or br != "bpser": continue
    rows.append((c, (w, a, b) if not ind else (v, b, a), ind))
for zmode in ("z53", "z64"):
    m = W3.model(*(np.array(t) for t in zip(*[r[1] for r in rows])), zmode=zmode, big=8)
    by = defaultdict(list)
    for mi, (c, (xx, aa, bb), ind) in zip(m, rows):
        by[(aa, bb)].append((float(mi), c["excel"]))
    print("==", zmode)
    for k in sorted(by): print("  a0,b0 =", k, L.fmt(L.score(by[k])))
