"""verify-r5-f-combine: a fresh hold-out for the frozen F-density models, at shapes no probes-numeric case or
cells.tsv row has used. F.DIST(x, d1, d2, FALSE) on runs of 24 consecutive doubles in the LN(fl(1+r)) branch of
Excel's log1p, at target mx/n = 0.62, 0.93 (side 1) and 1.37, 2.45 (side 2); x is written as an exact
=(H*2^-e1+L*2^-e2) formula (no literal truncation), echoed in column A.
Usage: mkhold.py [--write]   (refuses to overwrite)"""

from __future__ import annotations

import json
import math
import re
import sys

import vparse as VP

CASE = "t2586-verify-r5-f-combine-hold"
CANDIDATES = [(6, 13), (13, 6), (15, 33), (33, 15), (27, 61), (53, 53), (44, 110), (110, 44),
              (166, 166), (190, 310), (310, 190)]  # fmt: skip
TS = (0.62, 0.93, 1.37, 2.45)
RUN = 24


def used_shapes() -> set:
    used = set()
    pat = re.compile(r"F\.DIST\([^,]+,\s*([0-9.]+)\s*,\s*([0-9.]+)\s*,")
    for p in VP.CASES.glob("*.case.json"):
        if p.name.startswith(CASE):
            continue
        used |= {(int(float(a)), int(float(b))) for a, b in pat.findall(p.read_text())}
    for line in (VP.ROOT / "docs/research/t2464/cells.tsv").read_text().splitlines()[1:]:
        f = line.split("\t")
        if f[0].startswith("F.DIST"):
            a = f[3].split("|")
            try:
                used.add((int(float(a[1])), int(float(a[2]))))
            except (IndexError, ValueError):
                pass
    return used


def encode(x: float) -> str:
    man, ex = math.frexp(x)
    M, E = int(man * 2**53), ex - 53  # x = M * 2^E, M < 2^53
    hi, lo = M >> 26, M & (2**26 - 1)
    assert E + 26 < 0 and E < 0
    f = f"=({hi}*2^-{-(E + 26)}+{lo}*2^-{-E})"
    assert VP.two_piece(f) == x
    return f


def plan():
    out = []
    for i, (d1, d2) in enumerate(SHAPES):
        for j, t in enumerate(TS):
            x = t * d2 / d1 * (1 + 0.000237 * (4 * i + j) + 0.0000391)
            for _ in range(RUN):
                out.append((d1, d2, x))
                x = math.nextafter(x, math.inf)
    return out


used = used_shapes()
SHAPES = [s for s in CANDIDATES if s not in used]

if __name__ == "__main__":
    print("dropped (used):", [s for s in CANDIDATES if s in used], "kept:", SHAPES)
    cells = {}
    for k, (d1, d2, x) in enumerate(plan(), 1):
        cells[f"A{k}"] = encode(x)
        cells[f"B{k}"] = f"=F.DIST(A{k},{d1},{d2},FALSE)"
    doc = {
        "format": "rustsheet-case/v1",
        "description": "T-2586 verify-r5-f-combine fresh hold-out: F.DIST(x,d1,d2,FALSE) on runs of consecutive "
        "exact doubles x at unused shapes, r = min(mx/n, n/mx) > 0.375 (column A echoes x).",
        "tags": ["M9", "probe", "statistics", "f-dist", "T-2586"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    print(len(cells), "cells")
    path = VP.CASES / f"{CASE}.case.json"
    if "--write" in sys.argv:
        assert not path.exists(), path
        path.write_text(json.dumps(doc, indent=1) + "\n")
        print("wrote", path)
