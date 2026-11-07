"""Replays T-4844's counts and builds T-4888's pole probe (financial.md "IRR's second stage ignores the guess and reads
only NPV's values", "IRR's secant runs on past the pole").

Runs the engine's IRR secant in d exactly (worksheet NPV's running product with Excel's twice-rounded operators,
tools' `fitlab` exact arithmetic) without its 50-step cap, over the three t4844 probe cases, and prints:
- the guess ladder's split: guess cells equal to the default call against the cells where the replica fails or
  converges, the replica's bits where it converges, and the fixed points;
- each padded annuity's predicted underflow threshold, from the smallest 1 + r of the replica's run from 0.1;
- each annuity's |NPV(X)| and the 2^k at which it crosses the 1e-7 stop.

    cd tools && uv run python -I ../docs/research/irr-2026-11-07/replay.py          # the T-4844 counts
    cd tools && uv run python -I ../docs/research/irr-2026-11-07/replay.py pole     # T-4888's case: write or check
    cd tools && uv run python -I ../docs/research/irr-2026-11-07/replay.py stall    # T-4888's stall sweep
    cd tools && uv run python -I ../docs/research/irr-2026-11-07/replay.py stall2   # the sweep after its freeze

`pole` draws short series (3 to 6 flows, 6 significant digits, seed "T-4838/pole") whose secant from 0.1 crosses the
pole (an iterate at d >= 1), and keeps them by how two models answer: the secant run on past the pole (`secant(...,
run_on=True)`) and the engine's fallback before T-4888, Newton on P(v) = sum c_i v^i from v = 0 (plain doubles,
50 steps). It writes probes/t4838-irr-pole-runon (when missing) and its predictions file. Both depend only on the seed,
not on Excel's values, so they were fixed before the fill.

The secant here is exact (Excel's twice-rounded operators); docs/research/irr-2026-11-07/feasibility.py's is in plain
doubles for speed, so the two are kept apart.
"""

from __future__ import annotations

import json
import math
import random
import sys
from collections import Counter, defaultdict
from pathlib import Path

from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.kernels.irr_secant_d import _npv_v_running
from rustsheet_tools.fitlab.kernels.solvers import NoResultError

CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases"
GUESSES = 37  # -0.9..0.9 by 0.05
BLOCK = GUESSES + 2


def load(name: str) -> tuple[dict[str, str], dict[str, dict[str, object]]]:
    cells = json.loads((CASES / f"{name}.case.json").read_text())["sheets"][0]["cells"]
    oracle = json.loads((CASES / f"{name}.oracle.json").read_text())["sheets"]["P"]
    return cells, oracle


def flows(formula: str) -> list[float]:
    return [float(x) for x in formula[formula.index("{") + 1 : formula.index("}")].split(",")]


def secant(
    values: list[float], guess: float, cap: int = 400, *, run_on: bool = False
) -> tuple[float | None, list[float], str]:
    """The run's result (None when it fails), its iterates in d and how it ended: converged, flat, pole (an iterate at
    d >= 1 without `run_on`; with it the run goes on there), npv (out of range) or cap. A run-on result past the pole
    (v <= 0) is no result."""

    def npv(d: float) -> float:
        return _npv_v_running(values, 1.0 - d)

    path: list[float] = []
    try:
        d0 = X87.div(guess, 1 + guess)
        f0 = npv(d0)
        d1 = d0 - (1e-3 if f0 > 0 else -1e-3)
        f1 = npv(d1)
        path = [d0, d1]
        for _ in range(cap):
            if f1 == f0:
                return None, path, "flat"
            d2 = d1 - X87.div(f1 * (d1 - d0), f1 - f0)
            if d2 == 1 or (d2 > 1 and not run_on):
                return None, path, "pole"
            f2 = npv(d2)
            path.append(d2)
            if abs(d2 - d1) < 1e-7 and abs(f2) < 1e-7:
                return (X87.div(1.0, 1.0 - d2) - 1 if d2 < 1 else None), path, "converged"
            d0, f0, d1, f1 = d1, f1, d2, f2
    except NoResultError:
        return None, path, "npv"
    return None, path, "cap"


