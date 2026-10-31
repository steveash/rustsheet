"""T-3347: the IRR probe and Gate A sweep on conventional cash flows, with their exact roots.

Usage (from tools/, mpmath is not a tools dependency yet; T-3357 adds it):

    uv run --with mpmath python ../docs/research/t3347_irr_conventional_sweep.py

Writes, deterministically (fixed seeds), the cells of
eval/suites/probes/cases/t3347-irr-conventional-num.case.json and
eval/suites/probes-numeric/cases/t3347-irr-conventional-sweep.case.json, the R4 hold-out sweep
eval/suites/probes-numeric/cases/t3347-irr-conventional-holdout.case.json (a new seed drawn after the
fit was frozen, plus a sixth stratum of long series at -0.5 to -0.1 where the 50-step split falls),
and each case's
`.roots.tsv`: the 50-digit root of the flows as written (one outlay, then positive flows, so
exactly one root with 1 + r > 0) and Excel's value from the case's oracle sidecar when it exists.
"""

from __future__ import annotations

import json
import random
from pathlib import Path
from typing import Any

from mpmath import findroot, mp, mpf, nstr

mp.dps = 60
ROOT = Path(__file__).resolve().parents[2]
SWEEP_STRATA = ("near0", "near0-neg", "near-10pct", "pos-log", "neg-log")


def number(x: float) -> str:
    text = repr(x)
    return text[:-2] if text.endswith(".0") else text


def flows(rng: random.Random, rate: float, n: int, scale: float, level: bool) -> list[float]:
    """An outlay and n positive flows (level, or uniform in (0, scale]) whose IRR is about `rate`."""
    tail = [round(scale, 2)] * n if level else [round(rng.random() * scale, 2) or 0.01 for _ in range(n)]
    pv = sum(c / (1 + rate) ** (i + 1) for i, c in enumerate(tail))
    return [-round(pv, 2), *tail]


def exact_root(values: list[float], near: float) -> Any:
    return findroot(lambda r: sum(mpf(c) / (1 + r) ** i for i, c in enumerate(values)), mpf(near))


def probe_points() -> list[tuple[str, list[float], float]]:
    rng = random.Random(3347)
    points = []
    for n in (10, 20, 40):
        for rate in (-0.07, -0.085, -0.095, -0.1, -0.105, -0.115, -0.13):
            points.append(("near-10pct", flows(rng, rate, n, 1.38 if n == 20 else 15, True), rate))
    for scale in (1e3, 1e6, 1e9):
        for rate in (3e-6, 1e-5, 3e-5, 1e-4, 3e-4, 1e-3, 1e-2):
            points.append(("near0", flows(rng, rate, 24, scale, False), rate))
    return points


def sweep_points(seed: int = 33470, long_negative: bool = False) -> list[tuple[str, list[float], float]]:
    rng = random.Random(seed)

    def log_uniform(lo: float, hi: float) -> float:
        return float(10 ** rng.uniform(lo, hi))

    samplers = {
        "near0": lambda: log_uniform(-5, -3.52),
        "near0-neg": lambda: -log_uniform(-5, -3.52),
        "near-10pct": lambda: rng.uniform(-0.13, -0.07),
        "pos-log": lambda: log_uniform(-6, 0),
        "neg-log": lambda: -log_uniform(-6, -0.3),
    }
    points = []
    for name in SWEEP_STRATA:
        for _ in range(24):
            rate = samplers[name]()
            n = int(10 ** rng.uniform(0.3, 1.78))
            scale = 10 ** rng.uniform(0, 10)
            points.append((name, flows(rng, rate, n, scale, rng.random() >= 0.7), rate))
    if long_negative:
        # Where the fitted 50-step cap splits returning series from `#NUM!` ones (sweep A111,
        # A115, A117 are -0.11 to -0.40 and need 64 to 159 steps from v = 1).
        for _ in range(24):
            rate = rng.uniform(-0.5, -0.1)
            n = int(10 ** rng.uniform(1.3, 1.78))
            scale = 10 ** rng.uniform(0, 10)
            points.append(("neg-long", flows(rng, rate, n, scale, rng.random() >= 0.7), rate))
    return points


HOLDOUT_DESCRIPTION = (
    "T-3347 hold-out sweep (decision 0188 R4.1, R4.2), drawn with a new seed after irr_conventional was frozen: "
    "the five strata of t3347-irr-conventional-sweep (A1-A120, 24 each, same samplers) and A121-A144 long series "
    "(20-60 flows) at a rate uniform in -0.5 to -0.1, where the fitted 50-step cap splits numbers from #NUM!. "
    "50-digit mpmath roots in t3347-irr-conventional-holdout.roots.tsv (docs/research/t3347_irr_conventional_sweep.py)."
)


def create(case: Path, points: list[tuple[str, list[float], float]], description: str) -> None:
    """Write a new case for `points` (no-op when it exists; `write` then checks it)."""
    if case.exists():
        return
    cells = {f"A{i + 1}": "=IRR({" + ",".join(map(number, v)) + "})" for i, (_, v, _) in enumerate(points)}
    spec = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M9", "probe", "financial", "irr", "sweep"],
        "sheets": [{"name": "P", "cells": cells}],
        "expected": {},
        "check": [f"P!{c}" for c in cells],
    }
    case.write_text(json.dumps(spec, indent=1) + "\n")


def write(case: Path, points: list[tuple[str, list[float], float]]) -> None:
    spec = json.loads(case.read_text())
    cells = {f"A{i + 1}": "=IRR({" + ",".join(map(number, v)) + "})" for i, (_, v, _) in enumerate(points)}
    if spec["sheets"][0]["cells"] != cells:
        raise SystemExit(f"{case.name}: the committed cells differ from the generator's")
    sidecar = case.with_name(case.name.replace(".case.json", ".oracle.json"))
    excel = json.loads(sidecar.read_text())["sheets"]["P"] if sidecar.exists() else {}
    rows = ["cell\tstratum\texcel\texact_root"]
    for i, (stratum, values, rate) in enumerate(points):
        value = excel.get(f"A{i + 1}", {}).get("v", "")
        shown = repr(value) if isinstance(value, float) else str(value)
        rows.append(f"A{i + 1}\t{stratum}\t{shown}\t{nstr(exact_root(values, rate), 50)}")
    case.with_name(case.name.replace(".case.json", ".roots.tsv")).write_text("\n".join(rows) + "\n")


if __name__ == "__main__":
    write(ROOT / "eval/suites/probes/cases/t3347-irr-conventional-num.case.json", probe_points())
    write(ROOT / "eval/suites/probes-numeric/cases/t3347-irr-conventional-sweep.case.json", sweep_points())
    holdout = ROOT / "eval/suites/probes-numeric/cases/t3347-irr-conventional-holdout.case.json"
    create(holdout, sweep_points(334702, long_negative=True), HOLDOUT_DESCRIPTION)
    write(holdout, sweep_points(334702, long_negative=True))
