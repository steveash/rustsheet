"""T-1778: write OUT/tiny.txt ("999 x result" per line) for tinydiv.c from the tiny-x probe sidecars.

Usage: python3 mk_tiny.py OUT. Rows: kmyst-erf-mantissa columns B (ERF.PRECISE) and D (ERF), and
kmyst-erf-consecutive column B, all at x below about 1E-8.
"""

import json
import sys
from pathlib import Path

CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"


def rows(name: str, cols: str) -> list[str]:
    case = json.loads((CASES / f"{name}.case.json").read_text())["sheets"][0]["cells"]
    vals = json.loads((CASES / f"{name}.oracle.json").read_text())["sheets"]["Calc"]
    out = []
    for key, x in case.items():
        if not key.startswith("A"):
            continue
        r = key[1:]
        for col in cols:
            v = vals.get(f"{col}{r}", {}).get("v")
            if isinstance(v, float):
                out.append(f"999 {float(x).hex()} {v.hex()}")
    return out


lines = rows("kmyst-erf-mantissa", "BD") + rows("kmyst-erf-consecutive", "B")
Path(sys.argv[1], "tiny.txt").write_text("\n".join(lines) + "\n")
print(len(lines))
