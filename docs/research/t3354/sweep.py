"""T-3354 fresh sweep: F.DIST and F.DIST.RT at 200 stratified points (decision 0188 R2, R4.2).

Writes eval/suites/probes-numeric/cases/t3354-f-dist-sweep.case.json. Deterministic (seed 3354).
Run from the repository root: python3 docs/research/t3354/sweep.py
"""

import json
import random
from pathlib import Path

SEED = 3354
OUT = Path("eval/suites/probes-numeric/cases/t3354-f-dist-sweep.case.json")


def log_uniform(rng: random.Random, lo: float, hi: float) -> float:
    return 10 ** rng.uniform(lo, hi)


def lit(v: float) -> str:
    """A literal of at most 15 significant digits."""
    return f"{v:.15G}"


def point(rng: random.Random, row: int) -> tuple[float, int, int]:
    if row <= 80:  # the reported region: x near 1E-12, d2 near 7.8E6
        return log_uniform(rng, -13, -11), int(log_uniform(rng, 0, 1.78)), int(log_uniform(rng, 6.7, 7.0))
    if row <= 140:  # the task's acceptance range
        return log_uniform(rng, -15, -6), int(log_uniform(rng, 0, 2)), int(log_uniform(rng, 5, 10))
    if row <= 180:  # extreme but valid: x far below or above, a huge d1
        x = log_uniform(rng, -300, -15) if rng.random() < 0.5 else log_uniform(rng, -6, 3)
        return x, int(log_uniform(rng, 0, 6)), int(log_uniform(rng, 5, 9.99))
    return log_uniform(rng, -20, 6), int(log_uniform(rng, 0, 6)), int(log_uniform(rng, 0, 9.99))  # log-uniform


def main() -> None:
    rng = random.Random(SEED)
    cells: dict[str, str] = {}
    for row in range(1, 201):
        x, d1, d2 = point(rng, row)
        args = f"{lit(x)},{d1},{d2}"
        cells[f"A{row}"] = f"=F.DIST({args},TRUE)"
        cells[f"B{row}"] = f"=F.DIST({args},FALSE)"
        cells[f"C{row}"] = f"=F.DIST.RT({args})"
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-3354 sweep: F.DIST(x,d1,d2,TRUE) in column A, F.DIST(x,d1,d2,FALSE) in B and F.DIST.RT(x,d1,d2) in C "
            "at 200 stratified points (rows 1-80 the reported region: x 1E-13 to 1E-11, d1 1 to 60, d2 5E6 to 1E7; "
            "rows 81-140 the acceptance range: x 1E-15 to 1E-6, d1 1 to 100, d2 1E5 to 1E10; rows 141-180 extreme: "
            "x 1E-300 to 1E-15 or 1E-6 to 1000, d1 up to 1E6; rows 181-200 log-uniform x 1E-20 to 1E6, d1 1 to 1E6, "
            "d2 1 to 1E10); seed 3354, docs/research/t3354/sweep.py; scored against 50-digit mpmath by "
            "docs/research/t3354/score.py (decision 0188 R2, R4.2)."
        ),
        "tags": ["probe", "numeric", "statistics", "T-3354"],
        "sheets": [{"name": "S", "cells": cells}],
        "expected": {},
    }
    OUT.write_text(json.dumps(case, indent=1) + "\n")


if __name__ == "__main__":
    main()
