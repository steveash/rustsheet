"""T-2586 r4-f-small-df probes: F.DIST(x, d1, d2, FALSE) at small df, on runs of consecutive exact doubles x
(column A = x by w2-forensics' exact encoder, so no 15-digit literal truncation; column B = F.DIST(A,d1,d2,FALSE)).
  a     in-sample: d1 in {1,2,3,4} x d2 in {1,2,3,5,10,50,100} and the swaps (47 shapes) x 4 runs of 32 at
        mx/n ~ 0.12, 0.62, 1.61, 8.3 (side 1 series, side 1 log, side 2 log, side 2 series). For the 14 shapes with
        a df of 2, also C = POWER(1+d1*A/d2,-(d1+d2)/2) and D = POWER(1+d2/(d1*A),-(d1+d2)/2) (closed-form tests).
  hold  hold-out (built and filled with a, read once with the frozen model): 14 other small-df shapes x 4 runs of 24.
Rows are rebuilt from this generator; Excel values come from the eval sidecars (r3-fdensity's readers).
Usage: mkprobe.py [--write]"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "r3-fdensity"))
import flib as F  # noqa: E402

MP = F.MP
PREFIX = "t2586-r4-f-small-df"
_D1 = (1, 2, 3, 4)
_D2 = (1, 2, 3, 5, 10, 50, 100)
A_SHAPES = sorted({(m, n) for m in _D1 for n in _D2} | {(n, m) for m in _D1 for n in _D2})
H_SHAPES = [(1, 6), (1, 25), (1, 200), (2, 6), (2, 15), (2, 200), (6, 2), (15, 2), (200, 2), (25, 1), (8, 1),
            (3, 7), (7, 3), (4, 30)]  # fmt: skip
TS = (0.12, 0.62, 1.61, 8.3)  # target mx/n per run


def runs(shapes, cnt, salt):
    out = []
    i = 0
    for d1, d2 in shapes:
        for t in TS:
            x0 = t * d2 / d1 * (1 + 0.000173 * i + salt)
            i += 1
            out += [(f"run({d1},{d2},t{t:g})", float(d1), float(d2), x) for x in MP.run(x0, 0, cnt)]
    return out


PROBES = {"a": lambda: runs(A_SHAPES, 32, 0.0000529), "hold": lambda: runs(H_SHAPES, 24, 0.0000311)}


def row_cells(r, d1, d2, x, name):
    d1s, d2s, hs = f"{d1:g}", f"{d2:g}", f"{(d1 + d2) / 2:g}"
    A = f"A{r}"
    c = {A: "=" + MP.enc(x), f"B{r}": f"=F.DIST({A},{d1s},{d2s},FALSE)"}
    if name == "a" and 2 in (d1, d2):
        c[f"C{r}"] = f"=POWER(1+{d1s}*{A}/{d2s},-{hs})"
        c[f"D{r}"] = f"=POWER(1+{d2s}/({d1s}*{A}),-{hs})"
    return c


def build(name):
    cells = {k: v for r, (_, d1, d2, x) in enumerate(PROBES[name](), 1) for k, v in row_cells(r, d1, d2, x, name).items()}
    assert len(cells) <= 16384, len(cells)
    desc = (f"T-2586 r4-f-small-df probe {name}: F.DIST(x,d1,d2,FALSE) at small df on runs of consecutive exact doubles "
            "x (column A echoes x; C, D = POWER closed-form pieces at df 2).")
    return dict(format="rustsheet-case/v1", description=desc, tags=["M9", "probe", "statistics", "f-dist", "T-2586"],
                sheets=[{"name": "Calc", "cells": cells}], expected={})


def rows(name):
    """Probe rows with Excel's value in 'excel' (x checked against its echo in column A), 'pc'/'pd' the POWERs."""
    sh = F.sidecar(f"{PREFIX}-{name}")
    got = lambda col, r: F.val(sh, f"{col}{r}")  # noqa: E731
    out = [dict(src=name, blk=blk, x=x, d1=d1, d2=d2, excel=got("B", r), pc=got("C", r), pd=got("D", r), echo=got("A", r))
           for r, (blk, d1, d2, x) in enumerate(PROBES[name](), 1)]
    out = [o for o in out if o["excel"] is not None and o["excel"] > 0]
    bad = [o for o in out if o["echo"] != o["x"]]
    assert not bad, bad[:3]
    return out


if __name__ == "__main__":
    for name, path in ((k, MP.CASES / f"{PREFIX}-{k}.case.json") for k in PROBES):
        case = build(name)
        print(name, len(case["sheets"][0]["cells"]), "cells", "exists" if path.exists() else "new")
        if "--write" in sys.argv and not path.exists():
            path.write_text(json.dumps(case, indent=1) + "\n")
