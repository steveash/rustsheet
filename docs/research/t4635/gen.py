"""T-4635 probe: GAMMA.DIST at a huge finite x / beta, bracketing the #NUM! region T-4625 found.

Run: python3 -I docs/research/t4635/gen.py <case.json>
Hypothesis committed before the fill (README): #NUM! exactly for shapes 1 < a < 2.5 at x / beta 1E308, in both
forms; numbers (1 for the cdf, 0 for the density) elsewhere, and numbers for every shape at x / beta 1E200.
"""

import json
import sys

SHAPES = ["1", "1.01", "1.1", "1.25", "1.4", "1.5", "1.6", "1.75", "1.9", "1.99", "2", "2.01", "2.1", "2.25", "2.4",
          "2.45", "2.49", "2.5", "2.51"]
# x / beta = 1E-8 x; x from 1E208 (ratio 1E200) to 1.7E300 (ratio 1.7E308).
XS = ["1E208", "1E250", "1E290", "1E298", "1E299", "1E300", "1.5E300", "1.7E300"]
cols = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
cdf: dict[str, str] = {}
pdf: dict[str, str] = {}
for i, a in enumerate(SHAPES, 1):
    for j, x in enumerate(XS):
        cdf[f"{cols[j]}{i}"] = f"=GAMMA.DIST({x},{a},1E-8,TRUE)"
        pdf[f"{cols[j]}{i}"] = f"=GAMMA.DIST({x},{a},1E-8,FALSE)"
chi = {}
for i, df in enumerate(["2", "3", "4", "5"], 1):
    for j, x in enumerate(["1E200", "1E290"]):
        chi[f"{cols[j]}{i}"] = f"=CHISQ.DIST({x},{df},TRUE)"
        chi[f"{cols[j + 2]}{i}"] = f"=CHIDIST({x},{df})"
case = {
    "format": "rustsheet-case/v1",
    "description": (
        "GAMMA.DIST at a huge finite x / beta (T-4625 follow-up, T-4635), predictions in docs/research/t4635. "
        "Cdf and Pdf: row r is a shape from 1 to 2.51, column c an x from 1E208 to 1.7E300 at beta 1E-8 (x / beta "
        "1E200 to 1.7E308). Chi: CHISQ.DIST and CHIDIST at df 2 to 5, x 1E200 to 1.7E308."
    ),
    "tags": ["M9", "statistical", "probe"],
    "sheets": [{"name": "Cdf", "cells": cdf}, {"name": "Pdf", "cells": pdf}, {"name": "Chi", "cells": chi}],
    "expected": {},
}
json.dump(case, open(sys.argv[1], "w"), indent=1)
