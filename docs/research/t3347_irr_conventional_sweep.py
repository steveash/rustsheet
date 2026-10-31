"""T-3347: the IRR probe and Gate A sweep on conventional cash flows, with their exact roots.

Usage (from tools/, mpmath is not a tools dependency yet; T-3357 adds it):

    uv run --with mpmath python ../docs/research/t3347_irr_conventional_sweep.py

Writes, deterministically (fixed seeds), the cells of
eval/suites/probes/cases/t3347-irr-conventional-num.case.json and
eval/suites/probes-numeric/cases/t3347-irr-conventional-sweep.case.json and each case's
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


def sweep_points() -> list[tuple[str, list[float], float]]:
    rng = random.Random(33470)

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
    return points


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
