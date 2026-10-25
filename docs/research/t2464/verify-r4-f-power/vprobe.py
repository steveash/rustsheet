"""verify-r4-f-power: a fresh hold-out for the frozen r4-f-power `dd` model (built after it was frozen).
10 shapes not in any r4/r3 probe x 5 random x centres (log-uniform 0.3..150, seed 2586) x 20 consecutive doubles.
A = x as an exact two-piece formula (own encoder), B = F.DIST(A, d1, d2, FALSE).  Usage: vprobe.py [--write]"""
from __future__ import annotations

import json
import math
import random
import sys
from pathlib import Path

CASES = Path(__file__).resolve().parents[4] / "eval/suites/probes-numeric/cases"
CASE_ID = "t2586-verify-r4-f-power-hold"
SHAPES = [(56, 112), (130, 70), (76, 76), (44, 176), (180, 54), (36, 144), (96, 64), (160, 160), (66, 198), (88, 52)]
CENTRES, COUNT, SEED = 5, 20, 2586


def enc(x: float) -> str:
    """x = hi*2^-e1 + lo*2^-e2 exactly, hi < 2^27, lo < 2^26 (integers of at most 9 digits, no truncation)."""
    m, e = math.frexp(x)  # x = m 2^e, 0.5 <= m < 1
    q = int(m * 2**53)  # 53-bit integer, x = q 2^(e-53)
    hi, lo = q >> 26, q & (2**26 - 1)
    return f"=({hi}*2^{e - 27}+{lo}*2^{e - 53})"


def runs():
    rnd = random.Random(SEED)
    out = []
    for d1, d2 in SHAPES:
        for _ in range(CENTRES):
            x = math.exp(rnd.uniform(math.log(0.3), math.log(150.0)))
            for _k in range(COUNT):
                out.append((d1, d2, x))
                x = math.nextafter(x, math.inf)
    return out


def build():
    cells = {}
    for r, (d1, d2, x) in enumerate(runs(), 1):
        cells[f"A{r}"] = enc(x)
        cells[f"B{r}"] = f"=F.DIST(A{r},{d1},{d2},FALSE)"
    return {
        "format": "rustsheet-case/v1",
        "description": "T-2586 verify-r4-f-power fresh hold-out: F.DIST(x,d1,d2,FALSE) at 10 new large shapes, "
        "random x centres, 20 consecutive doubles each (x by an exact two-piece formula, echoed in column A).",
        "tags": ["M9", "probe", "statistics", "f-dist", "T-2586"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }


if __name__ == "__main__":
    c = build()
    p = CASES / f"{CASE_ID}.case.json"
    print(len(c["sheets"][0]["cells"]), "cells", p, "exists" if p.exists() else "new")
    if "--write" in sys.argv and not p.exists():
        p.write_text(json.dumps(c, indent=1) + "\n")
