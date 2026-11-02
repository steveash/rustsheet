"""T-3411 fresh sweep (decision 0188 R4.2) for IRR's secant cap and stall residual, generated after
both were frozen (task file "R4 plan"). Writes probes-numeric/t3411-irr-r4-fresh with a 50-digit
math TSV of the roots (mpmath bisection in ln v at 60 and 80 digits, kept only when they agree).

Strata (25 cells each), conventional flows (one outlay, then inflows; 30% negated):
  0 long near-level series (20..150 flows) with roots -0.3..-0.01: slow secant runs (the cap)
  1 large flows (1E+8..1E+13) with roots within 0.05 of 0: NPV noise above 1e-7 (the stall)
  2 guesses -0.9999..-0.99 on 5..80 flows: long first runs before the retry from 0.1 (the cap)
  3 long series (20..150 flows) with roots -0.25..-0.05 and spread flows: first-step jumps
  4 2..41 flows, |c| 1E-6..1E+9, roots +-1E-4..2, half with a guess -0.9..0.9
  5 every axis wide

Run: cd tools && uv run python -I ../docs/research/t3411/gen_sweep.py <repo> [case name]
(t3411-irr-r4-fresh: seed "T-3411/r4-fresh"; t3411-irr-r4-fresh-2: seed "T-3411/r4-fresh-2", round 2)
"""

from __future__ import annotations

import json
import math
import random
import sys
from pathlib import Path

import mpmath

from rustsheet_tools.accuracy.literal import round15, text
from rustsheet_tools.accuracy.sampler import log_uniform

REPO = Path(sys.argv[1])
NAME = sys.argv[2] if len(sys.argv) > 2 else "t3411-irr-r4-fresh"
PER = 25
FROZEN = {
    "t3411-irr-r4-fresh": "round 1: secant cap 220, stall residual 1E-4",
    "t3411-irr-r4-fresh-2": "round 2: secant cap 213",
}
STRATA = [
    "long near-level, root -0.3..-0.01",
    "|c| 1E+8..1E+13, root near 0",
    "guess -0.9999..-0.99",
    "long spread, root -0.25..-0.05",
    "typical",
    "all axes wide",
]


def root_at(flows: list[float], dps: int) -> mpmath.mpf | None:
    """The rate r of the one positive root v = 1/(1 + r) of sum c_i v^i, by bisection in ln v."""
    with mpmath.workdps(dps):
        cs = [mpmath.mpf(c) for c in reversed(flows)]

        def p(u: mpmath.mpf) -> mpmath.mpf:
            v, s = mpmath.exp(u), mpmath.mpf(0)
            for c in cs:
                s = s * v + c
            return s

        lo, hi = mpmath.mpf(-1), mpmath.mpf(1)
        while (p(lo) > 0) == (p(hi) > 0):
            lo, hi = lo * 2, hi * 2
            if hi > 4096:
                return None
        sign_lo = p(lo) > 0
        for _ in range(4 * dps):
            mid = (lo + hi) / 2
            if (p(mid) > 0) == sign_lo:
                lo = mid
            else:
                hi = mid
        return +mpmath.expm1(-(lo + hi) / 2)


def irr_math(flows: list[float]) -> str | None:
    a, b = root_at(flows, 60), root_at(flows, 80)
    if a is None or b is None or (b != 0 and abs((a - b) / b) > mpmath.mpf("1e-45")):
        return None
    with mpmath.workdps(80):
        return mpmath.nstr(b, 50, strip_zeros=False)