def newton_from_zero(values: list[float], cap: int = 50) -> float | None:
    """The engine's fallback before T-4888: Newton on P(v) by Horner from v = 0, plain doubles, stop at a step under
    1e-7 max(|v|, 1); the rate, or None."""
    v = 0.0
    for _ in range(cap):
        p = dp = 0.0
        for c in reversed(values):
            dp = dp * v + p
            p = p * v + c
        if dp == 0:
            return None
        nv = v - p / dp
        if not math.isfinite(nv):
            return None
        if abs(nv - v) < 1e-7 * max(abs(nv), 1):
            return 1 / nv - 1 if nv > 0 else None
        v = nv
    return None


POLE_CASE = "t4838-irr-pole-runon"
POLE_QUOTA = {"both, digits differ": 70, "run-on only": 45, "Newton only": 30, "both, same digits": 15}


def run_on(values: list[float]) -> float | None:
    result, _, _ = secant(values, 0.1, cap=200, run_on=True)
    return result if result is not None and result > -1 else None


def pole_rows() -> list[tuple[list[float], float | None, float | None, str]]:
    rng = random.Random("T-4838/pole")
    kept: defaultdict[str, list[tuple[list[float], float | None, float | None]]] = defaultdict(list)
    while any(len(kept[k]) < q for k, q in POLE_QUOTA.items()):
        n = rng.choice([3, 3, 4, 4, 5, 6])
        first = float(f"{rng.choice([1, -1]) * 10 ** rng.uniform(-1, 3):.6g}")
        values = [first] + [
            float(f"{rng.choice([1, -1]) * 10 ** rng.uniform(-1, 2):.6g}") for _ in range(n - 1)
        ]
        if all(c > 0 for c in values) or all(c < 0 for c in values):
            continue
        try:
            ending = secant(values, 0.1, cap=200)[2]
        except (OverflowError, ZeroDivisionError):
            continue
        if ending != "pole":
            continue
        ours, theirs = run_on(values), newton_from_zero(values)
        if ours is not None and theirs is not None:
            kind = "both, same digits" if ours == theirs else "both, digits differ"
        elif ours is not None:
            kind = "run-on only"
        elif theirs is not None:
            kind = "Newton only"
        else:
            continue
        if len(kept[kind]) < POLE_QUOTA[kind]:
            kept[kind].append((values, ours, theirs))
    return [(v, a, b, kind) for kind in POLE_QUOTA for v, a, b in kept[kind]]


def pole_case() -> int:
    rows = pole_rows()
    sheets = [
        {
            "name": "P",
            "cells": {
                f"A{i}": "=IRR({" + ",".join(map(repr, v)) + "})" for i, (v, _, _, _) in enumerate(rows, 1)
            },
        }
    ]
    path = CASES / f"{POLE_CASE}.case.json"
    with (CASES / f"{POLE_CASE}.predictions.tsv").open("w") as out:
        out.write("cell\tclass\trun-on secant\tnewton from v=0 (engine fallback model)\n")
        for i, (_, a, b, kind) in enumerate(rows, 1):
            out.write(f"A{i}\t{kind}\t{a!r}\t{b!r}\n")
    if path.exists():
        same = json.loads(path.read_text())["sheets"] == sheets
        print(f"{POLE_CASE}: {'up to date' if same else 'differs from the generator'}; predictions rewritten")
        return 0 if same else 1
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4838: IRR on 160 fresh short series (3 to 6 flows, seed 'T-4838/pole') whose secant from 0.1 crosses the "
            "pole (an iterate at d >= 1). Stratified by whether the run-on secant (continued past d = 1) and the "
            "engine's Newton fallback from v = 0 return, and whether their digits agree: A1:A70 both return, digits "
            "differ; A71:A115 run-on only; A116:A145 Newton only; A146:A160 both, same digits. Predictions in "
            f"{POLE_CASE}.predictions.tsv. docs/research/irr-2026-11-07/replay.py."
        ),
        "tags": ["M9", "probe", "financial", "irr"],
        "sheets": sheets,
        "expected": {},
    }
    path.write_text(json.dumps(case, indent=2) + "\n")
    print(f"wrote {path.name} and its predictions")
    return 0


