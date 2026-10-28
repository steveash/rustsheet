"""T-2681: dump the F.TEST sweep (the `f-test` fit's 291 cells) with today's engine value to JSON.

Run from tools/: `uv run python ../docs/research/t2681/dump_cells.py OUT.json` (the engine value
is `rustsheet-eval show --all` of a release build; rebuild it first to score a change).
"""

import json
import sys

from rustsheet_tools.fitlab.fit import engine_candidate
from rustsheet_tools.fitlab.kernels import ftest as F
from rustsheet_tools.fitlab.sweep import load_sweep

sweep = load_sweep(F._cases(), F._PATTERN, F.in_fit, False)
engine = engine_candidate(sorted({c.case for c in sweep.cells}))
out = []
for cell in sweep.cells:
    x, y, a, b = F._shape(cell)
    out.append(
        {"case": cell.case, "key": cell.key, "x": x, "y": y, "a": a, "b": b, "excel": cell.expected,
         "engine": engine.evaluate(cell), "model": F.variant()(*cell.args)}
    )
json.dump(out, open(sys.argv[1], "w"))
print(len(out), "cells")
