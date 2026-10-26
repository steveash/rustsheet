"""Build probes/npv-term-order (T-2671): NPVs that take Excel's NPV apart one rounding at a time.

Run from tools/: `uv run python ../docs/research/t2671_npv_term_order.py [--write]`.

Every cell is scored against T-1528's NPV candidates (fitlab `annuity-npv`, `npv_candidates`), and the
searched rates are the ones where those candidates give the most distinct doubles, so each cell separates
them. Without `--write` it prints, per rate, how many distinct candidate doubles its cells produce; with
it, it writes eval/suites/probes/cases/npv-term-order.case.json.

Cells (sheet P; the rates are exact doubles in R!A1:A20, so a 16- or 17-digit rate reaches Excel whole):
- one-term NPV(r,v): the first division alone;
- zero-padded NPV(r,0,..,0,1) for periods 2 to 12: each period's discount factor alone;
- two- and three-term NPVs with values of opposite sign and close magnitude: the summation order;
- on IRR.xlsx's rates, the IRR flows themselves (the residue cells of financial.md).
"""

from __future__ import annotations

import json
import random
import sys
from dataclasses import dataclass
from pathlib import Path

from rustsheet_tools.fitlab.kernels.annuity import npv_candidates

ROOT = Path(__file__).resolve().parents[2]
CASE = ROOT / "eval/suites/probes/cases/npv-term-order.case.json"

# IRR.xlsx (Excel-authored): General!L2, L3, L8 and Three Parameters!D2, with the flows they are roots of.
IRR_RATES: list[tuple[str, float, list[float]]] = [
    ("IRR.xlsx General!L2", 0.07930826116039724, [-20, 5, 5, 5, 5, 5]),
    ("IRR.xlsx General!L3", -0.1015006760535847, []),
    ("IRR.xlsx General!L8", -0.44350694133432755, []),
    ("IRR.xlsx Three Parameters!D2", 1.091399510766422e-07, [1, -2, 1]),
]
SWEEP_RATES = [0.05, 0.07, 0.1, 0.125, 0.2]  # probes-numeric/m9-fin-nper-npv
SEARCHED = 11  # rates picked by the search, to make 20


@dataclass
class _Cell:
    args: tuple[float, ...]


def shapes(r: float) -> list[tuple[float, ...]]:
    """The value lists probed at rate r (without the rate)."""
    out: list[tuple[float, ...]] = [(1.0,), (7.0,)]
    out += [tuple([0.0] * (n - 1) + [1.0]) for n in range(2, 13)]
    out += [(-20.0, 21.0), (5.0, -5.0), (-1.0, 1.0 + r)]
    out += [(1.0, -2.0, 1.0), (-20.0, 5.0, 15.0), (3.0, -7.0, 4.0)]
    return out


CANDIDATES = npv_candidates()


def distinct(r: float, values: tuple[float, ...]) -> set[float]:
    """The distinct doubles T-1528's candidates give for NPV(r, values)."""
    cell = _Cell((r, *values))
    out = set()
    for c in CANDIDATES:
        try:
            out.add(c.evaluate(cell))  # type: ignore[arg-type]
        except (ValueError, ZeroDivisionError, OverflowError):
            continue
    return out


def score(r: float) -> tuple[int, int]:
    """(cells where the candidates disagree, total distinct doubles) over r's shapes."""
    sets = [distinct(r, v) for v in shapes(r)]
    return sum(len(s) > 1 for s in sets), sum(len(s) for s in sets)


def search(count: int, seed: int = 2671) -> list[float]:
    """Rates with 15 significant digits in (-0.5, 0.5) where the candidates disagree most."""
    rng = random.Random(seed)
    pool = [float(f"{rng.uniform(-0.45, 0.45):.15g}") for _ in range(120)]
    ranked = sorted(pool, key=score, reverse=True)
    return ranked[:count]


def fmt(v: float) -> str:
    """A value as a formula literal (all shape values are small integers or the exact 1+r cell)."""
    return repr(int(v)) if v == int(v) else repr(v)


def build(rates: list[tuple[str, float, list[float]]]) -> dict[str, object]:
    r_cells: dict[str, object] = {}
    p_cells: dict[str, object] = {}
    row = 0
    for i, (_, r, flows) in enumerate(rates, 1):
        r_cells[f"A{i}"] = r
        r_cells[f"B{i}"] = f"=1+A{i}"
        ref = f"R!$A${i}"
        for values in shapes(r):
            row += 1
            if values == (-1.0, 1.0 + r):
                args = f"-1,R!$B${i}"
            else:
                args = ",".join(fmt(v) for v in values)
            p_cells[f"A{row}"] = f"=NPV({ref},{args})"
        if flows:
            row += 1
            p_cells[f"A{row}"] = f"=NPV({ref},{','.join(fmt(v) for v in flows)})"
    desc = (
        "T-2671: NPV one rounding at a time, for T-2672's fit. Sheet R holds the rates as exact doubles "
        "(A1:A4 IRR.xlsx General!L2, L3, L8 and Three Parameters!D2; A5:A9 the m9-fin-nper-npv rates "
        "0.05, 0.07, 0.1, 0.125, 0.2; A10:A20 15-digit rates searched where T-1528's NPV candidates "
        "(fitlab annuity-npv) give the most distinct doubles) and B = 1+A. Per rate: NPV(r,1), NPV(r,7), "
        "zero-padded NPV(r,0,..,0,1) for periods 2-12, two-term (-20,21), (5,-5), (-1,1+r) and three-term "
        "(1,-2,1), (-20,5,15), (3,-7,4); on L2 and D2 also the IRR flows. Generator and the candidate "
        "spread per rate: docs/research/t2671_npv_term_order.py."
    )
    return {
        "format": "rustsheet-case/v1",
        "description": desc,
        "tags": ["M9", "financial", "npv", "probe"],
        "sheets": [{"name": "R", "cells": r_cells}, {"name": "P", "cells": p_cells}],
        "expected": {},
    }


def main() -> None:
    rates = list(IRR_RATES)
    rates += [(f"m9-fin-nper-npv {r}", r, []) for r in SWEEP_RATES]
    rates += [("searched", r, []) for r in search(SEARCHED)]
    for i, (label, r, _) in enumerate(rates, 1):
        split, total = score(r)
        print(f"R!A{i}  {r!r:<24} {label:<32} cells split {split}/{len(shapes(r))}, distinct doubles {total}")
    case = build(rates)
    cells = case["sheets"][1]["cells"]  # type: ignore[index]
    print(f"{len(cells)} NPV cells")
    if "--write" in sys.argv:
        CASE.write_text(json.dumps(case, indent=2) + "\n")
        print(f"wrote {CASE}")


if __name__ == "__main__":
    main()