STALL_CASE = "t4888-irr-stall-sweep"
STALL_PER_DECADE = 8
NO_ROOT = [
    0.2,
    -0.0001,
    1e-15,
    -1.0,
    0.1,
    0.5,
    0.2,
    1.005,
    2.5,
    123456789.123,
    -2.5,
    2.5,
    0.2,
]  # gen-m9 fec39efa A283


def stall(values: list[float], guess: float = 0.1) -> tuple[int, float] | None:
    """The first zero step (d2 = d1) whose |NPV| is 1e-7 or more, within the run's first 50 steps, of the secant from
    `guess` run on past the pole: (step, |NPV|), or None when the run never stalls so."""
    try:
        d0 = X87.div(guess, 1 + guess)
        f0 = _npv_v_running(values, 1.0 - d0)
        d1 = d0 - (1e-3 if f0 > 0 else -1e-3)
        f1 = _npv_v_running(values, 1.0 - d1)
        for step in range(50):
            if f1 == f0:
                return None
            d2 = d1 - X87.div(f1 * (d1 - d0), f1 - f0)
            if d2 == 1:
                return None
            f2 = _npv_v_running(values, 1.0 - d2)
            if abs(d2 - d1) < 1e-7 and abs(f2) < 1e-7:
                return None
            if d2 == d1:
                return step, abs(f2)
            d0, f0, d1, f1 = d1, f1, d2, f2
    except NoResultError:
        return None
    return None


def stall_rows() -> list[tuple[list[float], float, int, float]]:
    """(flows, scale, stall step, |NPV| at the stall): random series whose run from 0.1 stalls, STALL_PER_DECADE per
    decade of the largest flow from 1E-9 to 1E+14, then gen-m9 A283's no-root flows times 10^k, k = -14..0."""
    rng = random.Random("T-4888/stall")
    per: defaultdict[int, list[tuple[list[float], float, int, float]]] = defaultdict(list)
    while any(len(per[e]) < STALL_PER_DECADE for e in range(-9, 15)):
        n = rng.choice([3, 4, 6, 10, 20, 40])
        root = rng.uniform(-0.3, 0.5)
        flows = [rng.uniform(0.2, 1.0) * rng.choice([1, 1, 1, -1]) for _ in range(n - 1)]
        outlay = -sum(c / (1 + root) ** (i + 1) for i, c in enumerate(flows))
        decade = rng.randint(-9, 14)
        scale = 10.0**decade
        values = [float(f"{c * scale:.6g}") for c in [outlay, *flows]]
        if len(per[decade]) >= STALL_PER_DECADE or all(c >= 0 for c in values) or all(c <= 0 for c in values):
            continue
        hit = stall(values)
        if hit is not None:
            per[decade].append((values, scale, *hit))
    rows = [row for e in range(-9, 15) for row in per[e]]
    for k in range(-14, 1):
        values = [float(f"{c * 10.0**k:.6g}") for c in NO_ROOT]
        hit = stall(values) or (-1, math.nan)
        rows.append((values, 10.0**k, *hit))
    return rows


