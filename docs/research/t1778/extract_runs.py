"""Write the run files the T-1778 erf scripts read, from the probes-numeric oracle sidecars.

Usage (from the repository root): python3 docs/research/t1778/extract_runs.py OUTDIR

Each line is `run_id x erf`, both values as float.hex(). OUTDIR/runs.txt holds kmyst-erf-zsweep (runs 0-63),
kmyst-erf-zsweep2 (64-159), kmyst-erf-ladder (5000-5021), kmyst-erf-scatter (8000-8023) and kmyst-erf-zladder
(9000-9062); OUTDIR/zs3.txt holds kmyst-erf-zsweep3 (runs 0-143) and OUTDIR/ws.txt
kmyst-erf-wsweep (runs 0-191) and kmyst-erf-wsweep2 (192-503), 128 rows each; OUTDIR/wb.txt holds kmyst-erf-wbig (runs 0-79) and OUTDIR/xc.txt kmyst-erf-xcross (runs 0-16), OUTDIR/xc2.txt kmyst-erf-xcross2 (runs 0-24, 128 rows), OUTDIR/wl.txt kmyst-erf-wlong (runs 0-17, 1,024 rows), OUTDIR/wl2.txt kmyst-erf-wlong2 (runs 0-15, lengths from the case description), OUTDIR/wk.txt kmyst-erf-wknee1..4 (runs 0-3 of 32,768 rows, runs 4-8 of 24,576). Run ids number consecutive blocks of rows.
"""

from __future__ import annotations

import bisect
import json
import re
import sys
from pathlib import Path

CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"
LAYOUT = {
    "runs.txt": [("kmyst-erf-zsweep", 0, 256), ("kmyst-erf-zsweep2", 64, 256), ("kmyst-erf-ladder", 5000, 512),
                 ("kmyst-erf-scatter", 8000, 256), ("kmyst-erf-zladder", 9000, 256)],
    "zs3.txt": [("kmyst-erf-zsweep3", 0, 256)],
    "ws.txt": [("kmyst-erf-wsweep", 0, 128), ("kmyst-erf-wsweep2", 192, 128)],
    "wb.txt": [("kmyst-erf-wbig", 0, 256)],
    "xc.txt": [("kmyst-erf-xcross", 0, 256)],
    "xc2.txt": [("kmyst-erf-xcross2", 0, 128)],
    "wl.txt": [("kmyst-erf-wlong", 0, 1024)],
    "wl2.txt": [("kmyst-erf-wlong2", 0, None)],
    "wk.txt": [("kmyst-erf-wknee1", 0, 32768), ("kmyst-erf-wknee2", 2, 32768), ("kmyst-erf-wknee3", 4, 24576),
               ("kmyst-erf-wknee4", 7, 24576)],
}


def rows(name: str, base: int, run_len: int | None) -> list[str]:
    case = json.loads((CASES / f"{name}.case.json").read_text())
    cells = case["sheets"][0]["cells"]
    starts = [int(m) for m in re.findall(r"row=(\d+)", case["description"])] if run_len is None else []
    values = json.loads((CASES / f"{name}.oracle.json").read_text())["sheets"]["Calc"]
    out = []
    for r in range(1, len(cells) // 2 + 1):
        v = values.get(f"B{r}", {}).get("v")
        if isinstance(v, float) and f"A{r}" in cells:
            run = (r - 1) // run_len if run_len else bisect.bisect_right(starts, r) - 1
            out.append(f"{base + run} {float(cells[f'A{r}']).hex()} {v.hex()}")
    return out


if __name__ == "__main__":
    outdir = Path(sys.argv[1])
    for fname, parts in LAYOUT.items():
        lines = [line for part in parts for line in rows(*part)]
        (outdir / fname).write_text("\n".join(lines) + "\n")
        print(fname, len(lines))
