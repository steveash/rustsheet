"""T-3406: where Excel's IRR switches between a number and `#NUM!` on conventional flows.

Usage (from tools/, mpmath is not a tools dependency yet):

    uv run --with mpmath python ../docs/research/t3406_irr_num_boundary.py

Writes eval/suites/probes/cases/t3406-irr-num-boundary.case.json (when missing; otherwise checks
it) and its `.roots.tsv` (Excel's value from the oracle sidecar, when filled, and the 50-digit
root). Two families:
- A1-A96: level annuities, an outlay then n flows of 1 (n = 20, 30, 40, 60; 24 cells each), the
  outlay rounded to 2 decimals so the root is near r = -0.02, -0.04, ..., -0.48;
- A97-A144: the flows of t3347-irr-conventional-holdout A26, A70, A142 and A61 (12 cells each),
  the outlay scaled by 0.97 to 1.03 so the root moves around the hold-out cell's.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from mpmath import findroot, mp, mpf, nstr

mp.dps = 60
ROOT = Path(__file__).resolve().parents[2]
CASE = ROOT / "eval/suites/probes/cases/t3406-irr-num-boundary.case.json"
HOLDOUT = ROOT / "eval/suites/probes-numeric/cases/t3347-irr-conventional-holdout.case.json"
DESCRIPTION = (
    "T-3406: where IRR on conventional flows switches between a number and #NUM! where the secant fails. "
    "A1-A96 an outlay then n flows of 1 (n = 20, 30, 40, 60 in blocks of 24) with the root near -0.02, -0.04, ..., "
    "-0.48; A97-A144 the flows of t3347-irr-conventional-holdout A26, A70, A142 and A61 (blocks of 12) with the "
    "outlay scaled by 0.97 to 1.03. 50-digit roots in t3406-irr-num-boundary.roots.tsv "
    "(docs/research/t3406_irr_num_boundary.py)."
)
# Hold-out cells whose flows the second family perturbs, from t3347-irr-conventional-holdout.
SEEDS = ("A26", "A70", "A142", "A61")


def number(x: float) -> str:
    text = repr(x)
    return text[:-2] if text.endswith(".0") else text


def level(n: int, rate: float) -> list[float]:
    pv = sum(1 / (1 + rate) ** (i + 1) for i in range(n))
    return [-round(pv, 2), *([1.0] * n)]


def holdout_flows() -> dict[str, list[float]]:
    cells = json.loads(HOLDOUT.read_text())["sheets"][0]["cells"]
    return {cell: [float(x) for x in cells[cell][len("=IRR({") : -2].split(",")] for cell in SEEDS}


def points() -> list[tuple[str, list[float]]]:
    out = []
    for n in (20, 30, 40, 60):
        out.extend((f"level-n{n}", level(n, -0.02 * (k + 1))) for k in range(24))
    for cell, flows in holdout_flows().items():
        for k in range(12):
            factor = 0.97 + 0.06 * k / 11
            out.append((f"holdout-{cell}", [round(flows[0] * factor, 2), *flows[1:]]))
    return out


def exact_root(values: list[float]) -> Any:
    npv = lambda r: sum(mpf(c) / (1 + r) ** i for i, c in enumerate(values))  # noqa: E731
    # One sign change: the root is the unique r > -1; bisect on 1 + r in (0, 2) before polishing.
    lo, hi = mpf("1e-6"), mpf(2)
    for _ in range(200):
        mid = (lo + hi) / 2
        if (npv(mid - 1) > 0) == (npv(hi - 1) > 0):
            hi = mid
        else:
            lo = mid
    return findroot(npv, (lo + hi) / 2 - 1)


def main() -> None:
    pts = points()
    cells = {f"A{i + 1}": "=IRR({" + ",".join(map(number, v)) + "})" for i, (_, v) in enumerate(pts)}
    if not CASE.exists():
        spec = {
            "format": "rustsheet-case/v1",
            "description": DESCRIPTION,
            "tags": ["M9", "probe", "financial", "irr"],
            "sheets": [{"name": "P", "cells": cells}],
            "expected": {},
            "check": [f"P!{c}" for c in cells],
        }
        CASE.write_text(json.dumps(spec, indent=1) + "\n")
    elif json.loads(CASE.read_text())["sheets"][0]["cells"] != cells:
        raise SystemExit(f"{CASE.name}: the committed cells differ from the generator's")
    sidecar = CASE.with_name(CASE.name.replace(".case.json", ".oracle.json"))
    excel = json.loads(sidecar.read_text())["sheets"]["P"] if sidecar.exists() else {}
    rows = ["cell\tfamily\texcel\texact_root"]
    for i, (family, values) in enumerate(pts):
        value = excel.get(f"A{i + 1}", {}).get("v", "")
        shown = repr(value) if isinstance(value, float) else str(value)
        rows.append(f"A{i + 1}\t{family}\t{shown}\t{nstr(exact_root(values), 50)}")
    CASE.with_name(CASE.name.replace(".case.json", ".roots.tsv")).write_text("\n".join(rows) + "\n")


if __name__ == "__main__":
    main()
