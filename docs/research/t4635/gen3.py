"""T-4635 probe 3: the fresh sweep after rule R froze (README, "Rule R"), and the low regime's shape cut.

Run: python3 -I docs/research/t4635/gen3.py <case.json>
Sheet Fresh: shapes drawn log-uniform from 1.15 to 4.5 (seeded), each at x / beta 0.97 and 1.03 times the predicted
cut a * DBL_MAX / 4 (columns A, B; cdf) and the same in the density (C, D); shapes 0.3 and 0.7 at 1.79E308 at the end.
Sheet Cut: shapes 1.11 to 1.14 (rows) at x / beta 1E285, 1E295, 1E300, 1E306 (cdf).
"""

import json
import math
import random
import sys

DBL_MAX = 1.7976931348623157e308
rng = random.Random(4635)
shapes = sorted(round(math.exp(rng.uniform(math.log(1.15), math.log(4.5))), 4) for _ in range(24))
fresh: dict[str, str] = {}
row = 0
for a in shapes:
    cut = DBL_MAX / 4 * a
    ratios = [min(f * cut, 1.79e308) for f in (0.97, 1.03)]
    if ratios[0] >= 1.79e308:
        continue
    row += 1
    for j, r in enumerate(ratios):
        x = f"{r * 1e-8:.4E}"
        fresh[f"{'AB'[j]}{row}"] = f"=GAMMA.DIST({x},{a},1E-8,TRUE)"
        fresh[f"{'CD'[j]}{row}"] = f"=GAMMA.DIST({x},{a},1E-8,FALSE)"
for a in ("0.3", "0.7"):
    row += 1
    fresh[f"A{row}"] = f"=GAMMA.DIST(1.79E300,{a},1E-8,TRUE)"
    fresh[f"C{row}"] = f"=GAMMA.DIST(1.79E300,{a},1E-8,FALSE)"
cut_cells = {
    f"{'ABCD'[j]}{i}": f"=GAMMA.DIST({x},{a},1E-8,TRUE)"
    for i, a in enumerate(["1.11", "1.12", "1.13", "1.14"], 1)
    for j, x in enumerate(["1E277", "1E287", "1E292", "1E298"])
}
case = {
    "format": "rustsheet-case/v1",
    "description": (
        "GAMMA.DIST at a huge finite x / beta, fresh sweep after the rule froze (T-4635; docs/research/t4635 rule R). "
        "Fresh: seeded log-uniform shapes 1.15 to 4.5, x / beta 0.97 and 1.03 times a * DBL_MAX / 4 at beta 1E-8 "
        "(A, B cdf; C, D density), then shapes 0.3 and 0.7 at 1.79E308. Cut: shapes 1.11 to 1.14 at x / beta 1E285 "
        "to 1E306."
    ),
    "tags": ["M9", "statistical", "probe"],
    "sheets": [{"name": "Fresh", "cells": fresh}, {"name": "Cut", "cells": cut_cells}],
    "expected": {},
}
with open(sys.argv[1], "w") as f:
    json.dump(case, f, indent=1)
