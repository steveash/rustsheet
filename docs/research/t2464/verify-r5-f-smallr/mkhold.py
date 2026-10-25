"""verify-r5-f-smallr: build the fresh hold-out t2586-verify-r5-f-smallr-hold (own encoder, own shapes).
10 shapes no earlier F.DIST case uses (checked against every probes-numeric case with vparse) x 2 runs x 48
consecutive doubles, all with r = min(fl(mx)/n, n/fl(mx)) <= 0.375.  A = x as '=(hi*2^k+lo*2^j)' (exact), B = F.DIST.
Usage: mkhold.py [--write]"""
from __future__ import annotations

import json
import math
import sys

import vparse as V
import vscore as S

ID = "t2586-verify-r5-f-smallr-hold"
SHAPES = [(52, 52), (34, 78), (78, 34), (104, 104), (62, 124), (124, 62), (230, 46), (46, 230), (58, 58), (86, 142)]
RUNS = [(1, 0.12), (2, 0.27), (1, 0.33), (2, 0.09)]
CNT = 48


def enc(x: float) -> str:
    mant, ex = math.frexp(x)
    M = int(mant * 2**53)
    e = ex - 53
    hi, lo = M >> 26, M & ((1 << 26) - 1)
    return f"=({hi}*2^{e + 26}+{lo}*2^{e})"


def build():
    used = set()
    for cid in V.all_case_ids():
        if cid != ID:
            used |= {(m, n) for _, _, m, n, _ in V.rows(cid)[0]}
    cells, i, row = {}, 0, 1
    for m, n in SHAPES:
        assert (float(m), float(n)) not in used, (m, n)
        for k in range(2):
            side, r = RUNS[(i + k) % 4]
            x = r * n / m if side == 1 else n / (m * r)
            for _ in range(CNT):
                s, rr = S.side_r(x, m, n)
                assert s == side and rr <= 0.375 and V.eval_formula(enc(x)) == x
                cells[f"A{row}"] = enc(x)
                cells[f"B{row}"] = f"=F.DIST(A{row},{m},{n},FALSE)"
                row += 1
                x = math.nextafter(x, math.inf)
        i += 1
    return {"format": "rustsheet-case/v1",
            "description": "T-2586 verify-r5-f-smallr fresh hold-out: F.DIST(x,d1,d2,FALSE) on runs of 48 consecutive "
                           "exact doubles x at 10 unused shapes, r = min(mx/n, n/mx) <= 0.375 (column A echoes x).",
            "tags": ["M9", "probe", "statistics", "f-dist", "T-2586"],
            "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}


if __name__ == "__main__":
    c = build()
    print(len(c["sheets"][0]["cells"]), "cells")
    p = V.CASES / f"{ID}.case.json"
    if "--write" in sys.argv:
        p.write_text(json.dumps(c, indent=1) + "\n")
        print("wrote", p)
