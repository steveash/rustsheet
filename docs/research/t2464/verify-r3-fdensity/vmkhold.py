"""Build the verifier's own hold-out t2513-verify-r3-fdensity-hold (probes-numeric, 1,920 cells): F.DIST(x,d1,d2,FALSE)
at 12 shapes never probed by r3-fdensity's runs, in four r bands (mx/n ~ 0.22 and ~0.6, n/mx ~ 0.5 and ~0.2), i.e.
both sides of the claimed 0.375 log1p cut on both sides of the claimed mx = n swap.  20 consecutive doubles per run
from a seeded random start.  Column A holds x as an exact encoder formula (hi*2^a+lo*2^b, each < 2^27), B the density.
usage: vmkhold.py   (writes the case only if it does not exist)"""

from __future__ import annotations

import json
import math
import random

import vlib as V

CID = "t2513-verify-r3-fdensity-hold"
SHAPES = [(2, 13), (1, 30), (5, 14), (9, 9), (11, 4), (14, 33), (18, 7), (22, 22), (35, 90), (45, 15), (120, 60),
          (10, 150)]
BANDS = (0.22, 0.6, 2.0, 5.0)  # target mx/n
PER = 20


def enc(t: float) -> str:
    m, e = math.frexp(t)
    mi = int(m * 2**53)
    ex = e - 53
    hi, lo = mi >> 27, mi & (2**27 - 1)
    f = f"=({hi}*2^{ex + 27}+{lo}*2^{ex})"
    assert V.enc_value(f) == t
    return f


def main():
    rng = random.Random(2513_1024)
    cells, r = {}, 0
    for d1, d2 in SHAPES:
        for t in BANDS:
            x = t * d2 / d1 * (1 + rng.uniform(-0.03, 0.03))
            for _ in range(PER):
                r += 1
                cells[f"A{r}"] = enc(x)
                cells[f"B{r}"] = f"=F.DIST(A{r},{d1},{d2},FALSE)"
                x = math.nextafter(x, math.inf)
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-2513 verify-r3-fdensity hold-out: F.DIST(x,d1,d2,FALSE) on 12 new shapes x 4 r bands x 20 "
        "consecutive doubles (A = exact x, B = density).",
        "tags": ["M9", "probe", "statistics", "f-dist", "T-2513"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    p = V.CASES / f"{CID}.case.json"
    print(len(cells), "cells", "exists" if p.exists() else "written")
    if not p.exists():
        p.write_text(json.dumps(case, indent=1) + "\n")


if __name__ == "__main__":
    main()
