"""T-3285 (decision 0188 R4.2): a fresh log-uniform sweep of CHISQ.DIST.RT over the accurate half-integer kernel's
whole range, filled after the kernel was frozen and never used for a choice.

  CHISQ.DIST.RT(2x, 2a) = Q(a, x) for a in {1.5, 2.5, ..., 19.5} (df 3 to 39, odd) and x log-uniform in [a + 1, 650]
  (`half_integer_upper`'s range up to its MAX_X): 12 points per shape, 228 cells. The chi-squared argument 2x is
  rounded to 10 significant digits, so the formula text is the double Excel reads (Excel keeps 15 digits).

Writes eval/suites/probes-numeric/cases/t3285-chisq-dist-rt-half-integer-sweep.case.json and, beside this script,
mpmath.json: Q(a, x) at 50 digits (mpmath `gammainc(a, x, inf, regularized=True)`) per cell, as a decimal string.
Usage (repo root): uv run --with mpmath python docs/research/t3285/mksweep.py [--write]"""

from __future__ import annotations

import json
import math
import random
import sys
from pathlib import Path

import mpmath

HERE = Path(__file__).parent
CASE = Path("eval/suites/probes-numeric/cases/t3285-chisq-dist-rt-half-integer-sweep.case.json")
SHAPES = [n + 0.5 for n in range(1, 20)]
PER_SHAPE = 12
MAX_X = 650.0
SEED = 3285


def points() -> list[tuple[float, float]]:
    """(df, chi2) per cell, in row order."""
    rng = random.Random(SEED)
    out = []
    for a in SHAPES:
        lo, hi = math.log(a + 1.0), math.log(MAX_X)
        xs = sorted(math.exp(rng.uniform(lo, hi)) for _ in range(PER_SHAPE))
        for x in xs:
            chi2 = float(f"{2.0 * x:.10g}")
            chi2 = max(chi2, 2.0 * (a + 1.0))
            out.append((2.0 * a, chi2))
    return out


def fmt(v: float) -> str:
    return repr(int(v)) if v == int(v) else repr(v)


def main() -> None:
    mpmath.mp.dps = 50
    pts = points()
    cells, hyp, ref = {}, {}, {}
    for row, (df, chi2) in enumerate(pts, start=1):
        cell = f"A{row}"
        cells[cell] = f"=CHISQ.DIST.RT({fmt(chi2)},{fmt(df)})"
        hyp[f"Calc!{cell}"] = f"t3285-sweep/a={df / 2}/x={chi2 / 2!r}"
        q = mpmath.gammainc(mpmath.mpf(df) / 2, mpmath.mpf(chi2) / 2, mpmath.inf, regularized=True)
        ref[f"Calc!{cell}"] = mpmath.nstr(q, 50, min_fixed=1, max_fixed=0)
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-3285 fresh sweep (decision 0188 R4.2): CHISQ.DIST.RT(2x, 2a) = Q(a, x) at half-integer a from 1.5 to "
            f"19.5, x log-uniform in [a + 1, 650], {PER_SHAPE} per shape ({len(pts)} cells); "
            "docs/research/t3285/mksweep.py, 50-digit mpmath values in docs/research/t3285/mpmath.json."
        ),
        "tags": ["M9", "probe", "statistics", "T-3285", "sweep"],
        "expected": {},
        "hypotheses": hyp,
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    if "--write" in sys.argv:
        CASE.write_text(json.dumps(case, indent=1) + "\n")
        (HERE / "mpmath.json").write_text(json.dumps(ref, indent=1) + "\n")
    print(f"{len(pts)} cells")


if __name__ == "__main__":
    main()
