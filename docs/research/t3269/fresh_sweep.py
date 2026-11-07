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


def write_case(
    path: Path, description: str, cells: dict[str, str], hypotheses: dict[str, str], task: str = "T-3269"
) -> None:
    """Write a one-sheet (Calc) probes-numeric case; fresh_sweep_2.py and t3469/fresh_sweep_3.py share it."""
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M9", "probe", "statistics", task, "poisson", "chisq", "sweep"],
        "expected": {},
        "hypotheses": hypotheses,
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    path.write_text(json.dumps(case, indent=1) + "\n", encoding="utf-8")


def shape_sweep(
    rng: random.Random, plan: list[tuple[float, float, float, float]], tag: str
) -> tuple[dict[str, str], dict[str, str]]:
    """Cells A1.. of Q(a, x) for (a_lo, a_hi, x_lo, x_hi) in `plan`, shared by fresh_sweep_2.py and
    t3469/fresh_sweep_3.py: a log-uniform rounded to a multiple of 1/2 and clamped to [a_lo, a_hi], x
    log-uniform with 6 significant digits; integer shapes alternate POISSON(a - 1, x, TRUE) and
    CHISQ.DIST.RT(2x, 2a), half-integer shapes are CHISQ.DIST.RT."""

    def log_uniform(lo: float, hi: float) -> float:
        return math.exp(rng.uniform(math.log(lo), math.log(hi)))

    cells, hypotheses = {}, {}
    for n, (a_lo, a_hi, x_lo, x_hi) in enumerate(plan, 1):
        a = min(max(round(2.0 * log_uniform(a_lo, a_hi)) / 2.0, a_lo), a_hi)
        x = float(f"{log_uniform(x_lo, x_hi):.6g}")
        if a.is_integer() and n % 2:
            formula, label = f"=POISSON({int(a) - 1},{x:g},TRUE)", f"{tag}/poisson/a={a:g}/x={x:g}"
        else:
            formula, label = f"=CHISQ.DIST.RT({2 * x:g},{int(2 * a)})", f"{tag}/chisq/a={a:g}/x={x:g}"
        cells[f"A{n}"] = formula
        hypotheses[f"Calc!A{n}"] = label
    return cells, hypotheses


def main() -> None:
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
    write_case(
        CASE,
        "T-3269 fresh log-uniform sweep after the far-tail model was frozen (decision 0188 R4.2): "
        "POISSON(k,x,TRUE), k 1 to 21, and CHISQ.DIST.RT(2x,df), odd df 3 to 41, x log-uniform from 1 to 1000 "
        "(240 cells; docs/research/t3269/fresh_sweep.py).",
        cells,
        hypotheses,
    )
    print(len(cells))


if __name__ == "__main__":
    main()
