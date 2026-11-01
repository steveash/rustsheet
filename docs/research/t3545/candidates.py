"""T-3545: sinh candidates for IMSIN's imaginary part at small |y|, on the 21 pure-imaginary IMSIN cells
(x = 0, |y| < 0.0105) of probes-numeric/t3525-im-hyperbolic-fresh-sweep, where the part is sinh y itself.

Each candidate's value is rounded to 15 significant digits (the IM* text) and compared with Excel's part.
Run from tools/: uv run python ../docs/research/t3545/candidates.py
"""

from __future__ import annotations

import json
import math
import re
from pathlib import Path

from rustsheet_tools.fitlab.exact import EXTENDED, X87, double_round, exact, exp_cr, round_to_bits
from rustsheet_tools.fitlab.kernels.fisher_expon_harmean import kahan_expm1
from rustsheet_tools.fitlab.sweep import parse_complex_text

CASE = (
    Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases/t3525-im-hyperbolic-fresh-sweep"
)


def cr_exp(t: float) -> float:
    return float(exp_cr(t))  # correctly rounded to a double


def extended_pair(y: float) -> float:
    a, b = exp_cr(y, EXTENDED), exp_cr(-y, EXTENDED)
    return float(round_to_bits(round_to_bits(a - b, EXTENDED) / 2, EXTENDED))


def kahan_sinh(y: float) -> float:
    e = kahan_expm1(y)
    return (e + e / (e + 1)) / 2


CANDIDATES = {
    "(EXP(y) - 1/EXP(y))/2, x87 EXP": lambda y: (X87.exp(y) - 1 / X87.exp(y)) / 2,
    "(EXP(y) - 1/EXP(y))/2, libm exp": lambda y: (math.exp(y) - 1 / math.exp(y)) / 2,
    "(EXP(y) - 1/EXP(y))/2, correctly rounded exp": lambda y: (cr_exp(y) - 1 / cr_exp(y)) / 2,
    "(EXP(y) - EXP(-y))/2, x87 EXP": lambda y: (X87.exp(y) - X87.exp(-y)) / 2,
    "(EXP(y) - EXP(-y))/2, extended subtraction": lambda y: (
        double_round(exact(X87.exp(y)) - exact(X87.exp(-y))) / 2
    ),
    "(EXP(y) - EXP(-y))/2, all extended": extended_pair,
    "(1/EXP(-y) - EXP(-y))/2": lambda y: (1 / X87.exp(-y) - X87.exp(-y)) / 2,
    "libm sinh": math.sinh,
    "Kahan expm1 sinh": kahan_sinh,
}


def main() -> None:
    cells = json.loads(CASE.with_suffix(".case.json").read_text())["sheets"][0]["cells"]
    oracle = json.loads(CASE.with_suffix(".oracle.json").read_text())["sheets"]["S"]
    rows = []
    for key, formula in cells.items():
        m = re.search(r'"(.*)"', formula)
        if not key.startswith("C") or m is None:
            continue
        x, y = parse_complex_text(m[1])
        if x == 0 and abs(y) < 0.0105:
            rows.append((y, parse_complex_text(oracle[key]["v"])[1]))
    for name, f in CANDIDATES.items():
        hits = sum(float(f"{f(y):.15g}") == part for y, part in rows)
        print(f"{hits:>3}/{len(rows)}  {name}")


if __name__ == "__main__":
    main()
