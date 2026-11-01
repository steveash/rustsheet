"""T-3382 fresh sweep: BETA.DIST's cdf on bratio's L140 bup+bgrat branch with one bup term.

`t3382-l140-sweep` (seed 33820) was generated and filled after T-3382's composition was frozen
(decision 0188 R4): W = fl(x87(P·fl(x0·y0)/b0')) plus an accurate bgrat increment. Every point lands
on that branch by bratio's own dispatch (min(a, b) > 1, lambda >= 0 after the swap, b0 < 40,
x0 > 0.7, a0 > 15) with 1 < b0 < 2: a0 log-uniform in (15, 1E4], b0 uniform in (1, 2), and
z = -(a0 + (b0 - 1)/2) ln x0 log-uniform in [0.5, 700] (bgrat's useful range, both ends past the
engine's [1, 650] window included), half the points written in the swapped orientation.

    python3 docs/research/t3382/gen_sweep.py   # rewrites the case.json file
"""

from __future__ import annotations

import json
import math
import random
from pathlib import Path

CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"
COUNT = 300


def lit(v: float) -> str:
    """A formula literal: 15 significant digits, as Excel keeps them."""
    return f"{v:.15g}"


def on_branch(a: float, b: float, x: float) -> bool:
    """bratio's dispatch lands on L140 bup+bgrat with one bup term (bratio/mod.rs `dispatch`)."""
    y = 1.0 - x
    if min(a, b) <= 1.0 or not 0.0 < x < 1.0:
        return False
    lam = (a + b) * y - b if a > b else a - (a + b) * x
    a0, b0, x0 = (b, a, y) if lam < 0 else (a, b, x)
    return 1.0 < b0 < 2.0 and b0 * x0 > 0.7 and x0 > 0.7 and a0 > 15.0


def point(rng: random.Random) -> str | None:
    a0 = math.exp(rng.uniform(math.log(15.0), math.log(1e4)))
    a0 = round(a0, 3) if rng.random() < 0.5 else float(round(a0))
    b0 = rng.uniform(1.0, 2.0)
    b0 = round(b0, 3) if rng.random() < 0.5 else float(lit(b0))
    if a0 <= 15.0 or not 1.0 < b0 < 2.0:
        return None
    nu = a0 + (b0 - 1.0) / 2.0
    z = math.exp(rng.uniform(math.log(0.5), math.log(700.0)))
    x0 = float(lit(math.exp(-z / nu)))
    if rng.random() < 0.5:
        a, b, x = a0, b0, x0
    else:
        a, b, x = b0, a0, float(lit(1.0 - x0))
    if not on_branch(a, b, x):
        return None
    return f"=BETA.DIST({lit(x)},{lit(a)},{lit(b)},TRUE)"


def main() -> None:
    rng = random.Random(33820)
    formulas: list[str] = []
    while len(formulas) < COUNT:
        f = point(rng)
        if f is not None and f not in formulas:
            formulas.append(f)
    cells = {f"A{i + 1}": f for i, f in enumerate(formulas)}
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-3382 fresh sweep after the freeze (seed 33820): BETA.DIST's cdf on TOMS 708 bratio's "
            "L140 bup+bgrat branch with one bup term (a0 in (15, 1E4], 1 < b0 < 2, x0 > 0.7, z in "
            "[0.5, 700]), half in the swapped orientation. Generator docs/research/t3382/gen_sweep.py."
        ),
        "tags": ["probe", "numeric", "statistics", "T-3382"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "check": [f"Calc!{k}" for k in cells],
        "expected": {},
    }
    (CASES / "t3382-l140-sweep.case.json").write_text(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
