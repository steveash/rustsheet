"""T-3726: how far Excel's IMSIN and IMCOS parts are from 50-digit math on the fresh sweep.

For decision 0188 R3 (G = 1e-4): prints, per part, Excel's relative error against the 50-digit value
of the cell's argument (the 15-digit text read as a double), and the count above 1e-9 and above G.
The engine matches Excel's text in every cell (Gate A envelope: equal to Excel at 15 digits).

    cd tools && uv run python ../docs/research/t3726/excel_vs_math.py
"""

from __future__ import annotations

import json
import re
from pathlib import Path

import mpmath

CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"
NAME = "t3726-imsin-imcos-near-half-pi-sweep"
CALL = re.compile(r'^=(IMSIN|IMCOS)\("([^"]+)"\)$')
PART = re.compile(r"^([+-]?[0-9.]+(?:E[+-]?\d+)?)([+-][0-9.]+(?:E[+-]?\d+)?)i$")


def parts(text: str) -> tuple[float, float]:
    m = PART.match(text)
    if m is None:
        raise ValueError(text)
    return float(m[1]), float(m[2])


def main() -> None:
    cells = json.loads((CASES / f"{NAME}.case.json").read_text())["sheets"][0]["cells"]
    oracle = json.loads((CASES / f"{NAME}.oracle.json").read_text())
    values = oracle["sheets"]["S"]
    mpmath.mp.dps = 50
    above_floor = above_g = total = 0
    rows = []
    for ref, formula in cells.items():
        m = CALL.match(formula)
        excel = values.get(ref, {}).get("v")
        if m is None or not isinstance(excel, str):
            continue
        x, y = parts(m[2])
        z = (mpmath.sin if m[1] == "IMSIN" else mpmath.cos)(mpmath.mpc(x, y))
        for name, e, t in zip(("re", "im"), parts(excel), (z.real, z.imag), strict=True):
            total += 1
            rel = abs((mpmath.mpf(e) - t) / t) if t != 0 else mpmath.mpf(abs(e))
            above_floor += rel > 1e-9
            if rel > 1e-4:
                above_g += 1
                rows.append(f"S!{ref} {name} {formula} excel {e!r} math {mpmath.nstr(t, 15)} rel {mpmath.nstr(rel, 3)}")
    print(f"{total} parts: {above_floor} off the math by more than 1e-9, {above_g} by more than G = 1e-4")
    print("\n".join(sorted(rows)))


if __name__ == "__main__":
    main()