def stall_case() -> int:
    rows = stall_rows()
    cells = {f"A{i}": "=IRR({" + ",".join(map(repr, v)) + "})" for i, (v, _, _, _) in enumerate(rows, 1)}
    path = CASES / f"{STALL_CASE}.case.json"
    with (CASES / f"{STALL_CASE}.stalls.tsv").open("w") as out:
        out.write("cell\tscale\tstall step\t|NPV| at the stall\n")
        for i, (_, scale, step, residual) in enumerate(rows, 1):
            out.write(f"A{i}\t{scale:g}\t{step}\t{residual:.6g}\n")
    if path.exists():
        same = json.loads(path.read_text())["sheets"] == [{"name": "P", "cells": cells}]
        print(f"{STALL_CASE}: {'up to date' if same else 'differs from the generator'}")
        return 0 if same else 1
    per = STALL_PER_DECADE * 24
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            f"T-4888: fresh sweep for IRR's zero-step stall bound, drawn after the bound was frozen (seed 'T-4888/stall'). "
            f"A1:A{per}: random series of 3 to 40 flows whose secant from 0.1 reaches a zero step at |NPV| >= 1e-7 within "
            f"50 steps, {STALL_PER_DECADE} per decade of the largest flow from 1E-9 to 1E+14. A{per + 1}:A{per + 15}: "
            "gen-m9 fec39efaba9284c3 Calc!A283's no-root flows times 10^k, k = -14..0. Each stall's step and |NPV| in "
            f"{STALL_CASE}.stalls.tsv. docs/research/irr-2026-11-07/replay.py stall."
        ),
        "tags": ["M9", "probe", "financial", "irr"],
        "sheets": [{"name": "P", "cells": cells}],
        "expected": {},
    }
    path.write_text(json.dumps(case, indent=2) + "\n")
    print(f"wrote {path.name}: {len(rows)} cells")
    return 0


STALL2_CASE = "t4888-irr-stall-sweep-2"
STALL2_GUESSES = [0.1, 0.1, 0.0, -0.5, 0.5, 2.0, -0.9, 10.0]
# The frozen rule (financial_util.rs `irr_stall_stops`), predicted per cell before the fill.
FROZEN_ABSOLUTE, FROZEN_RELATIVE, FROZEN_FLOWS = 1e-4, 1e-12, (1.3e-7, 1.3e14)


def frozen_holds(residual: float, peak: float) -> bool:
    return (
        FROZEN_FLOWS[0] <= peak <= FROZEN_FLOWS[1]
        and residual < FROZEN_ABSOLUTE
        and residual < FROZEN_RELATIVE * peak
    )


def stall_point(values: list[float], guess: float) -> tuple[float, float] | None:
    """The rate and |NPV| of the first zero step at |NPV| >= 1e-7 in the first 50 steps from `guess` (run on past the
    pole), or None."""
    hit = stall(values, guess)
    if hit is None:
        return None
    d0 = X87.div(guess, 1 + guess)
    f0 = _npv_v_running(values, 1.0 - d0)
    d1 = d0 - (1e-3 if f0 > 0 else -1e-3)
    f1 = _npv_v_running(values, 1.0 - d1)
    for _ in range(hit[0] + 1):
        d2 = d1 - X87.div(f1 * (d1 - d0), f1 - f0)
        f2 = _npv_v_running(values, 1.0 - d2)
        d0, f0, d1, f1 = d1, f1, d2, f2
    return X87.div(1.0, 1.0 - d1) - 1, hit[1]


