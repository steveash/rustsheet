"""T-3525: fresh log-uniform sweeps of IMSINH, IMCOSH, IMSIN and IMCOS over the domain of the
complex hyperbolic selectors (functions/engineering/complex/trig.rs `INVERSE_EXP_LOW`,
`INVERSE_EXP_HIGH`, `HYPERBOLIC_LOG_CUT`), decision 0188 R4.2. Deterministic (seeded).

First case (no flag, seed 3525): eval/suites/probes-numeric/cases/t3525-im-hyperbolic-fresh-sweep.case.json
- Rows 1..200 (band): |x| log-uniform in [1e-4, 709] (both 1/u cuts, 0.01 and 20, inside), a random
  sign, 15 significant digits. Even rows take y = 0 (the bare sinh and cosh), odd rows a y with
  |y| uniform in [0.1, 1.4] and a random sign (cos y and sin y well conditioned). A: IMSINH(x+yi),
  B: IMCOSH(x+yi), C: IMSIN(y+xi), D: IMCOS(y+xi) (the band also gates IMSIN and IMCOS's sinh and
  cosh of the imaginary part).
- Rows 201..300 (log cut): |x| log-uniform in [100, 800], a random sign, y uniform in (-3.1, 3.1),
  nonzero. A: IMSINH(x+yi), B: IMCOSH(x+yi).

Second case (`--log-band`, seed 35251): t3525-im-hyperbolic-log-band-sweep.case.json. The first case put
no point in 709 < |x| < 710.5, the only place the log form and the product form can differ (below 709
both are the product form; past about 710.48 both overflow), so 100 rows draw |x| uniform in
[708.9, 710.6] with a random sign and y uniform in (-3.1, 3.1), nonzero. A: IMSINH(x+yi), B: IMCOSH(x+yi).
"""

from __future__ import annotations

import json
import math
import random
import sys
from pathlib import Path

SEED = 3525
LOG_BAND_SEED = 35251
CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"
OUT = CASES / "t3525-im-hyperbolic-fresh-sweep.case.json"
LOG_BAND_OUT = CASES / "t3525-im-hyperbolic-log-band-sweep.case.json"


def sig15(v: float) -> str:
    return f"{v:.15g}".replace("e", "E")


def log_uniform(rng: random.Random, lo: float, hi: float) -> float:
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def complex_text(re: str, im: str | None) -> str:
    if im is None:
        return re
    return f"{re}{im if im.startswith('-') else '+' + im}i"


def nonzero_y(rng: random.Random) -> str:
    y = 0.0
    while y == 0.0:
        y = rng.uniform(-3.1, 3.1)
    return sig15(y)


def write_case(path: Path, description: str, cells: dict[str, str]) -> None:
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M9", "engineering", "probe", "T-3525"],
        "expected": {},
        "sheets": [{"name": "S", "cells": cells}],
    }
    path.write_text(json.dumps(case, indent=2) + "\n")


def fresh() -> None:
    rng = random.Random(SEED)
    cells: dict[str, str] = {}
    for row in range(1, 201):
        x = sig15(rng.choice((-1, 1)) * log_uniform(rng, 1e-4, 709.0))
        y = None if row % 2 == 0 else sig15(rng.choice((-1, 1)) * rng.uniform(0.1, 1.4))
        z = complex_text(x, y)
        w = complex_text(y if y is not None else "0", x)
        cells[f"A{row}"] = f'=IMSINH("{z}")'
        cells[f"B{row}"] = f'=IMCOSH("{z}")'
        cells[f"C{row}"] = f'=IMSIN("{w}")'
        cells[f"D{row}"] = f'=IMCOS("{w}")'
    for row in range(201, 301):
        x = sig15(rng.choice((-1, 1)) * log_uniform(rng, 100.0, 800.0))
        z = complex_text(x, nonzero_y(rng))
        cells[f"A{row}"] = f'=IMSINH("{z}")'
        cells[f"B{row}"] = f'=IMCOSH("{z}")'
    write_case(
        OUT,
        "T-3525: fresh log-uniform sweep (decision 0188 R4.2) over the complex hyperbolic selectors of "
        "complex/trig.rs. Rows 1-200: |x| in [1e-4, 709], y = 0 on even rows, |y| in [0.1, 1.4] on odd rows; "
        "A IMSINH(x+yi), B IMCOSH(x+yi), C IMSIN(y+xi), D IMCOS(y+xi). Rows 201-300: |x| in [100, 800], "
        "y in (-3.1, 3.1); A IMSINH, B IMCOSH. Generator: docs/research/t3525/gen_sweep.py (seed 3525).",
        cells,
    )


def log_band() -> None:
    rng = random.Random(LOG_BAND_SEED)
    cells: dict[str, str] = {}
    for row in range(1, 101):
        z = complex_text(sig15(rng.choice((-1, 1)) * rng.uniform(708.9, 710.6)), nonzero_y(rng))
        cells[f"A{row}"] = f'=IMSINH("{z}")'
        cells[f"B{row}"] = f'=IMCOSH("{z}")'
    write_case(
        LOG_BAND_OUT,
        "T-3525: fresh sweep of the log form's window (decision 0188 R4.2): |x| uniform in [708.9, 710.6], "
        "y in (-3.1, 3.1); A IMSINH(x+yi), B IMCOSH(x+yi). Generator: docs/research/t3525/gen_sweep.py "
        "--log-band (seed 35251).",
        cells,
    )


if __name__ == "__main__":
    if "--log-band" in sys.argv:
        log_band()
    else:
        fresh()
