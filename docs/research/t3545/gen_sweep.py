"""T-3545: a fresh sweep of IMSIN and IMCOS with a small imaginary part, drawn after the model was frozen
(decision 0188 R4.2): IMSIN and IMCOS take sinh y and cosh y as (u - 1/u)/2 and (u + 1/u)/2 with u = EXP(y) for
every |y| <= 20, the band's lower cut gone.

Deterministic (seed 3545). Writes eval/suites/probes-numeric/cases/t3545-imsin-imcos-small-imag-sweep.case.json.
150 rows: |y| log-uniform in [1e-7, 0.0105] with a random sign, 15 significant digits; x = 0 on even rows,
x uniform in (-1.5, 1.5) on odd rows. A: IMSIN(x+yi), B: IMCOS(x+yi).
"""

from __future__ import annotations

import json
import math
import random
from pathlib import Path

SEED = 3545
OUT = (
    Path(__file__).resolve().parents[3]
    / "eval/suites/probes-numeric/cases/t3545-imsin-imcos-small-imag-sweep.case.json"
)


def sig15(v: float) -> str:
    return f"{v:.15g}".replace("e", "E")


def main() -> None:
    rng = random.Random(SEED)
    cells: dict[str, str] = {}
    for row in range(1, 151):
        y = sig15(rng.choice((-1, 1)) * math.exp(rng.uniform(math.log(1e-7), math.log(0.0105))))
        x = "0" if row % 2 == 0 else sig15(rng.uniform(-1.5, 1.5))
        z = f"{x}{y if y.startswith('-') else '+' + y}i"
        cells[f"A{row}"] = f'=IMSIN("{z}")'
        cells[f"B{row}"] = f'=IMCOS("{z}")'
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-3545: fresh sweep after the freeze (decision 0188 R4.2) of IMSIN (A) and IMCOS (B) with |y| "
            "log-uniform in [1e-7, 0.0105], x = 0 on even rows and in (-1.5, 1.5) on odd rows. "
            "Generator: docs/research/t3545/gen_sweep.py (seed 3545)."
        ),
        "tags": ["M9", "engineering", "probe", "T-3545"],
        "expected": {},
        "sheets": [{"name": "S", "cells": cells}],
    }
    OUT.write_text(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