def stall2_case() -> int:
    """T-4888's second stall sweep, drawn after the two-part rule was frozen: varied guesses (-0.9 to 10), 2 to 150
    flows, scales 1E-9 to 1E+17. Writes the case and its `.predictions.tsv` (the stall point, |NPV|, largest flow, the
    frozen rule's hold-or-not and "no stall stop"'s), which depend only on the seed."""
    rng = random.Random("T-4888/stall-2")
    rows: list[tuple[list[float], float, float, float, float]] = []
    per: defaultdict[int, int] = defaultdict(int)
    while len(rows) < 240:
        n = rng.choice([2, 3, 5, 10, 20, 40, 80, 150])
        root = rng.uniform(-0.3, 0.6)
        flows = [rng.uniform(0.1, 1.0) * rng.choice([1, 1, 1, -1]) for _ in range(n - 1)]
        outlay = -sum(c / (1 + root) ** (i + 1) for i, c in enumerate(flows))
        decade = rng.randint(-9, 17)
        values = [float(f"{c * 10.0**decade:.6g}") for c in [outlay, *flows]]
        guess = rng.choice(STALL2_GUESSES)
        if per[decade] >= 10 or all(c >= 0 for c in values) or all(c <= 0 for c in values):
            continue
        try:
            point = stall_point(values, guess)
        except (NoResultError, OverflowError, ZeroDivisionError):
            continue
        if point is not None:
            per[decade] += 1
            rows.append((values, guess, point[0], point[1], max(abs(c) for c in values)))
    cells = {
        f"A{i}": "=IRR({" + ",".join(map(repr, v)) + "}" + ("" if g == 0.1 else f",{g!r}") + ")"
        for i, (v, g, _, _, _) in enumerate(rows, 1)
    }
    with (CASES / f"{STALL2_CASE}.predictions.tsv").open("w") as out:
        out.write(
            "cell\tguess\tstall rate\t|NPV| at the stall\tlargest |flow|\tfrozen rule holds\tno stall stop holds\n"
        )
        for i, (_, g, rate, residual, peak) in enumerate(rows, 1):
            out.write(
                f"A{i}\t{g!r}\t{rate!r}\t{residual:.6g}\t{peak:.6g}\t{frozen_holds(residual, peak)}\tFalse\n"
            )
    path = CASES / f"{STALL2_CASE}.case.json"
    if path.exists():
        same = json.loads(path.read_text())["sheets"] == [{"name": "P", "cells": cells}]
        print(
            f"{STALL2_CASE}: {'up to date' if same else 'differs from the generator'}; predictions rewritten"
        )
        return 0 if same else 1
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4888: second fresh sweep for IRR's zero-step stall rule, drawn after the two-part rule (|NPV| < 1e-4 and "
            "< 1e-12 of the largest flow, largest flows 1.3E-7..1.3E+14) was frozen (seed 'T-4888/stall-2'). 240 series "
            "of 2 to 150 flows, scales 1E-9 to 1E+17, guesses -0.9 to 10, each reaching a zero step at |NPV| >= 1e-7 "
            f"within 50 steps. The frozen rule's predictions in {STALL2_CASE}.predictions.tsv, committed before the "
            "fill. docs/research/irr-2026-11-07/replay.py stall2."
        ),
        "tags": ["M9", "probe", "financial", "irr"],
        "sheets": [{"name": "P", "cells": cells}],
        "expected": {},
    }
    path.write_text(json.dumps(case, indent=2) + "\n")
    print(f"wrote {path.name}: {len(rows)} cells")
    return 0


def main() -> None:
    cells, oracle = load("t4844-irr-guess-ladder")
    split: Counter[tuple[str, str]] = Counter()
    fixed = 0
    for block in range(len(cells) // BLOCK):
        top = 1 + BLOCK * block
        values = flows(cells[f"A{top}"])
        default = oracle[f"A{top}"]["v"]
        fixed += oracle[f"A{top + 1}"]["v"] == default
        for k in range(GUESSES):
            guess = round(-0.9 + 0.05 * k, 2)
            excel = oracle[f"A{top + 2 + k}"]["v"]
            result, _, _ = secant(values, guess)
            if result is None:
                split["replica fails", "Excel = default" if excel == default else "Excel other"] += 1
            else:
                split["replica converges", "Excel = replica" if excel == result else "Excel other"] += 1
    print(
        "guess ladder:", dict(split), f"fixed points equal to the default: {fixed} of {len(cells) // BLOCK}"
    )
    cells, oracle = load("t4844-irr-stage-invariance")
    for block in range(4):
        top = 1 + 15 * block
        values = flows(cells[f"A{top}"])
        x = oracle[f"A{top}"]["v"]
        residual = _npv_v_running(values, 1.0 - X87.div(x, 1 + x))
        _, path, _ = secant(values, 0.1)
        smallest = min(1 / (1 - d) for d in path)
        threshold = 1022 * math.log(2) / math.log(1 / smallest)
        print(
            f"annuity {block}: |NPV(X)| {abs(residual):.3g}, crosses 1e-7 at 2^{math.log2(1e-7 / abs(residual)):.2f};"
            f" smallest 1 + r of the run from 0.1 {smallest:.6f}, underflow threshold N {threshold:.1f}"
        )


if __name__ == "__main__":
    if sys.argv[1:] == ["pole"]:
        sys.exit(pole_case())
    if sys.argv[1:] == ["stall"]:
        sys.exit(stall_case())
    if sys.argv[1:] == ["stall2"]:
        sys.exit(stall2_case())
    main()
