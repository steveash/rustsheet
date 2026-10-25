"""T-2586 r5-f-final: the integrator's untouched hold-out t2586-r5-f-final-hold (built and filled BEFORE the final
model was assembled; read once, by final_score.py, after fdensity_excel.py was frozen).

Value rows (seeded, reproducible): 40 random shapes plus 4 with one df below 5, d1 and d2 log-uniform in [1, 500], about a third written as
non-integer literals (Excel truncates them); shapes used by any earlier F.DIST(.,FALSE) probe case are excluded.
Per shape: four runs of 8 consecutive doubles, one per region (side 1 r <= 0.375, side 1 r > 0.375, side 2
r > 0.375, side 2 r <= 0.375, r drawn log-uniform in [1e-3, 0.375] or uniform in (0.375, 1)), plus 4 random singletons
x = exp(U(-6, 6)) * d2/d1. Row cells: A = x (w2-forensics' exact encoder, echoed), B = F.DIST(A, d1, d2, FALSE).
Edge rows: one literal-argument cell each in column D (`EDGES`), parsed with 15-digit truncation by the scorer.
Usage: mkhold.py [--write]"""

from __future__ import annotations

import json
import math
import random
import re
import sys
from pathlib import Path

T = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(T / "r3-fdensity"))
import flib as F  # noqa: E402
import mkprobe as R3P  # noqa: E402  (r3-fdensity: row_cells, case_doc)

MP = F.MP
CASE = "t2586-r5-f-final-hold"
SEED = 25865
EDGES = [
    "0,1,5", "0,1.9,40", "0,2,7.9", "0,2.99,1", "0,3.5,4", "0,250,3",
    "-0.5,3,4", "-1E-300,10,10", "1.25,0.9,4", "1.25,4,0.99", "1.25,0.5,0.5",
    "2.5,10000000000.5,5", "2.5,5,10000000000.5", "2.5,10000000001,5",
    "EXP(-715),3,4", "EXP(-736),1,1", "1E300,1000000000,5", "1E300,100,5", "1.7E306,200,3",
    "1E-300,3,4", "1E-300,1,1", "1E-300,2,9", "1E-150,1,300", "1E300,2,3", "1E300,1,1", "1E150,7,3",
    "0.7,3.3,8.8", "0.123456789012345678,4,6", "1.00000000000000001,10,10", "3.14159265358979,2.5,7.5",
    "0.5,1,1", "1,1,1", "2,2,2", "1,2,2", "0.25,2,1", "9.99,1,2", "1E-8,1,2", "1E8,2,1",
    "1,499.9,1.1", "1,1.1,499.9", "0.002,450,450", "500,450,450", "1,450,450", "1.0000001,300,300",
    "0.95,123.456,321.987", "1.6,17,19", "35,3,60", "0.01,60,3", "TRUE,3,4", "\"2\",3,4",
]


def used_shapes() -> set:
    pat = re.compile(r"F\.DIST\([^,]+,\s*([0-9.eE+-]+)\s*,\s*([0-9.eE+-]+)\s*,\s*(?:FALSE|0)\)", re.I)
    out = set()
    for p in MP.CASES.glob("*.case.json"):
        if p.name.startswith(CASE):
            continue
        for m in pat.finditer(p.read_text()):
            try:
                out.add((math.trunc(float(m.group(1))), math.trunc(float(m.group(2)))))
            except ValueError:
                pass
    return out


def _df(rng: random.Random) -> float:
    v = math.exp(rng.uniform(0, math.log(500)))
    if rng.random() < 0.35:
        return float(f"{v:.3g}") if v >= 1 else 1.5
    return float(max(1, round(v)))


def shapes(rng: random.Random, k: int = 40, k_small: int = 4):
    """k random shapes, then k_small more with one df in [1, 5) (the small-df region)."""
    used, out = used_shapes(), []
    while len(out) < k + k_small:
        d1, d2 = _df(rng), _df(rng)
        if len(out) >= k:
            d1 = float(rng.choice((1, 2, 3, 4, 1.7, 3.6)))
            d1, d2 = (d1, d2) if rng.random() < 0.5 else (d2, d1)
        key = (math.trunc(d1), math.trunc(d2))
        if key in used or key in {(math.trunc(a), math.trunc(b)) for a, b in out}:
            continue
        out.append((d1, d2))
    return out


def plan():
    """[(block, d1, d2, x)] in sheet-row order (d1, d2 as written; Excel truncates)."""
    rng = random.Random(SEED)
    out = []
    for d1, d2 in shapes(rng):
        m, n = math.trunc(d1), math.trunc(d2)
        for reg in ("s1lo", "s1hi", "s2hi", "s2lo"):
            r = math.exp(rng.uniform(math.log(1e-3), math.log(0.375))) if reg.endswith("lo") else rng.uniform(0.38, 0.995)
            x0 = r * n / m if reg.startswith("s1") else n / (r * m)
            out += [(f"{reg}({d1:g},{d2:g})", d1, d2, x) for x in MP.run(x0, 0, 8)]
        for _ in range(4):
            out.append((f"rnd({d1:g},{d2:g})", d1, d2, math.exp(rng.uniform(-6, 6)) * n / m))
    return out


def build():
    cells: dict = {}
    for i, (_, d1, d2, x) in enumerate(plan(), 1):
        cells.update(R3P.row_cells(i, d1, d2, x, "hold"))
    for i, args in enumerate(EDGES, 1):
        cells[f"D{i}"] = f"=F.DIST({args},FALSE)"
    note = ("F.DIST(x,d1,d2,FALSE) on 40 random shapes (d1, d2 in [1, 500], some non-integer) x 4 regions x 8 consecutive "
            "doubles + random x (A echoes x, B the density), and literal-argument edge cells in column D")
    return R3P.case_doc(cells, f"T-2586 r5-f-final integrator hold-out: {note}.", "T-2586")


if __name__ == "__main__":
    doc, path = build(), MP.CASES / f"{CASE}.case.json"
    print(len(doc["sheets"][0]["cells"]), "cells", "exists" if path.exists() else "new")
    if "--write" in sys.argv and not path.exists():
        path.write_text(json.dumps(doc, indent=1) + "\n")
