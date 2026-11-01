"""T-3568 r5-bup: build probes t3568-r5-bup-terms-tune and -terms-hold: Excel's density at every bup shape.

For BETA.DIST points of t3568-r5-bup-tune (resp. -hold) with D by bgrat and P > 0, row k holds
BETA.DIST(y0, c + j, a0, FALSE) for j = 0..n-1 in columns A, B, ... (j = 0 repeats probe column C as a
repeatability check). If Excel's L140 sums one density per shape, W = sum_j P_j fl(x0 y0)/(c + j).
Meta: [[row, source row, a0, b0, x0, y0, n, c], ...]. usage: mkterms.py [--write]"""
from __future__ import annotations

import json
import random
import sys

import score as s
from common import CASES, w2p

COLS = [chr(ord("A") + i) for i in range(26)] + [f"A{chr(ord('A') + i)}" for i in range(13)]


def build(half: str, count: int, seed: int):
    rows = [r for r in s.load(half) if r["P"] != 0 and r["region"] == "beta" and r["dbr"] == "bgrat"]
    random.Random(seed).shuffle(rows)
    cells, meta = {}, []
    for r in rows[:count]:
        k = len(meta) + 1
        for j in range(r["n"]):
            cells[f"{COLS[j]}{k}"] = f"=BETA.DIST({w2p.num(r['y0'])},{w2p.num(r['c'] + j)},{w2p.num(r['a0'])},FALSE)"
        meta.append([k, r["k"], r["a0"], r["b0"], r["x0"], r["y0"], r["n"], r["c"]])
    return cells, meta


for half, seed in (("tune", 1), ("hold", 2)):
    name = f"t3568-r5-bup-terms-{half}"
    cells, meta = build(half, 300, 35683 + seed)
    print(name, len(meta), "points", len(cells), "cells")
    if "--write" in sys.argv:
        assert len(cells) <= 16384
        case = {"format": "rustsheet-case/v1",
                "description": f"T-3568 r5-bup: Excel's beta density at every bup shape c + j of {len(meta)} "
                               f"t3568-r5-bup-{half} points (docs/research/t2464/r5-bup)",
                "tags": ["M9", "probe", "statistics", "T-3568", "incomplete-beta"],
                "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
        (CASES / f"{name}.case.json").write_text(json.dumps(case, indent=1) + "\n")
        json.dump(meta, open(s.HERE / f"{name}.meta.json", "w"))
