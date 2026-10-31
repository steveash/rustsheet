"""T-3269 R4 fresh sweep (decision 0188 R4.2): written after the model was frozen, filled once.

Log-uniform over the far tail's domain and past it: x from 1 to 1000 at the shapes the model
covers and beyond (POISSON k 1 to 21, so a = 2 to 22; CHISQ.DIST.RT at odd df 3 to 41, so
a = 1.5 to 20.5), each x rounded to 6 significant digits (well inside Excel's 15-digit literals).
Seeded, so re-running writes the same case. Writes
eval/suites/probes-numeric/cases/t3269-fresh-sweep.case.json.

Run: python3 -I docs/research/t3269/fresh_sweep.py
"""

import json
import math
import random
from pathlib import Path

CASE = Path(__file__).resolve().parents[3] / (
    "eval/suites/probes-numeric/cases/t3269-fresh-sweep.case.json"
)
rng = random.Random(3269)
cells, hypotheses = {}, {}
for n in range(1, 241):
    x = float(f"{math.exp(rng.uniform(0.0, math.log(1000.0))):.6g}")
    if n % 2:
        k = rng.randint(1, 21)
        formula, label = f"=POISSON({k},{x:g},TRUE)", f"fresh/poisson/k={k}/mean={x:g}"
    else:
        df = rng.randrange(3, 42, 2)
        formula, label = f"=CHISQ.DIST.RT({2 * x:g},{df})", f"fresh/chisq/df={df}/x={2 * x:g}"
    cells[f"A{n}"] = formula
    hypotheses[f"Calc!A{n}"] = label
case = {
    "format": "rustsheet-case/v1",
    "description": "T-3269 fresh log-uniform sweep after the far-tail model was frozen (decision 0188 R4.2): "
    "POISSON(k,x,TRUE), k 1 to 21, and CHISQ.DIST.RT(2x,df), odd df 3 to 41, x log-uniform from 1 to 1000 "
    "(240 cells; docs/research/t3269/fresh_sweep.py).",
    "tags": ["M9", "probe", "statistics", "T-3269", "poisson", "chisq", "sweep"],
    "expected": {},
    "hypotheses": hypotheses,
    "sheets": [{"name": "Calc", "cells": cells}],
}
CASE.write_text(json.dumps(case, indent=1) + "\n", encoding="utf-8")
print(len(cells))
