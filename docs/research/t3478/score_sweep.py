"""Score a T-3478 sweep: engine vs Excel (bit-exact, 4 ulp, 15 digits), vs 50-digit math (1e-9),
the Gate A envelope, and the correctly rounded candidate vs Excel. Per stratum (hypothesis).

Run: cd tools && uv run python -I ../docs/research/t3478/score_sweep.py <repo> <case-name> [--list]
"""

from __future__ import annotations

import collections
import json
import sys
from decimal import Decimal
from pathlib import Path

from rustsheet_tools.fitlab.score import CLOSE_RELATIVE, ulp_distance, within_fifteen_digits
from rustsheet_tools.snapshot import show_cells

REPO = Path(sys.argv[1])
NAME = sys.argv[2]
LIST = "--list" in sys.argv
CASES = REPO / "eval" / "suites" / "probes-numeric" / "cases"


def num(s: str) -> float | None:
    try:
        return float(s)
    except (TypeError, ValueError):
        return None


case = json.loads((CASES / f"{NAME}.case.json").read_text())
oracle = json.loads((CASES / f"{NAME}.oracle.json").read_text())["sheets"]["Calc"]
maths = {}
for line in (CASES / f"{NAME}.math.tsv").read_text().splitlines()[1:]:
    cell, m, _ = line.split("\t", 2)
    maths[cell] = m
shown = show_cells(f"probes-numeric/{NAME}.case.json")

rows: dict[str, collections.Counter] = collections.defaultdict(collections.Counter)
outside = []
for key, hyp in case["hypotheses"].items():
    stratum = hyp.split("/", 1)[1]
    c = rows[stratum]
    c["points"] += 1
    addr = key.split("!")[1]
    o = oracle.get(addr, {})
    excel = o.get("v") if o.get("t") == "n" else (o.get("v") or o.get("e") or "?")
    actual = shown[key]["actual"]
    a = num(actual)
    m = maths[key]
    mv = Decimal(m) if not m.startswith("#") else None
    ex = excel if isinstance(excel, float) else None
    # engine vs Excel
    if ex is not None and a is not None:
        c["excel number"] += 1
        c["bit-exact"] += a == ex
        c["<=4ulp"] += ulp_distance(a, ex) <= 4
        c["<=15d"] += within_fifteen_digits(ex, a)
    elif ex is None and a is None and str(excel) == actual:
        c["both error"] += 1
        c["bit-exact"] += 1
        c["<=4ulp"] += 1
        c["<=15d"] += 1
    elif ex is None and a is None:
        c["error kinds differ"] += 1
    elif ex is None:
        c["excel error, engine number"] += 1
    else:
        c["excel number, engine error"] += 1
    # vs math
    in_math = False
    if a is not None and mv is not None:
        rel = abs(Decimal(a) - mv) / max(abs(mv), Decimal("1e-300"))
        in_math = rel <= Decimal(repr(CLOSE_RELATIVE))
    c["<=1e-9 math"] += in_math
    same_error = ex is None and a is None and str(excel) == actual
    in_excel = (ex is not None and a is not None and within_fifteen_digits(ex, a)) or same_error
    # R2: an error against a number fails, and so does one error kind against another
    mismatch = (ex is None) != (a is None) or (ex is None and a is None and not same_error)
    inside = (in_excel or in_math) and not mismatch
    c["inside envelope"] += inside
    if not inside:
        outside.append((key, stratum, shown[key]["formula"], excel, actual, m[:22]))
    # correctly rounded candidate vs Excel
    if mv is not None and ex is not None:
        cr = float(mv)
        c["CR bit-exact"] += cr == ex
        c["CR <=4ulp"] += ulp_distance(cr, ex) <= 4
        c["CR <=15d"] += within_fifteen_digits(ex, cr)
    elif mv is None and ex is None:
        c["CR bit-exact"] += 1
        c["CR <=4ulp"] += 1
        c["CR <=15d"] += 1

cols = [
    "points",
    "bit-exact",
    "<=4ulp",
    "<=15d",
    "<=1e-9 math",
    "inside envelope",
    "CR bit-exact",
    "CR <=4ulp",
    "CR <=15d",
    "both error",
    "excel error, engine number",
    "excel number, engine error",
    "error kinds differ",
]
print("| Stratum | " + " | ".join(cols) + " |")
print("|---|" + "---|" * len(cols))
total: collections.Counter = collections.Counter()
for s, c in rows.items():
    total += c
    print(f"| {s} | " + " | ".join(str(c[k]) for k in cols) + " |")
print("| **all** | " + " | ".join(str(total[k]) for k in cols) + " |")
print(f"\noutside the envelope: {len(outside)}")
for o in outside[: (len(outside) if LIST else 15)]:
    print("  ", *o)
