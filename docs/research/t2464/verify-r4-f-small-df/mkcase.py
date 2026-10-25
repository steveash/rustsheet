"""verify-r4-f-small-df: build the verifier's own hold-out case t2586-verify-r4-f-small-df-hold.

16 shapes x 4 runs x 14 consecutive doubles; column A = x as an exact two-piece formula, B = F.DIST(A,m,n,FALSE).
Run targets r = 0.21, 0.55 (side 1: series / LN(1+r) branch of log1p), r = 1/1.9, 1/4.7 seen from side 2
(n/(m x) = 0.526, 0.213). Built before any score was computed on it.
Usage: python mkcase.py  (writes the case into eval/suites/probes-numeric/cases)"""

from __future__ import annotations

import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CASE_ID = "t2586-verify-r4-f-small-df-hold"
SHAPES = [
    (6, 5), (8, 5), (10, 8), (11, 5), (6, 10), (8, 11),
    (11, 10), (12, 5), (1, 7), (7, 1), (2, 11), (11, 2),
    (1, 1), (2, 2), (1, 3), (4, 1),
]
# (side, r): side 1 x = r n / m; side 2 x = n / (m r)
RUNS = [(1, 0.21), (1, 0.55), (2, 1 / 1.9), (2, 1 / 4.7)]
PER_RUN = 14
JITTER = 0.0000173  # moves run starts off round numbers


def encode(x: float) -> str:
    """Exact formula for a positive double: (hi*2^(e+26) + lo*2^e) with hi, lo < 2^27."""
    mant, ex = math.frexp(x)
    M = int(mant * 2**53)
    e = ex - 53
    hi, lo = M >> 26, M & ((1 << 26) - 1)
    return f"=({hi}*2^{e + 26}+{lo}*2^{e})"


def plan():
    out = []
    for m, n in SHAPES:
        for side, r in RUNS:
            x = r * n / m if side == 1 else n / (m * r)
            x *= 1 + JITTER * (1 + (m * 7 + n * 3) % 11)
            for _ in range(PER_RUN):
                out.append((x, m, n))
                x = math.nextafter(x, math.inf)
    return out


def main() -> None:
    cells = {}
    for i, (x, m, n) in enumerate(plan(), start=1):
        cells[f"A{i}"] = encode(x)
        cells[f"B{i}"] = f"=F.DIST(A{i},{m},{n},FALSE)"
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-2586 verify-r4-f-small-df hold-out: F.DIST(x,d1,d2,FALSE) on runs of consecutive exact doubles "
        "(column A echoes x), 16 shapes, built by the verifier to score the frozen M-sd once.",
        "tags": ["M9", "probe", "statistics", "f-dist", "T-2586"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    p = ROOT / "eval/suites/probes-numeric/cases" / f"{CASE_ID}.case.json"
    p.write_text(json.dumps(case, indent=1) + "\n")
    print(p, len(cells), "cells")


if __name__ == "__main__":
    main()
