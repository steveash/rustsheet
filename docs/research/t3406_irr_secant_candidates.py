"""T-3406: score candidate models of IRR's type (number, `#NUM!`, `#DIV/0!`) on conventional flows.

Usage (from the repo root; plain Python, no dependencies):

    python3 docs/research/t3406_irr_secant_candidates.py [extra .case.json ...]

A double-precision replica (not Excel's twice-rounded operators, which the engine's `irr_npv`
uses) of IRR's secant in d = 1 - v from guess 0.1 (`irr_secant`), scored on
probes/t3406-irr-num-boundary, probes-numeric/t3347-irr-conventional-holdout and any extra cases
given (T-3347's probe and sweep live on its branch, PR #2529). Prints the type agreement for the
secant under a 50, 100 and 200-step cap, stopping on a flat step or carrying on with a Newton step,
and the guesses from which the secant converges on each cell where the guess-0.1 run fails.
All of them are refuted (financial.md "IRR on conventional flows where the secant fails").
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CASES = [
    ROOT / "eval/suites/probes/cases/t3406-irr-num-boundary.case.json",
    ROOT / "eval/suites/probes-numeric/cases/t3347-irr-conventional-holdout.case.json",
]
GUESSES = (-0.9, -0.75, -0.5, -0.3, -0.2, -0.15, -0.1, -0.05, -0.01, 0, 0.01, 0.05, 0.2, 0.3, 0.5, 1, 2)


def npv(c: list[float], d: float) -> float | None:
    """NPV at v = 1 - d with a running discount product, `None` where it is not finite."""
    v = 1.0 - d
    if v == 0:
        return None
    base, total, disc = 1.0 / v, 0.0, 1.0
    for i, x in enumerate(c):
        if i:
            disc *= base
        if disc == 0 or not math.isfinite(disc):
            return None
        total += x / disc
    return total if math.isfinite(total) else None


def dnpv(c: list[float], d: float) -> float:
    v = 1.0 - d
    try:
        return sum(-i * x * v ** (i - 1) for i, x in enumerate(c) if i)
    except OverflowError:
        return math.inf


def next_d(c: list[float], d0: float, f0: float, d1: float, f1: float, *, on_flat: str) -> float | None:
    """The secant's next iterate, a Newton step on a flat secant when `on_flat` is "newton", or
    `None` where the run fails (a flat step under "stop", a zero or infinite slope, d >= 1)."""
    if f1 != f0:
        d2 = d1 - f1 * (d1 - d0) / (f1 - f0)
    elif on_flat == "newton" and (g := dnpv(c, d1)) != 0 and math.isfinite(g):
        d2 = d1 - f1 / g
    else:
        return None
    return d2 if math.isfinite(d2) and d2 < 1.0 else None


def secant(c: list[float], guess: float, cap: int, on_flat: str) -> str:
    """'num', 'NUM' or 'DIV' (an NPV without a finite value) for one run."""
    largest = max(abs(x) for x in c)
    d0 = guess / (1 + guess)
    f0 = npv(c, d0)
    d1 = d0 - (1e-3 if f0 is not None and f0 > 0 else -1e-3)
    f1 = npv(c, d1)
    for _ in range(cap):
        if f0 is None or f1 is None:
            return "DIV"
        d2 = next_d(c, d0, f0, d1, f1, on_flat=on_flat)
        if d2 is None:
            return "NUM"
        f2 = npv(c, d2)
        stalled = f2 is not None and d2 == d1 and abs(f2) < 1e-7 * largest
        if f2 is not None and abs(d2 - d1) < 1e-7 and (abs(f2) < 1e-7 or stalled):
            return "num"
        d0, f0, d1, f1 = d1, f1, d2, f2
    return "DIV" if f1 is None else "NUM"


def cells(case: Path) -> list[tuple[str, list[float], str]]:
    spec = json.loads(case.read_text())["sheets"][0]["cells"]
    rows = case.with_name(case.name.replace(".case.json", ".roots.tsv")).read_text().splitlines()[1:]
    excel = {r.split("\t")[0]: r.split("\t")[2] for r in rows}
    out = []
    for cell, formula in spec.items():
        value = excel[cell]
        kind = "DIV" if value == "#DIV/0!" else "NUM" if value.startswith("#") else "num"
        out.append(
            (
                f"{case.name.removesuffix('.case.json')}!{cell}",
                [float(x) for x in formula[6:-2].split(",")],
                kind,
            )
        )
    return out


def main() -> None:
    data = [row for case in [*CASES, *map(Path, sys.argv[1:])] for row in cells(case)]
    for on_flat in ("stop", "newton"):
        for cap in (50, 100, 200):
            agree = sum(secant(c, 0.1, cap, on_flat) == kind for _, c, kind in data)
            print(f"secant cap {cap}, on a flat step {on_flat}: {agree} of {len(data)}")
    print("guesses:", GUESSES)
    for name, c, kind in data:
        if kind != "DIV" and secant(c, 0.1, 50, "stop") != "num":
            converges = "".join("o" if secant(c, g, 50, "stop") == "num" else "." for g in GUESSES)
            print(kind, converges, name)


if __name__ == "__main__":
    main()
