"""Oracle probe t2464-r2-bratio-x29: is bratio's min(a,b) <= 1 switch to w1 = bpser(b0, a0, y0) at x0 >= 0.29 (R,
PR#13786) or at x0 >= 0.3 (TOMS 1992)? BETA.DIST(x, a, b, TRUE) with a < 1 < b at x = 0.2900 .. 0.3009 (step 1e-4,
short decimal literals), shapes a in {0.25, 0.5, 0.75, 0.9}, b in {2.5, 5, 7.5, 12, 14.5, 20, 50}: under the 0.29 rule
the cells in [0.29, 0.3) take w1_bpser (modelled well), under the 0.3 rule bup+bgrat or bgrat.
usage: make_probe_x29.py [--write]"""
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("w2probes", HERE.parent / "w2-forensics" / "make_probes.py")
w2p = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w2p)

NAME = "t2464-r2-bratio-x29"
cells, meta = {}, []
i = 0
for a in (0.25, 0.5, 0.75, 0.9):
    for b in (2.5, 5.0, 7.5, 12.0, 14.5, 20.0, 50.0):
        for k in range(110):
            x = f"{0.29 + k * 1e-4:.4f}"
            i += 1
            cells[f"A{i}"] = f"=BETA.DIST({x},{a!r},{b!r},TRUE)"
            meta.append((i, float(x), a, b))
case = {"format": "rustsheet-case/v1",
        "description": "T-2464 r2-bratio: BETA.DIST cdf at x in [0.29, 0.301), a < 1 < b, to place bratio's x0 >= 0.29 / 0.3 switch",
        "tags": ["M9", "probe", "statistics", "T-2464", "incomplete-beta"],
        "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
p = w2p.CASES / f"{NAME}.case.json"
print(len(cells), "cells", p)
if "--write" in sys.argv:
    p.write_text(json.dumps(case, indent=1) + "\n")
    json.dump(meta, open(HERE / f"{NAME}.meta.json", "w"))
