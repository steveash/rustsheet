"""T-3383 r4-algdiv: probe t3383-r4-algdiv-gl, Excel's GAMMALN at b and a + b for every shape of probes a and hold
(A = exact literal of the argument, read back; B = GAMMALN(A)). `gl_values()` returns {arg: Excel GAMMALN}.
Usage: mkgl.py   (writes the case and probe_gl.args.json)"""

import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r3-m1-residual"))
import m1lib as M  # noqa: E402
import mkalgprobe as P  # noqa: E402

CASE = "t3383-r4-algdiv-gl"


def args():
    out = set()
    for name in P.SPECS:
        for _, a, b in P.rows_of(name):
            out |= {a, b, a + b}
    return sorted(out)


def gl_values():
    o = M.read_sidecar(CASE)
    got = {}
    for i, v in enumerate(json.loads((HERE / "probe_gl.args.json").read_text())):
        if o.get(f"A{i + 1}") == v and f"B{i + 1}" in o:
            got[v] = o[f"B{i + 1}"]
    return got


if __name__ == "__main__":
    vs = args()
    cells = {}
    for i, v in enumerate(vs):
        cells[f"A{i + 1}"] = "=" + M.lit(v)
        cells[f"B{i + 1}"] = f"=GAMMALN(A{i + 1})"
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-3383 r4-algdiv: Excel's GAMMALN at a, b and a + b of the r4-algdiv probe shapes (exact "
        "literals read back in A)",
        "tags": ["M9", "probe", "statistics", "T-3383", "incomplete-beta"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    (M.SUITE / f"{CASE}.case.json").write_text(json.dumps(case, indent=1) + "\n")
    (HERE / "probe_gl.args.json").write_text(json.dumps(vs) + "\n")
    print(CASE, len(vs), "arguments")
