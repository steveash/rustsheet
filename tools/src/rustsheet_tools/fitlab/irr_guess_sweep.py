"""Generator of probes/t2565-irr-guess-sweeps (T-2565).

For IRR flows whose root is far above the default guess, the case asks Excel for `IRR(flows, g_k)`
at every iterate r_k = 1/v_k - 1 of the fit-lab secant's run from guess 0.1 (`irr_secant_v`), and
for `IRR(flows, 0.1*(1+j*1E-4))`, j = -5..5. Each base flow is also scaled by 1E6 and by -1 (the
residual test depends on scale). Column layout per (flows, scaling) block: A the flows' own
`IRR(flows)`, B the guess, C `=IRR(flows, B)`.

  cd tools && uv run python -m rustsheet_tools.fitlab.irr_guess_sweep case
"""

from __future__ import annotations

import contextlib
import json
import math
import sys
from pathlib import Path

from rustsheet_tools.fitlab.kernels.solvers import NoResultError, irr_secant_v

ROOT = Path(__file__).resolve().parents[4]
CASE = ROOT / "eval/suites/probes/cases/t2565-irr-guess-sweeps"
BASES: list[tuple[str, list[float]]] = [
    ("c0681a8b", [-0.0001, 2, 42]),
    ("67a4c03a", [-2.5, -1, 123456789.123]),
]
SCALES: list[tuple[str, float]] = [("x1", 1.0), ("x1e6", 1e6), ("x-1", -1.0)]
J_RANGE = range(-5, 6)


def literal(x: float) -> str:
    """`x` as a 17-significant-digit formula literal."""
    if float(x).is_integer() and abs(x) < 1e15:
        return str(int(x))
    return f"{x:.17g}".replace("e+", "E+").replace("e-", "E-")


def iterates(flows: list[float]) -> list[float]:
    """Every iterate r_k of the model's secant run from guess 0.1, as rates."""
    trace: list[float] = []
    with contextlib.suppress(NoResultError, ZeroDivisionError, OverflowError):
        irr_secant_v(trace=trace)(flows, 0.1, None)
    return [r for r in trace if math.isfinite(r) and r > -1]


def blocks() -> list[tuple[str, str, list[float], list[float]]]:
    """(base id, scale id, flows, guesses) for every block; guesses are the iterates then the j sweep."""
    out = []
    for base, flows in BASES:
        for scale_id, scale in SCALES:
            scaled = [c * scale for c in flows]
            guesses = iterates(scaled) + [0.1 * (1 + j * 1e-4) for j in J_RANGE]
            out.append((base, scale_id, scaled, guesses))
    return out


def write_case() -> None:
    cells: dict[str, str] = {}
    layout: list[str] = []
    row = 1
    for base, scale_id, flows, guesses in blocks():
        array = "{" + ",".join(literal(c) for c in flows) + "}"
        cells[f"A{row}"] = f"=IRR({array})"
        first = row
        for g in guesses:
            cells[f"B{row}"] = literal(g)
            cells[f"C{row}"] = f"=IRR({array},B{row})"
            row += 1
        n_iter = len(guesses) - len(J_RANGE)
        layout.append(
            f"{base} {scale_id}: A{first} no guess; rows {first}-{first + n_iter - 1} iterates g_k (k = 1..{n_iter}),"
            f" rows {first + n_iter}-{row - 1} j = -5..5"
        )
    description = (
        "IRR guess sweeps on large-root flows (T-2565): for {-0.0001,2,42} (c0681a8b) and {-2.5,-1,123456789.123}"
        " (67a4c03a), each as is, scaled by 1E6 and by -1: IRR(flows, g_k) at every iterate of the fit-lab secant run"
        " from 0.1 (irr_secant_v trace, as 17-digit literals in column B) then IRR(flows, 0.1*(1+j*1E-4)), j = -5..5."
        " Column C = IRR(flows, B). Blocks: " + "; ".join(layout) + "."
    )
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M9", "probe"],
        "sheets": [{"name": "P", "cells": cells}],
        "expected": {},
    }
    Path(f"{CASE}.case.json").write_text(json.dumps(case, indent=1), encoding="utf-8")


def main() -> None:
    {"case": write_case}[sys.argv[1]]()


if __name__ == "__main__":
    main()
