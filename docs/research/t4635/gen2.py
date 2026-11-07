"""T-4635 probe 2: locate the cuts the first fill (probes/t4635-gamma-dist-huge-ratio) brackets.

Run: python3 -I docs/research/t4635/gen2.py <case.json>
Sheet Lo (cdf) and LoD (density): shapes near 1, x / beta from 1E260 to 1E306 at beta 1E-8 (the cut that y^a
overflowing predicts for shapes 1.01 and 1.1). Sheet Hi (cdf) and HiD (density): shapes 0.5 to 10, x / beta from 2E307
to 1.79E308 at beta 1E-8 (the second cut, between 1E307 and 1.5E308). Sheet Beta: the same ratios at beta 1E-100 and
1E-200, to tell a cut on x / beta from a cut on x.
"""

import json
import sys

COLS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
LO_SHAPES = ["1.01", "1.02", "1.05", "1.1", "1.15", "1.2", "1.22", "1.25", "1.3"]
LO_RATIOS = ["1E260", "1E270", "1E275", "1E280", "1E282", "1E285", "1E290", "1E295", "1E300", "1E302", "1E304",
             "1E305", "1E306"]
HI_SHAPES = ["0.5", "0.9", "0.99", "1", "1.25", "1.5", "2", "2.1", "2.15", "2.2", "2.25", "2.5", "2.6", "2.75",
             "2.9", "3", "4", "10"]
HI_RATIOS = ["2E307", "3E307", "5E307", "7E307", "9E307", "1E308", "1.1E308", "1.2E308", "1.3E308", "1.4E308",
             "1.5E308", "1.6E308", "1.7E308", "1.79E308"]


def x_of(ratio: str, beta_exp: int) -> str:
    """The literal x = ratio * 10^beta_exp, written exactly (mantissa kept, exponent shifted)."""
    mant, exp = ratio.split("E")
    return f"{mant}E{int(exp) + beta_exp}"


def grid(shapes: list[str], ratios: list[str], beta_exp: int, cumulative: bool) -> dict[str, str]:
    flag = "TRUE" if cumulative else "FALSE"
    return {
        f"{COLS[j]}{i}": f"=GAMMA.DIST({x_of(r, beta_exp)},{a},1E{beta_exp},{flag})"
        for i, a in enumerate(shapes, 1)
        for j, r in enumerate(ratios)
    }


beta_cells: dict[str, str] = {}
# Rows 1-3 at beta 1E-100, rows 4-6 at 1E-200: shapes 1.1, 1.5 and 2.25 across the cut ratios.
B_RATIOS = ["1E280", "1E290", "1E307", "1E308", "1.5E308"]
for k, beta_exp in enumerate([-100, -200]):
    for i, a in enumerate(["1.1", "1.5", "2.25"], 1 + 3 * k):
        for j, r in enumerate(B_RATIOS):
            beta_cells[f"{COLS[j]}{i}"] = f"=GAMMA.DIST({x_of(r, beta_exp)},{a},1E{beta_exp},TRUE)"
case = {
    "format": "rustsheet-case/v1",
    "description": (
        "GAMMA.DIST at a huge finite x / beta, second bracket (T-4635; plan in docs/research/t4635). Lo/LoD: shapes "
        "1.01 to 1.3 (rows) at x / beta 1E260 to 1E306 (columns), beta 1E-8, cdf and density. Hi/HiD: shapes 0.5 to "
        "10 at x / beta 2E307 to 1.79E308, beta 1E-8. Beta: shapes 1.1, 1.5, 2.25 at x / beta 1E280 to 1.5E308 with "
        "beta 1E-100 (rows 1-3) and 1E-200 (rows 4-6)."
    ),
    "tags": ["M9", "statistical", "probe"],
    "sheets": [
        {"name": "Lo", "cells": grid(LO_SHAPES, LO_RATIOS, -8, True)},
        {"name": "LoD", "cells": grid(LO_SHAPES, LO_RATIOS, -8, False)},
        {"name": "Hi", "cells": grid(HI_SHAPES, HI_RATIOS, -8, True)},
        {"name": "HiD", "cells": grid(HI_SHAPES, HI_RATIOS, -8, False)},
        {"name": "Beta", "cells": beta_cells},
    ],
    "expected": {},
}
with open(sys.argv[1], "w") as f:
    json.dump(case, f, indent=1)
