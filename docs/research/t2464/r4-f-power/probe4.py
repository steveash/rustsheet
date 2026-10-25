"""T-2586 r4-f-power probes: F.DIST(x, d1, d2, FALSE) on runs of consecutive doubles at large shapes.
Rows: A = x (w2-forensics' exact encoder, no 15-digit literal truncation), B = F.DIST(A, d1, d2, FALSE).
  a     in-sample: 6 shapes x 4 x centres x 224 consecutive doubles
  hold  hold-out (built and filled with a, read once after the model is frozen): 8 other shapes x 4 other centres x 48
Rows are rebuilt from this generator; Excel values come from the eval sidecars.  Usage: probe4.py [--write]"""
from __future__ import annotations

import json
import sys

import plib as P

F = P.F
MP = F.MP
PREFIX = "t2586-r4-f-power"
SPEC = {
    "a": ([(100, 100), (50, 100), (100, 50), (60, 200), (50, 50), (20, 100)], (0.37, 1.9, 13.0, 100.0), 224),
    "hold": ([(70, 140), (120, 80), (90, 90), (40, 160), (150, 60), (80, 240), (64, 128), (110, 55)],
             (0.45, 2.6, 9.0, 60.0), 48),
}


def runs(name):
    shapes, xs, cnt = SPEC[name]
    out = []
    i = 0
    for d1, d2 in shapes:
        for c in xs:
            x0 = c * (1 + 0.000173 * i + 0.0000417)
            i += 1
            out += [(f"run({d1},{d2},x{c:g})", float(d1), float(d2), x) for x in MP.run(x0, 0, cnt)]
    return out


def build(name):
    cells = {}
    for r, (_, d1, d2, x) in enumerate(runs(name), 1):
        cells[f"A{r}"] = "=" + MP.enc(x)
        cells[f"B{r}"] = f"=F.DIST(A{r},{d1:g},{d2:g},FALSE)"
    assert len(cells) <= 16384, len(cells)
    return {
        "format": "rustsheet-case/v1",
        "description": f"T-2586 r4-f-power probe {name}: F.DIST(x,d1,d2,FALSE) on runs of consecutive doubles "
        "at large shapes (x by an exact two-piece formula, echoed in column A).",
        "tags": ["M9", "probe", "statistics", "f-dist", "T-2586"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }


def rows(name):
    sh = F.sidecar(f"{PREFIX}-{name}")
    out = []
    for r, (blk, d1, d2, x) in enumerate(runs(name), 1):
        xr, v = F.val(sh, f"A{r}"), F.val(sh, f"B{r}")
        if v is None or not v > 0:
            continue
        assert xr == x, (name, r, xr, x)
        out.append(dict(src=name, blk=blk, x=x, d1=d1, d2=d2, excel=v))
    return out


if __name__ == "__main__":
    tot = 0
    for name in SPEC:
        c = build(name)
        k = len(c["sheets"][0]["cells"])
        tot += k
        p = MP.CASES / f"{PREFIX}-{name}.case.json"
        print(name, k, "cells", "exists" if p.exists() else "new")
        if "--write" in sys.argv and not p.exists():
            p.write_text(json.dumps(c, indent=1) + "\n")
    print("total", tot)
