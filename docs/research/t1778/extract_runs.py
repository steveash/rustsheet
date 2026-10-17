"""Write the run files the T-1778 erf scripts read, from the probes-numeric oracle sidecars.

Usage (from the repository root): python3 docs/research/t1778/extract_runs.py OUTDIR

Each line is `run_id x erf`, both values as float.hex(). OUTDIR/runs.txt holds kmyst-erf-zsweep (runs 0-63),
kmyst-erf-zsweep2 (64-159), kmyst-erf-ladder (5000-5021), kmyst-erf-scatter (8000-8023) and kmyst-erf-zladder
(9000-9062); OUTDIR/zs3.txt holds kmyst-erf-zsweep3 (runs 0-143) and OUTDIR/ws.txt
kmyst-erf-wsweep (runs 0-191) and kmyst-erf-wsweep2 (192-503), 128 rows each. Run ids number consecutive blocks of rows.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"
LAYOUT = {
    "runs.txt": [("kmyst-erf-zsweep", 0, 256), ("kmyst-erf-zsweep2", 64, 256), ("kmyst-erf-ladder", 5000, 512),
                 ("kmyst-erf-scatter", 8000, 256), ("kmyst-erf-zladder", 9000, 256)],
    "zs3.txt": [("kmyst-erf-zsweep3", 0, 256)],
    "ws.txt": [("kmyst-erf-wsweep", 0, 128), ("kmyst-erf-wsweep2", 192, 128)],
}


def rows(name: str, base: int, run_len: int) -> list[str]:
    cells = json.loads((CASES / f"{name}.case.json").read_text())["sheets"][0]["cells"]
    values = json.loads((CASES / f"{name}.oracle.json").read_text())["sheets"]["Calc"]
    out = []
    for r in range(1, len(cells) // 2 + 1):
        v = values.get(f"B{r}", {}).get("v")
        if isinstance(v, float) and f"A{r}" in cells:
            out.append(f"{base + (r - 1) // run_len} {float(cells[f'A{r}']).hex()} {v.hex()}")
    return out


if __name__ == "__main__":
    outdir = Path(sys.argv[1])
    for fname, parts in LAYOUT.items():
        lines = [line for part in parts for line in rows(*part)]
        (outdir / fname).write_text("\n".join(lines) + "\n")
        print(fname, len(lines))
