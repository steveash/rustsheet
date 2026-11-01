"""T-3570 r5-bpser-lg: probe t3570-r5-bpser-lg-aux, Excel's values for candidate lnGamma(a0) building blocks at every
a0 of t3570-r5-bpser-lg (same rows): A = GAMMALN(fl(1 + a0)), B = GAMMA(a0), C = LN(GAMMA(a0)),
D = GAMMA(fl(1 + a0)), E = LN(GAMMA(fl(1 + a0))). Built after pin.py/lgwin.py showed neither GAMMALN(a0) nor
fl(gamln1(a0) - LN(a0)) wins at every a0; it measures building blocks only, nothing is fitted to it.
usage: mkaux.py [--write]"""
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-bpser-small"))
import lib  # noqa: E402

NUM = lib.w2p.num
NAME = "t3570-r5-bpser-lg-aux"
meta = json.load(open(HERE / "t3570-r5-bpser-lg.meta.json"))
cells = {}
for row, a0, b0, xs in meta:
    a1 = a0 + 1.0
    cells[f"A{row}"] = f"=GAMMALN({NUM(a1)})"
    cells[f"B{row}"] = f"=GAMMA({NUM(a0)})"
    cells[f"C{row}"] = f"=LN(GAMMA({NUM(a0)}))"
    cells[f"D{row}"] = f"=GAMMA({NUM(a1)})"
    cells[f"E{row}"] = f"=LN(GAMMA({NUM(a1)}))"
print(NAME, len(cells), "cells")
if "--write" in sys.argv:
    case = {"format": "rustsheet-case/v1",
            "description": "T-3570 r5-bpser-lg: GAMMALN, GAMMA and LN(GAMMA) at the a0 and 1 + a0 of t3570-r5-bpser-lg "
                           "(docs/research/t2464/r5-bpser-lg)",
            "tags": ["M9", "probe", "statistics", "T-3570", "gamma"],
            "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
    (lib.CASES / f"{NAME}.case.json").write_text(json.dumps(case, indent=1) + "\n")
