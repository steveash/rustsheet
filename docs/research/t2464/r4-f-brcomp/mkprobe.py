"""T-2586 r4-f-brcomp probes: F.DIST(x, d1, d2, FALSE) on exact-input x runs (column A = x by w2-forensics' exact
encoder, column B = F.DIST(A, d1, d2, FALSE)); r3-fdensity's row layout and readers are reused by import.
  hold  hold-out, written and filled before any model work and read once with the frozen model: 16 shapes
        (none used in r3's probes) x 3 runs of 32 consecutive doubles at x ~ 0.37, 1.9, 13 (r3's run starts).
  big   in-sample: 8 large shapes x 6 runs of 48 at x ~ 0.21, 0.6, 0.95, 1.05, 2.4, 6 (the large-shape noise).
Usage: mkprobe.py [--write]"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "r3-fdensity"))
import flib as F  # noqa: E402
import mkprobe as R3P  # noqa: E402  (r3-fdensity: runs(), row_cells())

MP = F.MP
PREFIX = "t2586-r4-f-brcomp"
HOLD_SHAPES = [
    (1, 30), (2, 7), (2, 80), (4, 9), (9, 4), (11, 11), (16, 16), (18, 64),
    (24, 24), (32, 90), (45, 12), (64, 64), (70, 150), (120, 40), (150, 150), (200, 200),
]
BIG_SHAPES = [(16, 40), (30, 30), (40, 100), (60, 60), (100, 30), (100, 200), (160, 160), (300, 300)]
BIG_XS = (0.21, 0.6, 0.95, 1.05, 2.4, 6.0)


def big_runs():
    out = []
    i = 0
    for d1, d2 in BIG_SHAPES:
        for c in BIG_XS:
            x0 = c * (1 + 0.000211 * i + 0.0000417)
            i += 1
            out += [(f"run({d1},{d2},x{c:g})", float(d1), float(d2), x) for x in MP.run(x0, 0, 48)]
    return out


PROBES = {"hold": lambda: R3P.runs(HOLD_SHAPES, 32), "big": big_runs}


def build(name):
    cells = {}
    for r, (_, d1, d2, x) in enumerate(PROBES[name](), 1):
        cells.update(R3P.row_cells(r, d1, d2, x, "hold"))
    return R3P.case_doc(cells, f"T-2586 r4-f-brcomp probe {name}: F.DIST(x,d1,d2,FALSE) on runs of consecutive exact "
                        "doubles x (column A echoes x).", "T-2586")


def rows(name):
    """Probe rows with Excel's value in 'excel' (x checked against its echo in column A)."""
    sh = F.sidecar(f"{PREFIX}-{name}")
    out = []
    for r, (blk, d1, d2, x) in enumerate(PROBES[name](), 1):
        xr, v = F.val(sh, f"A{r}"), F.val(sh, f"B{r}")
        if v is None or not v > 0:
            continue
        assert xr == x, (name, r, xr, x)
        out.append(dict(src=name, blk=blk, x=x, d1=d1, d2=d2, excel=v))
    return out


if __name__ == "__main__":
    for name in PROBES:
        c = build(name)
        p = MP.CASES / f"{PREFIX}-{name}.case.json"
        print(name, len(c["sheets"][0]["cells"]), "cells", "exists" if p.exists() else "new")
        if "--write" in sys.argv and not p.exists():
            p.write_text(json.dumps(c, indent=1) + "\n")
