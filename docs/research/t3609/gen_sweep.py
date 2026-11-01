"""T-3609: fresh sweep drawn after the small-real IMSECH and IMCSCH kernels were frozen (decision 0188 R4.2).

The frozen change (commit a44bbca9e): at 0 < |x| <= 0.01 a real IMSECH is 1 over libm's cosh x and a real IMCSCH
is 1 over the series sinh x (complex/trig.rs `imsech`, `imcsch`). Nothing was chosen or tuned on this sweep.

Deterministic. Writes eval/suites/probes-numeric/cases/t3609-imsech-imcsch-small-real-sweep (seed 3609): 200 rows,
|x| log-uniform in [1e-8, 0.01] with a random sign, 15 significant digits. A IMSECH, B IMCSCH, C IMCOSH, D IMSINH
of x (C and D guard the parts the kernels share).
"""

from __future__ import annotations

import json
import math
import random
from pathlib import Path

CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"


def main() -> None:
    rng = random.Random(3609)
    cells: dict[str, str] = {}
    for row in range(1, 201):
        x = f"{rng.choice((-1, 1)) * math.exp(rng.uniform(math.log(1e-8), math.log(0.01))):.15g}".replace("e", "E")
        for col, fn in zip("ABCD", ("IMSECH", "IMCSCH", "IMCOSH", "IMSINH"), strict=True):
            cells[f"{col}{row}"] = f'={fn}("{x}")'
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-3609 (decision 0188 R4.2): fresh sweep for the small-real IMSECH (1/libm cosh) and IMCSCH (1/series "
            "sinh) kernels, drawn after commit a44bbca9e froze them and never used to choose them. 200 rows, |x| "
            "log-uniform in [1E-8, 0.01] with a random sign: A IMSECH, B IMCSCH, C IMCOSH, D IMSINH of x. "
            "Generator docs/research/t3609/gen_sweep.py, seed 3609."
        ),
        "tags": ["M9", "engineering", "probe", "T-3609"],
        "expected": {},
        "sheets": [{"name": "S", "cells": cells}],
    }
    (CASES / "t3609-imsech-imcsch-small-real-sweep.case.json").write_text(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
