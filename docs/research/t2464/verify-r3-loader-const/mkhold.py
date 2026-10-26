"""verify-r3-loader-const: build the verifier's own hold-out case (written before any model is scored on it).

100 new random shapes (a, b log-uniform in (1.2, 70), 1-4 decimals, none seen in any existing case) x 16 x, all with
both bd0 sides in log form (v >= 0.21); 8 x per shape from the high-density region (ln pdf within 3 of its max), 8
from anywhere with ln pdf >= -30 (larger |E|, which the worker's hold-out barely has). x is written exactly as
(h*2^-p + l*2^-q) with h, l < 2^27, so no literal is truncated. Echoes: x every 8th row (column B), a and b once per
shape (C, D). 1,600 pdf cells, 2,000 cells in all.
Usage: mkhold.py [--write]
"""

from __future__ import annotations

import json
import math
import random
import sys

import vdata
import vmodel as VM

NAME = "t2513-verify-r3-loader-const-hold"
CASE = vdata.SUITES / "probes-numeric" / "cases" / f"{NAME}.case.json"


def exact_expr(x: float) -> str:
    m, e = math.frexp(x)  # x = m 2^e, 0.5 <= m < 1
    M = int(m * 2**53)
    E = e - 53
    h, low = M >> 26, M & ((1 << 26) - 1)
    out = f"({h}*2^{E + 26}+{low}*2^{E})"
    assert h * 2.0 ** (E + 26) + low * 2.0**E == x
    return out


def lnpdf(x: float, a: float, b: float) -> float:
    return (
        (a - 1) * math.log(x)
        + (b - 1) * math.log1p(-x)
        - (math.lgamma(a) + math.lgamma(b) - math.lgamma(a + b))
    )


def both_log(x: float, a: float, b: float, margin: float = 0.21) -> bool:
    k, n, nk = (float(v) for v in VM.pieces(a, b))
    m1, m2 = n * x, n * (1.0 - x)
    return abs(k - m1) / (k + m1) >= margin and abs(nk - m2) / (nk + m2) >= margin


def xs_for(a: float, b: float, rng: random.Random):
    grid = [i / 4000 for i in range(1, 4000)]
    ok = [x for x in grid if both_log(x, a, b) and lnpdf(x, a, b) >= -30]
    if len(ok) < 40:
        return None
    top = max(lnpdf(x, a, b) for x in ok)
    hi = [x for x in ok if lnpdf(x, a, b) >= top - 3]
    out: list[float] = []
    for pool, target in ((hi, 8), (ok, 16)):
        while len(out) < target:
            x = rng.choice(pool) + rng.uniform(-1.2e-4, 1.2e-4)
            if 0 < x < 1 and both_log(x, a, b) and lnpdf(x, a, b) >= -30 and x not in out:
                out.append(x)
    return out


def design():
    rows, _ = vdata.load_all()
    seen = {(r["a"], r["b"]) for r in rows if r["case"] != NAME}  # reproducible after the case exists
    rng = random.Random(91251)
    shapes, plan = [], []
    while len(shapes) < 100:
        a = round(math.exp(rng.uniform(math.log(1.2), math.log(70))), rng.choice((1, 2, 3, 4)))
        b = round(math.exp(rng.uniform(math.log(1.2), math.log(70))), rng.choice((1, 2, 3, 4)))
        if a <= 1 or b <= 1 or (a, b) in seen or (a, b) in shapes:
            continue
        xs = xs_for(a, b, rng)
        if xs is None:
            continue
        shapes.append((a, b))
        plan += [(x, a, b) for x in xs]
    return plan


def build():
    cells, first = {}, set()
    for i, (x, a, b) in enumerate(design(), 1):
        cells[f"A{i}"] = f"=BETA.DIST({exact_expr(x)},{a!r},{b!r},FALSE)"
        if i % 8 == 1:
            cells[f"B{i}"] = f"={exact_expr(x)}"
        if (a, b) not in first:
            first.add((a, b))
            cells[f"C{i}"] = f"={a!r}"
            cells[f"D{i}"] = f"={b!r}"
    return {
        "format": "rustsheet-case/v1",
        "description": "T-2513 verify-r3-loader-const hold-out: BETA.DIST pdf at 100 new shapes a, b > 1, 16 x each "
        "with both Loader bd0 sides in log form; column A the density, B/C/D echo x, a, b (parser check).",
        "tags": ["M9", "probe", "statistics", "T-2513", "incomplete-beta"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }


if __name__ == "__main__":
    c = build()
    n = len(c["sheets"][0]["cells"])
    print(NAME, n, "cells;", "exists" if CASE.exists() else "new")
    assert n <= 2000
    if "--write" in sys.argv and not CASE.exists():
        CASE.write_text(json.dumps(c, indent=1) + "\n")
