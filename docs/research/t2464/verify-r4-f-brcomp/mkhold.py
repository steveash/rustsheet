"""Verifier hold-out t2586-verify-r4-f-brcomp-hold: 16 shapes in no earlier F.DIST(.,FALSE) probe x 4 runs of 15
consecutive doubles at x ~ 0.3, 0.6, 1.4, 4.0 (none of the worker's run centres); A = x (exact two-term encoding),
B = F.DIST(A, d1, d2, FALSE). 960 rows, 1,920 cells. Usage: mkhold.py [--write]"""
import json
import math
import sys
from fractions import Fraction

import vlib as V

ID = "t2586-verify-r4-f-brcomp-hold"
SHAPES = [(6, 5), (8, 5), (10, 8), (12, 12), (13, 26), (17, 17), (21, 50), (26, 13),
          (33, 33), (44, 88), (55, 25), (75, 75), (96, 48), (130, 65), (180, 90), (240, 240)]
XS = (0.3, 0.6, 1.4, 4.0)
RUN = 15


def encode(x: float) -> str:
    """x = M * 2^e with a 53-bit M, written as hi*2^(e+27) + lo*2^e (both integer literals below 2^27)."""
    fr = Fraction(x)
    m, e = math.frexp(x)
    M = int(Fraction(m) * 2**53)
    e -= 53
    assert Fraction(M) * Fraction(2) ** e == fr
    hi, lo = divmod(M, 2**27)
    return f"=({hi}*2^{e + 27}+{lo}*2^{e})"


def rows():
    out, i = [], 0
    for d1, d2 in SHAPES:
        for c in XS:
            x = c * (1 + 0.000173 * i + 0.0000613)
            i += 1
            for _ in range(RUN):
                out.append((d1, d2, x))
                x = math.nextafter(x, math.inf)
    return out


def build():
    cells = {}
    for r, (d1, d2, x) in enumerate(rows(), 1):
        cells[f"A{r}"] = encode(x)
        assert V.exact_expr(cells[f"A{r}"]) == x
        cells[f"B{r}"] = f"=F.DIST(A{r},{d1},{d2},FALSE)"
    return {"format": "rustsheet-case/v1",
            "description": "T-2586 verify-r4-f-brcomp hold-out: F.DIST(x,d1,d2,FALSE) on runs of consecutive exact "
            "doubles x at 16 new shapes (column A echoes x).",
            "tags": ["M9", "probe", "statistics", "f-dist", "T-2586"],
            "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}


if __name__ == "__main__":
    c = build()
    p = V.CASES / f"{ID}.case.json"
    print(len(c["sheets"][0]["cells"]), "cells", "exists" if p.exists() else "new")
    if "--write" in sys.argv and not p.exists():
        p.write_text(json.dumps(c, indent=1) + "\n")
