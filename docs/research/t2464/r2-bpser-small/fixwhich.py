"""For a variant's misses on a run: does moving s (the series) or E (= EXP(z)) by k ulps (|k| <= 2) make the cell
exact? Counts per (which, k), to tell which stored intermediate carries the miss.
usage: fixwhich.py run=<substring> [variant=L53]"""
import math
import sys
from collections import Counter

import lib
import bscore
from lib import sx

kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
v = bscore.VARIANTS[kw.get("variant", "L53")]
rows = [r for r in lib.load_probe(("runs",)) if kw["run"] in r["blk"]]
res = lib.run([(r["a"], r["b"], r["x"], r["y"]) for r in rows], v)
m = lib.Model(lib.bx.BEST, v)


def step(u, k):
    for _ in range(abs(k)):
        u = math.nextafter(u, math.inf if k > 0 else -math.inf)
    return u


c = Counter()
for r, q in zip(rows, res):
    if q[0] == r["excel"]:
        continue
    fn, a, b, x = q[3][-1]
    s, e = m.series_w3(a, b, x), m.w3_pre(a, b, x)
    fixes = []
    for k in (-2, -1, 1, 2):
        if sx("*", e, step(s, k)) == r["excel"]:
            fixes.append(f"s{k:+d}")
        if sx("*", step(e, k), s) == r["excel"]:
            fixes.append(f"E{k:+d}")
    c[" ".join(fixes) or "none"] += 1
print(kw["run"], dict(c.most_common()))
