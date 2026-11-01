"""T-3570 r5-bpser-lg: probe t3570-r5-bpser-lg (probes-numeric). Which lnGamma(a0 < 1) does Excel's bpser use in L?

Unswapped w_bpser cells in TOMS's 1 < b0 < 8 sub-branch: bpser(a0, b0, x) with a = a0 < 1 < b = b0 < 8, a0 stepped by
single ulps around each center, so lnGamma(a0) moves by about one ulp per step while every other piece is fixed.
Per a0: 8 short-decimal x where r2-bpser-small's model fires bpser (rows A..H); GAMMALN(a0) and GAMMALN(fl(a0 + b0))
(columns I, J), so ln B is built from Excel's own GAMMALN, not the engine model; GAMMALN(b0) once per b0 (column K).
Meta: [[row, a0, b0, [x...]], ...]; row 1 also holds K = GAMMALN(b0) per b0 group in K{row of the group's first a0}.
usage: mkprobe.py [--write]"""
from __future__ import annotations

import json
import math
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-bpser-small"))
import lib  # noqa: E402

NUM = lib.w2p.num
CENTERS = (0.15, 0.3, 0.4, 0.5, 0.6, 0.7, 0.85)
B0 = (1.61, 3.37, 5.83)
STEPS = range(-12, 12)
XS = (0.00031, 0.00057, 0.00083, 0.0013, 0.0027, 0.0041, 0.0063, 0.0089, 0.013, 0.019, 0.027, 0.037, 0.051, 0.071, 0.093, 0.12, 0.17, 0.23)


def step(v: float, k: int) -> float:
    d = math.inf if k > 0 else -math.inf
    for _ in range(abs(k)):
        v = math.nextafter(v, d)
    return v


def fires(a: float, b: float, x: float) -> bool:
    r = lib.kernel(a, b, x, 1.0 - x)
    return r[2] == "w_bpser" and bool(r[3]) and r[3][-1][1:3] == (a, b)


def build():
    cells, meta, row = {}, [], 0
    for b0 in B0:
        first = True
        for c in CENTERS:
            ok = [x for x in XS if fires(c, b0, x)]
            xs = ok[-8:]  # the largest x where bpser fires: more series terms, more L resolution
            assert len(xs) == 8, (c, b0, xs)
            for k in STEPS:
                a0 = step(c, k)
                row += 1
                for col, x in zip("ABCDEFGH", xs):
                    cells[f"{col}{row}"] = f"=BETA.DIST({NUM(x)},{NUM(a0)},{NUM(b0)},TRUE)"
                cells[f"I{row}"] = f"=GAMMALN({NUM(a0)})"
                cells[f"J{row}"] = f"=GAMMALN({NUM(a0 + b0)})"
                if first:
                    cells[f"K{row}"] = f"=GAMMALN({NUM(b0)})"
                    first = False
                meta.append([row, a0, b0, xs])
    return cells, meta


def main() -> None:
    lib.bx.fetch_missing()
    cells, meta = build()
    name = "t3570-r5-bpser-lg"
    print(name, len(meta), "a0 points", len(cells), "cells")
    assert len(cells) <= 16384
    if "--write" in sys.argv:
        case = {"format": "rustsheet-case/v1",
                "description": "T-3570 r5-bpser-lg: BETA.DIST cdf in bpser's 1 < b0 < 8 sub-branch with a0 stepped by "
                               "single ulps, plus GAMMALN at a0, a0 + b0 and b0 (docs/research/t2464/r5-bpser-lg)",
                "tags": ["M9", "probe", "statistics", "T-3570", "incomplete-beta"],
                "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
        (lib.CASES / f"{name}.case.json").write_text(json.dumps(case, indent=1) + "\n")
        json.dump(meta, open(HERE / f"{name}.meta.json", "w"))


if __name__ == "__main__":
    main()