def point(rng: random.Random, stratum: int) -> tuple[str, str] | None:
    guess = None
    spread = (0.05, 1.0)
    if stratum == 0:
        n, scale, root = (
            round(log_uniform(rng, 20, 150)),
            log_uniform(rng, 1, 1e6),
            -log_uniform(rng, 0.01, 0.3),
        )
        spread = (0.8, 1.0)
    elif stratum == 1:
        n, scale = rng.randint(2, 60), log_uniform(rng, 1e8, 1e13)
        root = (1 if rng.random() < 0.5 else -1) * log_uniform(rng, 1e-6, 0.05)
    elif stratum == 2:
        n, scale = rng.randint(5, 80), log_uniform(rng, 1e-3, 1e9)
        root = (1 if rng.random() < 0.7 else -1) * log_uniform(rng, 1e-4, 0.5)
        guess = -log_uniform(rng, 0.99, 0.9999)
    elif stratum == 3:
        n, scale, root = (
            round(log_uniform(rng, 20, 150)),
            log_uniform(rng, 1e-2, 1e9),
            -log_uniform(rng, 0.05, 0.25),
        )
    elif stratum == 4:
        n, scale = rng.randint(2, 41), log_uniform(rng, 1e-6, 1e9)
        root = (1 if rng.random() < 0.7 else -1) * log_uniform(rng, 1e-4, 0.99 if rng.random() < 0.3 else 2)
        if rng.random() < 0.5:
            guess = (1 if rng.random() < 0.7 else -1) * log_uniform(rng, 1e-3, 0.9)
    else:
        n, scale = round(log_uniform(rng, 2, 150)), log_uniform(rng, 1e-12, 1e15)
        root = (1 if rng.random() < 0.6 else -1) * log_uniform(rng, 1e-6, 0.999)
        if rng.random() < 0.5:
            guess = -log_uniform(rng, 0.99, 0.9999) if rng.random() < 0.5 else log_uniform(rng, 1e-3, 1e4)
    tail = [scale * rng.uniform(*spread) for _ in range(n - 1)]
    with mpmath.workdps(40):
        v = 1 / (mpmath.mpf(1) + root)
        c0 = float(-sum(mpmath.mpf(c) * v ** (i + 1) for i, c in enumerate(tail)))
    if not math.isfinite(c0) or not 1e-300 < abs(c0) < 1e300:
        return None
    flows = [c0, *tail]
    if rng.random() < 0.3:
        flows = [-c for c in flows]
    flows = [round15(c) for c in flows]
    m = irr_math(flows)
    if m is None:
        return None
    body = "{" + ",".join(text(c) for c in flows) + "}"
    formula = f"=IRR({body})" if guess is None else f"=IRR({body},{text(round15(guess))})"
    return (formula, m) if len(formula) <= 8000 else None


def main() -> None:
    rng = random.Random("T-3411/r4-fresh" if NAME == "t3411-irr-r4-fresh" else f"T-3411/{NAME.removeprefix('t3411-irr-')}")
    cells: dict[str, str] = {}
    hypotheses: dict[str, str] = {}
    rows: list[str] = []
    for stratum, label in enumerate(STRATA):
        got = 0
        while got < PER:
            p = point(rng, stratum)
            if p is None:
                continue
            got += 1
            cell = f"A{len(cells) + 1}"
            cells[cell] = p[0]
            hypotheses[f"Calc!{cell}"] = f"irr/{label}"
            rows.append(f"Calc!{cell}\t{p[1]}\t{p[0]}")
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            f"T-3411 fresh sweep (decision 0188 R4.2) of IRR on conventional flows, generated after the model was "
            f"frozen ({FROZEN[NAME]}): six strata of 25 (docs/research/t3411/gen_sweep.py). "
            "Math: math.tsv (50-digit mpmath roots)."
        ),
        "tags": ["M9", "probe", "financial", "T-3411", "sweep", "irr", "r4"],
        "expected": {},
        "hypotheses": hypotheses,
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    out = REPO / "eval" / "suites" / "probes-numeric" / "cases"
    (out / f"{NAME}.case.json").write_text(json.dumps(case, indent=1) + "\n")
    (out / f"{NAME}.math.tsv").write_text("cell\tmath\tformula\n" + "\n".join(rows) + "\n")
    print(NAME, len(cells))


main()
