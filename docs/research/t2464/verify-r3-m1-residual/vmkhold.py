"""Build this verifier's own hold-out probe eval/suites/probes-numeric/cases/t2513-verify-r3-m1-residual-hold.case.json
(< 2,000 cells), on shapes in no earlier dataset, made before scoring the frozen model on it.

Layout (sheet Calc): H/I = a/b of shape k (row k), J/K = an argument and =GAMMALN(J) (Excel's GAMMALN, an input of
M2's GL form), A = x as an exact two-piece literal, B = BETA.DIST(A,$H$k,$I$k,FALSE). 27 rows per shape.
"""

from __future__ import annotations

import json
import math
import random

import vlib as V

CASE_ID = "t2513-verify-r3-m1-residual-hold"
SHAPES = {
    # min < 1, max <= 8 (GL form), incl. b < 1, both < 1 and the max = 8 boundary
    "gl": "0.13:2.2 0.27:4.15 0.39:7.95 0.48:1.05 0.63:3.3 0.74:6.6 0.92:5.05 0.36:0.82 0.57:0.21 "
    "1.85:0.44 4.6:0.73 7.2:0.16 0.4:8.0 8.0:0.66",
    # min = 1 (-LN(max)): a = 1 and b = 1, max below 2, between 2 and 8 and above 8
    "min1": "1:1.35 1:4.7 1:63.5 1:12345.6 0.37:1 2.6:1 15.2:1 3000.5:1",
    # min < 1 < 8 < max (TOMS algdiv, the open part)
    "algdiv": "0.23:17.0 0.47:8.6 0.81:250.5 0.09:45.0 30.3:0.62 9.2:0.41 0.56:1.2e5 0.33:33.3 0.71:9.0 "
    "0.96:11.4",
}
PER = 27


def lit(v: float) -> str:
    """Exact formula literal: short decimals as written (<= 15 digits, echoed back), else m*2^-k + n*2^-j."""
    r = repr(v)
    if len(r.replace("-", "").replace(".", "").replace("e", "").lstrip("0")) <= 12 and "e" not in r:
        assert V.eval_arg(r, {}) == v, (v, r)
        return r
    m, e = math.frexp(v)  # v = m * 2^e, 0.5 <= m < 1
    M = int(m * 2**53)
    hi, lo = M >> 26, M & (2**26 - 1)
    k1, k2 = 27 - e, 53 - e
    assert k1 > 0, v
    out = f"({hi}*2^-{k1}+{lo}*2^-{k2})"
    assert V.eval_arg(out, {}) == v, (v, out)
    return out


def density_ok(x: float, a: float, b: float) -> bool:
    """The density is far from overflow and underflow (a screen only, in floats)."""
    lg = (a - 1) * math.log(x) + (b - 1) * math.log1p(-x) - (math.lgamma(a) + math.lgamma(b) - math.lgamma(a + b))
    return -640.0 < lg < 640.0


def xs_for(a: float, b: float, rng: random.Random) -> list[float]:
    """PER distinct x with a moderate exponent ((b-1)(-ln(1-x)) and (a-1)(-ln x) up to 300), log- and
    uniformly spread, so both ly zones (x <= 0.375 and above) appear wherever the shape allows."""
    hi = 1.0 - math.exp(-300.0 / (b - 1.0)) if b > 1.0 else 0.9995
    lo = math.exp(-300.0 / (a - 1.0)) if a > 1.0 else 1e-6
    hi, lo = min(hi, 0.9995), max(lo, 1e-6)
    out: list[float] = []
    while len(out) < PER:
        u = rng.random()
        x = lo * (hi / lo) ** u if len(out) % 2 else lo + (hi - lo) * u
        if 0.0 < x < 1.0 and density_ok(x, a, b) and x not in out:
            out.append(x)
    return out


def main() -> None:
    rng = random.Random(20261024)
    shapes = [(float(a), float(b)) for blk in SHAPES.values() for a, b in (s.split(":") for s in blk.split())]
    seen = set()
    for name in ("old", "probe1", "a", "hold"):
        rows = (
            V.rows_old()
            if name == "old"
            else V.rows_probe1()
            if name == "probe1"
            else V.rows_case(f"t2513-r3-m1-residual-{name}")
        )
        seen |= {(r["a"], r["b"]) for r in rows}
    assert not set(shapes) & seen, set(shapes) & seen
    cells: dict = {}
    args = []
    for k, (a, b) in enumerate(shapes, start=1):
        cells[f"H{k}"], cells[f"I{k}"] = "=" + lit(a), "=" + lit(b)
        lo, hi = min(a, b), max(a, b)
        if lo < 1.0:
            args += [lo] if hi > 8.0 else [lo, hi, a + b]
    args = sorted(set(args))
    for j, v in enumerate(args, start=1):
        cells[f"J{j}"], cells[f"K{j}"] = "=" + lit(v), f"=GAMMALN(J{j})"
    r = 0
    for k, (a, b) in enumerate(shapes, start=1):
        for x in xs_for(a, b, rng):
            r += 1
            cells[f"A{r}"] = "=" + lit(x)
            cells[f"B{r}"] = f"=BETA.DIST(A{r},$H${k},$I${k},FALSE)"
    assert len(cells) <= 2000, len(cells)
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-2513 verify-r3-m1-residual hold-out: BETA.DIST(x,a,b,FALSE) with min(a,b) <= 1 on 32 new "
        "shapes (GL form, min = 1, algdiv) plus Excel's GAMMALN at the shapes' lnB arguments; all inputs are "
        "exact literals echoed back",
        "tags": ["M9", "probe", "statistics", "T-2513", "incomplete-beta"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    (V.CASES / f"{CASE_ID}.case.json").write_text(json.dumps(case, indent=1) + "\n")
    print(CASE_ID, len(shapes), "shapes", r, "density rows", len(args), "GAMMALN args", len(cells), "cells")


if __name__ == "__main__":
    main()
