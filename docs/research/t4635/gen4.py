"""T-4635 probe 4: rule R in CHISQ.DIST, CHISQ.DIST.RT and CHIDIST (scale 2, shape df / 2).

Run: python3 -I docs/research/t4635/gen4.py <case.json>
Only df 3 (shape 1.5) can reach the cut at a finite x: x / 2 > 1.5 * DBL_MAX / 4 is x > 1.348E308 (df 4 needs
x > DBL_MAX). x is computed (`1.3E300*1E8`), since Excel for the web refuses a typed literal from 1E307 up. Rule R
predicts: x 1.3E308 a number, 1.4E308 and 1.79E308 `#NUM!`, for df 3; numbers at df 2 and 4. Row 4 repeats rule R on
GAMMA.DIST at scale 1 with the same computed x (shape 1.5: cut 6.74E307).
"""

import json
import sys

XS = ["1.3E300*1E8", "1.4E300*1E8", "1.79E300*1E8"]
COLS = "ABCDEFGHIJKL"
cells: dict[str, str] = {}
for i, df in enumerate(["3", "2", "4"], 1):
    for j, x in enumerate(XS):
        cells[f"{COLS[j]}{i}"] = f"=CHISQ.DIST({x},{df},TRUE)"
        cells[f"{COLS[j + 3]}{i}"] = f"=CHISQ.DIST({x},{df},FALSE)"
        cells[f"{COLS[j + 6]}{i}"] = f"=CHISQ.DIST.RT({x},{df})"
        cells[f"{COLS[j + 9]}{i}"] = f"=CHIDIST({x},{df})"
for j, x in enumerate(["6.5E299*1E8", "7E299*1E8"]):
    cells[f"{COLS[j]}4"] = f"=GAMMA.DIST({x},1.5,1,TRUE)"
case = {
    "format": "rustsheet-case/v1",
    "description": (
        "Rule R of T-4635 (docs/research/t4635) in the chi-squared functions: rows df 3, 2, 4; columns x 1.3E308, "
        "1.4E308, 1.79E308 (computed) for CHISQ.DIST cdf (A-C), density (D-F), CHISQ.DIST.RT (G-I), CHIDIST (J-L). "
        "Row 4: GAMMA.DIST(x,1.5,1,TRUE) at x 6.5E307 and 7E307."
    ),
    "tags": ["M9", "statistical", "probe"],
    "sheets": [{"name": "Chi", "cells": cells}],
    "expected": {},
}
with open(sys.argv[1], "w") as f:
    json.dump(case, f, indent=1)
