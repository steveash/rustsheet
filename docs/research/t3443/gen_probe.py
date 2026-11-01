"""T-3443 probe generator: BETA.DIST / F.DIST at TOMS 708 bratio's basym points (both shapes above
100, |lambda| <= 0.03 min(a, b)) and BETA.INV / F.INV round trips and literal quantiles there.

Three cases with fixed seeds, the split fixed before any scoring: `t3443-basym-a` (in-sample, seed
3443), `t3443-basym-hold` (hold-out, seed 34431, scored once on the frozen rule) and
`t3443-basym-sweep` (seed 34432, generated and filled after the rule was frozen; decision 0188 R4).

    python3 docs/research/t3443/gen_probe.py   # rewrites the three case.json files
"""

from __future__ import annotations

import json
import math
import random
from pathlib import Path

CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"


def lit(v: float) -> str:
    """A formula literal: 15 significant digits, as Excel keeps them."""
    return f"{v:.15g}"


def shape(rng: random.Random, lo: float, hi: float) -> float:
    """Log-uniform shape, rounded to 3 decimals half the time and to an integer otherwise."""
    v = math.exp(rng.uniform(math.log(lo), math.log(hi)))
    return round(v, 3) if rng.random() < 0.5 else float(round(v))


def basym_x(rng: random.Random, a: float, b: float) -> float:
    """x with bratio's lambda = a - (a + b) x inside +-0.03 min(a, b) (times 0.95), log-uniform in
    |lambda| from a tenth of a standard deviation, either sign."""
    lam_max = 0.95 * 0.03 * min(a, b)
    sd = math.sqrt(a * b / (a + b))
    lam_min = min(0.1 * sd, lam_max / 10)
    lam = math.exp(rng.uniform(math.log(lam_min), math.log(lam_max))) * rng.choice((-1.0, 1.0))
    return (a - lam) / (a + b)


def beta_args(rng: random.Random) -> tuple[str, str, str, str]:
    a, b = shape(rng, 100.5, 1e5), shape(rng, 100.5, 1e5)
    x = basym_x(rng, a, b)
    bounds = ""
    if rng.random() < 0.25:
        lo = round(rng.uniform(-2, 1), 2)
        hi = round(lo + rng.uniform(0.5, 5), 2)
        x = lo + x * (hi - lo)
        bounds = f",{lit(lo)},{lit(hi)}"
    return lit(x), lit(a), lit(b), bounds


def f_args(rng: random.Random) -> tuple[str, str, str]:
    d1, d2 = int(shape(rng, 201, 2e5)), int(shape(rng, 201, 2e5))
    x = basym_x(rng, d1 / 2, d2 / 2)
    return lit(d2 * x / (d1 * (1 - x))), str(d1), str(d2)


def cells(rng: random.Random) -> list[str]:
    out: list[str] = []
    for i in range(50):
        x, a, b, bd = beta_args(rng)
        legacy = i % 10 == 9
        out.append(f"=BETADIST({x},{a},{b}{bd})" if legacy else f"=BETA.DIST({x},{a},{b},TRUE{bd})")
    for _ in range(25):
        f, d1, d2 = f_args(rng)
        out.append(f"=F.DIST({f},{d1},{d2},TRUE)")
    for i in range(15):
        f, d1, d2 = f_args(rng)
        out.append(f"={'FDIST' if i % 5 == 4 else 'F.DIST.RT'}({f},{d1},{d2})")
    for i in range(50):
        x, a, b, bd = beta_args(rng)
        inv = "BETAINV" if i % 10 == 9 else "BETA.INV"
        out.append(f"={inv}(BETA.DIST({x},{a},{b},TRUE{bd}),{a},{b}{bd})")
    for _ in range(40):
        _, a, b, _ = beta_args(rng)
        p = rng.uniform(0.02, 0.98) if rng.random() < 0.8 else 10 ** -rng.uniform(1.5, 8)
        if rng.random() < 0.5 and p < 0.02:
            p = 1 - p
        out.append(f"=BETA.INV({lit(p)},{a},{b})")
    for _ in range(25):
        f, d1, d2 = f_args(rng)
        out.append(f"=F.INV(F.DIST({f},{d1},{d2},TRUE),{d1},{d2})")
    for i in range(15):
        f, d1, d2 = f_args(rng)
        inv = "FINV" if i % 5 == 4 else "F.INV.RT"
        out.append(f"={inv}(F.DIST.RT({f},{d1},{d2}),{d1},{d2})")
    return out


def write(name: str, seed: int, role: str) -> None:
    rng = random.Random(seed)
    formulas = cells(rng)
    refs = [f"A{i + 1}" for i in range(len(formulas))]
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            f"T-3443 {role} (seed {seed}): BETA.DIST, F.DIST and F.DIST.RT at TOMS 708 bratio's basym points "
            "(both shapes 100.5 to 1E5, |lambda| up to 0.95 * 0.03 min(a, b)), then BETA.INV and F.INV "
            "round trips and literal BETA.INV quantiles at those shapes: Excel's quantile crossing at basym "
            "points. Generator docs/research/t3443/gen_probe.py."
        ),
        "tags": ["probe", "numeric", "statistics", "T-3443"],
        "sheets": [{"name": "Calc", "cells": dict(zip(refs, formulas))}],
        "check": [f"Calc!{r}" for r in refs],
        "expected": {},
    }
    (CASES / f"{name}.case.json").write_text(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    write("t3443-basym-a", 3443, "in-sample")
    write("t3443-basym-hold", 34431, "hold-out")
    write("t3443-basym-sweep", 34432, "fresh sweep after the freeze")
