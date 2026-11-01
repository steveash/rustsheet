"""T-3568 r5-bup: today's engine on the t3568-r5-bup-* probes, from a `make eval` run (eval/runs/latest/report.json).

Counts per case and column the cells the engine has visibly off (outside half a unit of the 15th digit of Excel's).
usage: engcols.py   (after `make eval`)"""
from __future__ import annotations

import json
import re
from collections import Counter

import score as s
from common import sl

rep = json.load(open(s.HERE.parents[3] / "eval" / "runs" / "latest" / "report.json"))
off, seen = Counter(), Counter()


def walk(o) -> None:
    if isinstance(o, dict):
        case = o.get("case", "")
        if "cell" in o and "t3568-r5-bup-" in case and "terms" not in case and o.get("cell", "").startswith("Calc!"):
            name = case.split("/")[1].split(".")[0]
            col = re.match(r"Calc!([A-Z]+)", o["cell"]).group(1)
            try:
                e, a = float(o["expected"]), float(o["actual"])
                vis = not sl.w15(a, e)
            except (TypeError, ValueError):
                vis = True
            seen[(name, col)] += 1
            off[(name, col)] += vis
        for v in o.values():
            walk(v)
    elif isinstance(o, list):
        for v in o:
            walk(v)


walk(rep)
for k in sorted(seen):
    print(f"{k[0]} column {k[1]}: {seen[k]} listed failures, {off[k]} visibly off")
