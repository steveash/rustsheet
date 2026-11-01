"""T-3589: fresh sweeps drawn after the model was frozen (decision 0188 R4.2).

The frozen model: at |x| <= 0.01 IMSINH's sinh x is the ascending Taylor sum x + x^3/3! + x^5/5! + ...
(doubles, each term the previous times x^2/((2k)(2k+1)), until a term leaves the sum unchanged) and
cosh x the `1/u` form (u = EXP(-x)), the same parts for IMCOSH, IMSECH and IMCSCH; IMSIN and IMCOS
take sinh y and cosh y as (u - 1/u)/2 and (u + 1/u)/2 with u = EXP(y) for every |y|, the EXP pair
past |y| = 20 gone.

Deterministic. Writes two cases to eval/suites/probes-numeric/cases/:
- t3589-im-hyperbolic-small-x-sweep (seed 3589): 150 rows, |x| log-uniform in [1e-6, 0.01] with a random
  sign, 15 significant digits; y = 0 on even rows, |y| uniform in [0.1, 1.4] with a random sign on odd
  rows. A IMSINH, B IMCOSH, C IMSECH, D IMCSCH of x+yi.
- t3589-imsin-imcos-large-imag-sweep (seed 35891): 150 rows, |y| uniform in [20, 709.7] with a random
  sign; x = 0 on rows divisible by 4, else uniform in (-3.1, 3.1). A IMSIN, B IMCOS of x+yi.
"""

from __future__ import annotations

import json
import math
import random
from pathlib import Path

CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"


def sig15(v: float) -> str:
    return f"{v:.15g}".replace("e", "E")


def signed(rng: random.Random, magnitude: float) -> str:
    return sig15(rng.choice((-1, 1)) * magnitude)


def complex_text(x: str, y: str) -> str:
    if y == "0":
        return x
    return f"{x}{y if y.startswith('-') else '+' + y}i"


def write(name: str, cells: dict[str, str], description: str) -> None:
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M9", "engineering", "probe", "T-3589"],
        "expected": {},
        "sheets": [{"name": "S", "cells": cells}],
    }
    (CASES / f"{name}.case.json").write_text(json.dumps(case, indent=2) + "\n")


def small_x() -> None:
    rng = random.Random(3589)
    cells: dict[str, str] = {}
    for row in range(1, 151):
        x = signed(rng, math.exp(rng.uniform(math.log(1e-6), math.log(0.01))))
        y = "0" if row % 2 == 0 else signed(rng, rng.uniform(0.1, 1.4))
        z = complex_text(x, y)
        for col, fn in zip("ABCD", ("IMSINH", "IMCOSH", "IMSECH", "IMCSCH"), strict=True):
            cells[f"{col}{row}"] = f'={fn}("{z}")'
    write(
        "t3589-im-hyperbolic-small-x-sweep",
        cells,
        "T-3589: fresh sweep after the freeze (decision 0188 R4.2) of IMSINH (A), IMCOSH (B), IMSECH (C) and "
        "IMCSCH (D) with |x| log-uniform in [1e-6, 0.01], y = 0 on even rows and |y| in [0.1, 1.4] on odd rows. "
        "Generator: docs/research/t3589/gen_sweep.py (seed 3589).",
    )


def large_y() -> None:
    rng = random.Random(35891)
    cells: dict[str, str] = {}
    for row in range(1, 151):
        y = signed(rng, rng.uniform(20.0, 709.7))
        x = "0" if row % 4 == 0 else sig15(rng.uniform(-3.1, 3.1))
        z = complex_text(x, y)
        cells[f"A{row}"] = f'=IMSIN("{z}")'
        cells[f"B{row}"] = f'=IMCOS("{z}")'
    write(
        "t3589-imsin-imcos-large-imag-sweep",
        cells,
        "T-3589: fresh sweep after the freeze (decision 0188 R4.2) of IMSIN (A) and IMCOS (B) with |y| uniform "
        "in [20, 709.7] with a random sign, x = 0 on rows divisible by 4, else in (-3.1, 3.1). "
        "Generator: docs/research/t3589/gen_sweep.py (seed 35891).",
    )


if __name__ == "__main__":
    small_x()
    large_y()
