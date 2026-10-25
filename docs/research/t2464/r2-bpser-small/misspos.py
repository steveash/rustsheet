"""Positions (index in the run) of a variant's misses on one run, as a 0/1 string in rows of 64, with the signed
ulp error of each miss; plus E*s's rounding position (fraction of an ulp above the lower double) for misses vs hits.
usage: misspos.py run=<substring> [variant=L53]"""
import sys
from collections import Counter

import lib
import bscore

kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
v = bscore.VARIANTS[kw.get("variant", "L53")]
rows = [r for r in lib.load_probe(("runs",)) if kw["run"] in r["blk"]]
rows.sort(key=lambda r: r["x"])
res = lib.run([(r["a"], r["b"], r["x"], r["y"]) for r in rows], v)
m = lib.Model(lib.bx.BEST, v)
line, frac = "", Counter()
for i, (r, q) in enumerate(zip(rows, res)):
    d = round(lib.ulps_signed(q[0], r["excel"]))
    line += "." if d == 0 else ("+" if d > 0 else "-")
    fn, a, b, x = q[3][-1]
    s = m.series_w3(a, b, x)
    e = m.w3_pre(a, b, x)
    p = lib.mpf(e) * s
    lo = float(p)
    if lo > p:
        lo = lib.math.nextafter(lo, -lib.math.inf)
    f = float((p - lo) / lib.math.ulp(lo))
    frac[(d == 0, round(f * 10) / 10)] += 1
for k in range(0, len(line), 64):
    print(line[k : k + 64])
print("E*s position within its ulp (tenths) for hits / misses:")
for hit in (True, False):
    print("  hit " if hit else "  miss", [frac[(hit, t / 10)] for t in range(11)])
