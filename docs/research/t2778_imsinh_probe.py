"""Build probes/t2778-imsinh-sign-sweep (T-2778): real IMSINH arguments, both signs, where sinh candidates disagree.

Run from tools/: `uv run python ../docs/research/t2778_imsinh_probe.py [--write]`.

IMSINH returns text with 15 significant digits, so a sinh kernel shows only where its double sits next to a
15-digit rounding boundary. The search walks 15-digit arguments in (0.01, 40) and keeps those where the candidates
below give different 15-digit texts; each is probed at +x and -x. Without `--write` it prints how many arguments
each pair of candidates is split on; with it, it writes the case.
"""

from __future__ import annotations

import json
import math
import random
import sys
from collections.abc import Callable
from fractions import Fraction
from pathlib import Path

from rustsheet_tools.fitlab.exact import EXTENDED, X87Insn, exp_cr, round_to_bits, to_double

ROOT = Path(__file__).resolve().parents[2]
CASE = ROOT / "eval/suites/probes/cases/t2778-imsinh-sign-sweep.case.json"
COUNT = 150  # arguments, each at both signs


def _dbl(v: Fraction) -> Fraction:
    return Fraction(to_double(v))


def _ext(v: Fraction) -> Fraction:
    return round_to_bits(v, EXTENDED)


def _exp_dbl(x: float) -> Fraction:
    return Fraction(math.exp(x))


def _exp_ext(x: float) -> Fraction:
    return exp_cr(x, EXTENDED)


def _exp_x87(x: float) -> Fraction:
    return X87Insn.exp(x)


def _two_exps(
    plus: Callable[[float], Fraction], minus: Callable[[float], Fraction]
) -> Callable[[float], float]:
    """(E(x) - E(-x))/2 with each EXP at its own precision, the difference and halving at 64 bits."""
    return lambda x: to_double(_ext(_ext(plus(x) - minus(-x)) / 2))


def _reciprocal(
    e: Callable[[float], Fraction], rnd: Callable[[Fraction], Fraction]
) -> Callable[[float], float]:
    """(e - 1/e)/2 from one EXP."""

    def sinh(x: float) -> float:
        v = e(x)
        return to_double(rnd(rnd(v - rnd(1 / v)) / 2))

    return sinh


def _expm1_form(x: float) -> float:
    """glibc's sinh for |x| < 22: t = expm1(|x|), 0.5*(2t - t^2/(t+1)) below 1, 0.5*(t + t/(t+1)) above."""
    t = math.expm1(abs(x))
    h = math.copysign(0.5, x)
    if abs(x) < 1:
        return h * (2 * t - t * t / (t + 1))
    return h * (t + t / (t + 1))


CANDIDATES: dict[str, Callable[[float], float]] = {
    "libm sinh": math.sinh,
    "correctly rounded": lambda x: to_double((exp_cr(x, 120) - exp_cr(-x, 120)) / 2),
    "(EXP(x)-EXP(-x))/2 doubles": lambda x: (math.exp(x) - math.exp(-x)) / 2,
    "E+ ext, E- double": _two_exps(_exp_ext, _exp_dbl),
    "E+ double, E- ext": _two_exps(_exp_dbl, _exp_ext),
    "E+ x87, E- double": _two_exps(_exp_x87, _exp_dbl),
    "E+ x87, E- x87": _two_exps(_exp_x87, _exp_x87),
    "(e-1/e)/2 e ext": _reciprocal(_exp_ext, _ext),
    "(e-1/e)/2 e double": _reciprocal(_exp_dbl, _dbl),
    "glibc expm1 form": _expm1_form,
}


def text15(d: float) -> float:
    """The double IMSINH's 15-digit text parses back to."""
    return float(f"{d:.15g}")


def spread(x: float) -> int:
    """How many distinct 15-digit texts the candidates give at x and -x together."""
    return len({(text15(f(x)), text15(f(-x))) for f in CANDIDATES.values()})


def search(count: int, seed: int = 2778) -> list[float]:
    rng = random.Random(seed)
    found: list[float] = []
    while len(found) < count:
        x = float(f"{math.exp(rng.uniform(math.log(0.01), math.log(40))):.15g}")
        if spread(x) > 1 and x not in found:
            found.append(x)
    return sorted(found)


def build(xs: list[float]) -> dict[str, object]:
    cells: dict[str, object] = {}
    for i, x in enumerate(xs, 1):
        cells[f"A{i}"] = f'=IMSINH("{x!r}")'
        cells[f"B{i}"] = f'=IMSINH("{-x!r}")'
    # T-2535's EXP form for IMSIN and IMCOS never saw imaginary parts where it and libm differ.
    extra = [f'IM{fn}("{s}{y}i")' for fn in ("SIN", "COS") for y in ("4", "12", "0.01") for s in ("", "-")]
    for i, f in enumerate(extra, 1):
        cells[f"C{i}"] = f"={f}"
    return {
        "format": "rustsheet-case/v1",
        "description": (
            "T-2778: IMSINH of 15-digit real arguments at both signs (A: +x, B: -x), each chosen where sinh "
            "candidates (libm, correctly rounded, (EXP(x)-EXP(-x))/2 at mixed precisions, (e-1/e)/2, glibc's expm1 "
            "form) give different 15-digit texts; C: IMSIN and IMCOS at imaginary parts +-4, +-12 and +-0.01. "
            "Generator: docs/research/t2778_imsinh_probe.py."
        ),
        "tags": ["M9", "engineering", "complex", "probe"],
        "sheets": [{"name": "S", "cells": cells}],
        "expected": {},
    }


def main() -> None:
    xs = search(COUNT)
    print(f"{len(xs)} arguments, {2 * len(xs) + 12} cells; range {xs[0]} .. {xs[-1]}")
    if "--write" in sys.argv:
        CASE.write_text(json.dumps(build(xs), indent=2) + "\n")
        print(f"wrote {CASE}")


if __name__ == "__main__":
    main()
