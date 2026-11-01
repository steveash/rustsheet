"""T-3570 r5-bpser-lg: per (center, b0), the ceiling of any single g per a0 (best count of the 8 cells over g within
+-R ulps of Excel's GAMMALN(a0)), and which x positions miss at that best g.
usage: ceil.py [R=12]"""
import math
import sys
from collections import Counter, defaultdict

import pin

kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
R = int(kw.get("R", "12"))
pts = pin.load()
agg = defaultdict(Counter)
for p in pts:
    g = p["gl"]
    for _ in range(R):
        g = math.nextafter(g, -math.inf)
    best, bestk, bestmiss = -1, None, None
    for k in range(-R, R + 1):
        hit = [pin.predict(p, g, x) == e for x, e in p["cells"]]
        if sum(hit) > best:
            best, bestk, bestmiss = sum(hit), k, [i for i, h in enumerate(hit) if not h]
        g = math.nextafter(g, math.inf)
    key = (f"{p['a0']:.2f}", p["b0"])
    t = agg[key]
    t["n"] += 1
    t["cells"] += 8
    t["ceil"] += best
    t["full"] += best == 8
    t[f"k{bestk:+d}"] += 1
    for i in bestmiss:
        t[f"x{i}"] += 1
for key in sorted(agg):
    t = agg[key]
    ks = " ".join(f"{k}:{v}" for k, v in sorted(t.items()) if k.startswith("k"))
    xs = " ".join(f"{k}:{v}" for k, v in sorted(t.items()) if k.startswith("x"))
    print(f"a0~{key[0]} b0={key[1]:5} n={t['n']} full={t['full']:2d} ceil={t['ceil']}/{t['cells']}  best-k {ks} | miss {xs}")
