"""Decide bratio's x0 >= 0.29 (R) versus x0 >= 0.3 (TOMS 1992) switch on probe t2464-r2-bratio-x29: on the cells
where the two dispatches give different doubles, which one equals Excel? Split by hold-out half and by b0 <= 15
(bup+bgrat) / b0 > 15 (bgrat)."""
from collections import Counter
from dataclasses import replace

import bratio_excel as bx
import score_lib as sl

rows = [r for r in sl.load(("x29",)) if r["x"] < 0.3]
pts = [(r["a"], r["b"], r["x"], r["y"]) for r in rows]
v29 = bx.run(pts, bx.BEST)
v30 = bx.run(pts, replace(bx.BEST, x29=0.3))
c = Counter()
for r, p, q in zip(rows, v29, v30):
    if p[0] == q[0]:
        c[("same", r["b"] > 15)] += 1
        continue
    key = ("bgrat" if r["b"] > 15 else "bup+bgrat", "hold" if r["hold"] else "tune")
    c[key + ("0.29 matches",)] += p[0] == r["excel"]
    c[key + ("0.3 matches",)] += q[0] == r["excel"]
    c[key + ("differ",)] += 1
for k in sorted(c, key=str):
    print(k, c[k])
