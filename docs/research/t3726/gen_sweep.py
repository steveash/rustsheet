"""T-3726: fresh sweep drawn after the model was frozen (decision 0188 R4.2).

The frozen model: IMSIN and IMCOS of a complex x+yi (y != 0) take sin x and cos x from SIN and COS's
own kernel (`excel_sin_cos`: x reduced by the x87 64-bit pi) below 2^27, instead of libm.

Deterministic (seed 3726). Writes eval/suites/probes-numeric/cases/t3726-imsin-imcos-near-half-pi-sweep:
200 rows, x = k*pi/2 + d printed to 15 significant digits, k uniform in {-4..-1, 1..4}; d = 0 on rows
divisible by 3 (x is then the 15-digit neighbour of k*pi/2), else |d| log-uniform in [1e-14, 1e-3] with a
random sign; |y| log-uniform in [1, 709] with a random sign. A IMSIN, B IMCOS of x+yi.
"""

from __future__ import annotations

import json
import math
import random
from pathlib import Path

CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"


def sig15(v: float) -> str:
    return f"{v:.15g}".replace("e", "E")


def complex_text(x: str, y: str) -> str:
    return f"{x}{y if y.startswith('-') else '+' + y}i"


def main() -> None:
    rng = random.Random(3726)
    cells: dict[str, str] = {}
    for row in range(1, 201):
        k = rng.choice((-4, -3, -2, -1, 1, 2, 3, 4))
        d = 0.0 if row % 3 == 0 else rng.choice((-1, 1)) * math.exp(rng.uniform(math.log(1e-14), math.log(1e-3)))
        x = sig15(k * math.pi / 2 + d)
        y = sig15(rng.choice((-1, 1)) * math.exp(rng.uniform(0.0, math.log(709.0))))
        z = complex_text(x, y)
        cells[f"A{row}"] = f'=IMSIN("{z}")'
        cells[f"B{row}"] = f'=IMCOS("{z}")'
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-3726: fresh sweep after the freeze (decision 0188 R4.2) of IMSIN (A) and IMCOS (B) with x "
        "near k*pi/2 (k = -4..4, k != 0; offset 0 or |d| log-uniform in [1e-14, 1e-3]) and |y| log-uniform in "
        "[1, 709] with a random sign. Generator: docs/research/t3726/gen_sweep.py (seed 3726).",
        "tags": ["M9", "engineering", "probe", "T-3726"],
        "expected": {},
        "sheets": [{"name": "S", "cells": cells}],
    }
    (CASES / "t3726-imsin-imcos-near-half-pi-sweep.case.json").write_text(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
