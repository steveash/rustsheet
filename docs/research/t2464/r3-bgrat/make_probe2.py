"""Oracle probe t2512-r3-bgrat-split (probes-numeric): Excel's bgrat increment alone at every L140 bup+bgrat call.

L140 bup+bgrat forms I_x(a, b) = RN(w_bup + bgrat(a, b0, x)) with b0 = b - int(b) in (0, 1). BETA.DIST(x, a, b0, TRUE)
dispatches (min(a, b0) <= 1, swapped, b0' = a > 15) to bratio's direct bgrat branch with w = 0 and returns that
same increment as the lower tail, rounded once (where the model says the direct cell takes bgrat; others are
skipped). Column A = BETA.DIST(x, a, b0, TRUE). Meta: [[row, a, b0, x, src_case, src_cell], ...]
usage: make_probe2.py [--write]"""
from __future__ import annotations

import importlib.util
import json
import pickle
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-tdist"))
import xlk  # noqa: E402

import branches  # noqa: E402

spec = importlib.util.spec_from_file_location("w2probes", HERE.parent / "w2-forensics" / "make_probes.py")
w2p = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w2p)
NAME = "t2512-r3-bgrat-split"

d = pickle.load(open(HERE / "calls.pkl", "rb"))
seen, cells, meta = set(), {}, []
for i, c in d["calls"].items():
    a, b0, x, y, w, lnx, g = c
    br = d["res"][i][2]
    if not br.startswith("L140 bup+bgrat") or b0 == 0.5 or b0 == 1.0 or (a, b0, x) in seen:
        continue
    seen.add((a, b0, x))
    if y != 1.0 - x:  # the direct cell's y must be the call's y
        continue
    _, _, br2 = xlk.bx.kernel(a, b0, x, 1.0 - x, branches.FINAL)
    if not br2.startswith("bgrat"):
        continue
    k = len(cells) + 1
    cells[f"A{k}"] = f"=BETA.DIST({w2p.num(x)},{w2p.num(a)},{w2p.num(b0)},TRUE)"
    r = d["rows"][i]
    meta.append((k, a, b0, x, r["case"], r["cell"]))
case = {"format": "rustsheet-case/v1",
        "description": "T-2512 r3-bgrat: BETA.DIST(x, a, frac(b)) at every L140 bup+bgrat cell (Excel's bgrat increment alone)",
        "tags": ["M9", "probe", "statistics", "T-2512", "incomplete-beta"],
        "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
p = w2p.CASES / f"{NAME}.case.json"
print(len(cells), "cells", p)
if "--write" in sys.argv:
    p.write_text(json.dumps(case, indent=1) + "\n")
    json.dump(meta, open(HERE / f"{NAME}.meta.json", "w"))
