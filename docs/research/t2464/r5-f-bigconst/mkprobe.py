"""T-2586 r5-f-bigconst probes (probes-numeric, excel-graph): F.DIST(x, d1, d2, FALSE) at large shapes
(min(d) >= 16) where the x terms are known or tiny, to pin Excel's shape constant per (shape, side).
Per shape: 9 consecutive doubles around x = 1 (P = fl(c LN x) tiny) and, for d1 != d2, 9 consecutive doubles around
fl(d2/d1) (the side switch mx = n, r ~ 1, both sides); d1 = d2 gets 18 doubles around 1 (both sides).
Each row: A = x as an exact formula (w2-forensics' encoder), B = F.DIST(A, d1, d2, FALSE) (r3-fdensity row_cells).
  a     in-sample: every pair of GA (19 values, 361 shapes)
  hold  hold-out, built and filled together with a, read once after freezing: every pair of GH (12 values)
`rows(name)` regenerates the x values and reads Excel's from the sidecar, asserting each echo.
Usage: mkprobe.py [--write]"""

from __future__ import annotations

import json
import sys

from blib import MP, R3P, F  # noqa: F401  (blib imports this module lazily for rows(); no cycle at import time)

PREFIX = "t2586-r5-f-bigconst"
GA = (16, 17, 20, 24, 31, 40, 50, 63, 64, 80, 100, 127, 128, 160, 200, 256, 300, 400, 1000)
GH = (18, 26, 33, 36, 48, 57, 70, 96, 140, 220, 330, 600)
SPEC = {"a": GA, "hold": GH}


def plan(name):
    """[(run label, d1, d2, x)] in sheet-row order."""
    out = []
    for d1 in SPEC[name]:
        for d2 in SPEC[name]:
            if d1 == d2:
                out += [(f"x1({d1},{d2})", float(d1), float(d2), x) for x in MP.run(1.0, -9, 18)]
                continue
            out += [(f"x1({d1},{d2})", float(d1), float(d2), x) for x in MP.run(1.0, -4, 9)]
            out += [(f"sw({d1},{d2})", float(d1), float(d2), x) for x in MP.run(d2 / d1, -4, 9)]
    return out


def build(name):
    cells: dict = {}
    for i, (_, d1, d2, x) in enumerate(plan(name), 1):
        cells.update(R3P.row_cells(i, d1, d2, x, "hold"))  # kind "hold": columns A, B only
    note = "F.DIST(x,d1,d2,FALSE) at min(d) >= 16 on consecutive exact doubles around x = 1 and x = d2/d1"
    return R3P.case_doc(cells, f"T-2586 r5-f-bigconst probe {name}: {note} (column A echoes x).", "T-2586")


def rows(name):
    sh = F.sidecar(f"{PREFIX}-{name}")
    got = [(blk, d1, d2, x, F.val(sh, f"A{i}"), F.val(sh, f"B{i}")) for i, (blk, d1, d2, x) in enumerate(plan(name), 1)]
    assert all(echo == x for _, _, _, x, echo, v in got if v is not None), name
    return [dict(src=name, blk=b, x=x, d1=d1, d2=d2, excel=v) for b, d1, d2, x, _, v in got if v is not None and v > 0]


if __name__ == "__main__":
    for nm in SPEC:
        doc, path = build(nm), MP.CASES / f"{PREFIX}-{nm}.case.json"
        same = path.exists() and json.loads(path.read_text()) == doc
        state = ("exists" + (" (identical)" if same else " (DIFFERS)")) if path.exists() else "new"
        print(nm, len(doc["sheets"][0]["cells"]), "cells", state)
        if "--write" in sys.argv and not path.exists():
            path.write_text(json.dumps(doc, indent=1) + "\n")
