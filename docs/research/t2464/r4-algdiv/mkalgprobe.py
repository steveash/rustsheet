"""T-3383 r4-algdiv: build eval/suites/probes-numeric/cases/t3383-r4-algdiv-<name>.case.json.

Item 2 of T-3383 (algdiv for b0 >= 8): b sweeps at a fixed a < 1, so the density's lnB = GL(a) + algdiv(a, b) can be
pinned per b and its offset from the accurate value read as a function of b (a smooth curve is an approximation
error, a scatter is a storage pattern). Rows: A = x, B = a, C = b (exact two-piece literals, read back),
D = BETA.DIST(A,B,C,FALSE), r3-m1-residual's layout so its loaders and model M2 apply unchanged.
  in-sample `a`: a = 0.5 and 0.25; b = 24 geometric points on [8.25, 5000] plus 16 fine steps 16 + k/8.
  hold-out `hold`: a = 0.75 and 0.4; b = 24 different geometric points plus 16 fine steps 40 + k/4.
  x: 50 per (a, b), r3-m1-residual's `xs_for` sampler.  Usage: mkalgprobe.py a|hold"""

import json
import random
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r3-m1-residual"))
import m1lib as M  # noqa: E402
from mkprobe import xs_for  # noqa: E402

SPECS = {
    "a": dict(seed=3383, a=(0.5, 0.25), lo=8.25, hi=5000.0, fine=(16.0, 0.125)),
    "hold": dict(seed=8333, a=(0.75, 0.4), lo=8.6, hi=7000.0, fine=(40.0, 0.25)),
}
PER_B = 50


def bs(sp):
    geo = [float(f"{sp['lo'] * (sp['hi'] / sp['lo']) ** (k / 23):.6g}") for k in range(24)]
    f0, st = sp["fine"]
    return geo + [f0 + k * st for k in range(16)]


def rows_of(name):
    sp = SPECS[name]
    rng = random.Random(sp["seed"])
    return [(x, a, b) for a in sp["a"] for b in bs(sp) for x in xs_for(a, b, PER_B, rng)]


def main(name):
    rows = rows_of(name)
    assert len(rows) <= 4096, len(rows)
    cells = {}
    for i, (x, a, b) in enumerate(rows):
        r = i + 1
        cells[f"A{r}"] = "=" + M.lit(x)
        cells[f"B{r}"] = "=" + M.lit(a)
        cells[f"C{r}"] = "=" + M.lit(b)
        cells[f"D{r}"] = f"=BETA.DIST(A{r},B{r},C{r},FALSE)"
    cid = f"t3383-r4-algdiv-{name}"
    case = {
        "format": "rustsheet-case/v1",
        "description": f"T-3383 r4-algdiv probe {name}: BETA.DIST(x,a,b,FALSE) at fixed a < 1 over b sweeps (b > 8), "
        "pinning the density's algdiv(a, b) per b; x, a, b are exact two-piece literals read back in A:C",
        "tags": ["M9", "probe", "statistics", "T-3383", "incomplete-beta"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    (M.SUITE / f"{cid}.case.json").write_text(json.dumps(case, indent=1) + "\n")
    (HERE / f"probe_{name}.meta.json").write_text(json.dumps({"case": cid, "rows": rows}) + "\n")
    print(cid, len(rows), "rows", len(cells), "cells")


if __name__ == "__main__":
    main(sys.argv[1])
