"""T-2586 r5-f-combine probes (probes-numeric, excel-graph): F.DIST(x, d1, d2, FALSE) in the LN(fl(1+r)) branch of
Excel's log1p (r = min(mx/n, n/mx) > 0.375), on both sides, at small, mid and large df.  Each row: A = x as an exact
two-piece formula (w2-forensics' encoder, so no 15-digit literal truncation), B = F.DIST(A, d1, d2, FALSE); the
cells come from r3-fdensity's row_cells / case_doc.
  a     in-sample: 40 shapes x 4 runs of 40 consecutive doubles at mx/n ~ 0.5, 0.85 (side 1), 1.2, 2.2 (side 2)
  hold  hold-out, filled together with a and read once by final_score.py after freezing: 16 other shapes x 4 x 24
`rows(name)` regenerates the x values and reads Excel's from the sidecar, asserting each echo.
Usage: mkprobe.py [--write]"""

from __future__ import annotations

import json
import sys

import clib as C

F, MP = C.F, C.F.MP
R3P = C.SL.B._load("r5c_r3mkprobe", C.T / "r3-fdensity" / "mkprobe.py")
PREFIX = "t2586-r5-f-combine"
A_SHAPES = [(5, 6), (6, 5), (5, 7), (7, 5), (6, 9), (9, 6), (5, 5), (6, 6), (7, 7), (8, 8), (9, 11), (11, 9), (7, 12),
            (12, 7), (12, 12), (14, 20), (20, 14), (18, 30), (30, 18), (25, 25), (22, 40), (40, 22), (33, 33), (45, 45),
            (56, 56), (70, 90), (90, 70), (80, 80), (64, 200), (200, 64), (120, 120), (150, 250), (250, 150),
            (180, 180), (300, 100), (100, 300), (400, 400), (260, 260), (36, 140), (140, 36)]  # fmt: skip
H_SHAPES = [(5, 8), (8, 5), (10, 10), (13, 17), (17, 13), (26, 26), (35, 50), (50, 35), (75, 75), (96, 160),
            (160, 96), (220, 220), (130, 70), (70, 130), (350, 350), (11, 30)]  # fmt: skip
TS = (0.5, 0.85, 1.2, 2.2)  # target mx/n per run
SPEC = {"a": (A_SHAPES, 40, 0.0000613), "hold": (H_SHAPES, 24, 0.0000277)}


def plan(name):
    """[(run label, d1, d2, x)] in sheet-row order."""
    shapes, cnt, salt = SPEC[name]
    starts = [(d1, d2, t, t * d2 / d1 * (1 + 0.000191 * i + salt)) for i, (d1, d2, t) in
              enumerate((d1, d2, t) for d1, d2 in shapes for t in TS)]  # fmt: skip
    return [
        (f"run({d1},{d2},t{t:g})", float(d1), float(d2), x)
        for d1, d2, t, x0 in starts
        for x in MP.run(x0, 0, cnt)
    ]


def build(name):
    cells: dict = {}
    for i, (_, d1, d2, x) in enumerate(plan(name), 1):
        cells.update(R3P.row_cells(i, d1, d2, x, "hold"))  # kind "hold": columns A, B only
    note = "F.DIST(x,d1,d2,FALSE) on runs of consecutive exact doubles x with r = min(mx/n, n/mx) > 0.375"
    return R3P.case_doc(cells, f"T-2586 r5-f-combine probe {name}: {note} (column A echoes x).", "T-2586")


def rows(name):
    sh = F.sidecar(f"{PREFIX}-{name}")
    got = [
        (blk, d1, d2, x, F.val(sh, f"A{i}"), F.val(sh, f"B{i}"))
        for i, (blk, d1, d2, x) in enumerate(plan(name), 1)
    ]
    assert all(echo == x for _, _, _, x, echo, v in got if v is not None), name
    return [
        dict(src=name, blk=b, x=x, d1=d1, d2=d2, excel=v)
        for b, d1, d2, x, _, v in got
        if v is not None and v > 0
    ]


if __name__ == "__main__":
    for nm in SPEC:
        doc, path = build(nm), MP.CASES / f"{PREFIX}-{nm}.case.json"
        same = path.exists() and json.loads(path.read_text()) == doc
        print(
            nm,
            len(doc["sheets"][0]["cells"]),
            "cells",
            "exists" + (" (identical)" if same else " (DIFFERS)") if path.exists() else "new",
        )
        if "--write" in sys.argv and not path.exists():
            path.write_text(json.dumps(doc, indent=1) + "\n")
