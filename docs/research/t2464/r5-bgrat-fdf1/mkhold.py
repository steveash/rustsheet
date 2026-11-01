"""T-3567 r5-fdf1: build the hold-out probe t3567-r5-fdf1-hold (seed 35670). Two cells per point: F.DIST.RT(x,m,n)
in column A and F.DIST(x,m,n,TRUE) in column B. Regions:
  bg1   d1 = 1, n in 31..400, points where fmodel's kernel takes bgrat (b0 = 1/2)
  d1    d1 = 1, n in 3..30, x = e^U(-8,8) n
  d2    d2 = 1, m in 1..400, x = e^U(-8,8)/m
  gen   d1, d2 log-uniform in [1, 500], 1/3 non-integer literals, x = e^U(-6,6) n/m
  tie   m x = n exactly (x = n/m when representable as a short decimal)
Every x is a decimal of at most 15 significant digits. usage: mkhold.py [--write]"""
import json, math, random, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import fmodel  # noqa
CASES = HERE.parents[2].parent / "eval/suites/probes-numeric/cases"
NAME = "t3567-r5-fdf1-hold"
rnd = random.Random(35670)
def dec(v, k=None):
    return float(f"{v:.{k or rnd.randint(3, 15)}g}")
pts = []
cand = []
while len(cand) < 1500:
    n = rnd.randint(31, 400)
    x = dec(math.exp(rnd.uniform(-1, 4)))
    cand.append((x, 1, n))
br = fmodel.run(cand)
bg = [p for p, o in zip(cand, br) if o[2] == "bgrat"]
pts += [("bg1", *p) for p in bg[:500]]
for _ in range(400):
    n = rnd.randint(3, 30)
    pts.append(("d1", dec(n * math.exp(rnd.uniform(-8, 8))), 1, n))
for _ in range(300):
    m = rnd.randint(1, 400)
    pts.append(("d2", dec(math.exp(rnd.uniform(-8, 8)) / m), m, 1))
for _ in range(500):
    m, n = (math.exp(rnd.uniform(0, math.log(500))) for _ in range(2))
    m, n = (dec(v, 4) if rnd.random() < 1 / 3 else float(round(v)) for v in (m, n))
    m, n = max(m, 1.0), max(n, 1.0)
    pts.append(("gen", dec(math.trunc(n) / math.trunc(m) * math.exp(rnd.uniform(-6, 6))), m, n))
for m in (1, 2, 4, 5, 8, 10, 16, 20, 25, 40):
    for n in (1, 3, 7, 9, 31, 50, 64, 100, 125):
        q = n / m
        if float(f"{q:.15g}") * m == n:
            pts.append(("tie", q, m, n))
cells, meta = {}, []
for k, (reg, x, m, n) in enumerate(pts, 1):
    f = lambda v: repr(v) if v != int(v) else str(int(v))
    cells[f"A{k}"] = f"=F.DIST.RT({f(x)},{f(m)},{f(n)})"
    cells[f"B{k}"] = f"=F.DIST({f(x)},{f(m)},{f(n)},TRUE)"
    meta.append((k, reg, x, m, n))
case = {"format": "rustsheet-case/v1",
        "description": "T-3567 r5-fdf1 hold-out: F.DIST.RT and F.DIST(.,TRUE) at d1 = 1 (bgrat at b = 1/2), d2 = 1 and general dfs",
        "tags": ["M9", "probe", "statistics", "T-3567", "incomplete-beta"],
        "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
from collections import Counter
print(len(pts), "points", len(cells), "cells", Counter(p[0] for p in pts))
if "--write" in sys.argv:
    (CASES / f"{NAME}.case.json").write_text(json.dumps(case, indent=1) + "\n")
    json.dump(meta, open(HERE / f"{NAME}.meta.json", "w"))
