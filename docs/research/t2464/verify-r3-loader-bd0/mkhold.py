"""Verifier's own hold-out: t2513-verify-r3-loader-bd0-hold (1,920 cells), built before scoring the frozen model.
15 blocks of 128 consecutive doubles on 5 shapes absent from every earlier probe, at p-side ratios
v = (k - n x)/(k + n x) chosen so that 8 blocks take the series on both sides, 5 the log form on both sides
(one side beyond ratio 2), 2 mixed.  Pre-registered predictions: r3 > r3-ph on ss blocks; r3 > r3-ld on ll blocks;
r3 > null everywhere.  Usage: mkhold.py [--write]"""
from __future__ import annotations

import json
import math
import sys

import vdata as V

CID = "t2513-verify-r3-loader-bd0-hold"
PLAN = [((17.5, 33.5), (-0.40, -0.12, 0.13)), ((9.7, 21.3), (-0.45, 0.08, 0.40)),
        ((45.5, 30.5), (0.40, 0.08, -0.12)), ((4.75, 11.25), (-0.45, 0.13, -0.12)),
        ((70.3, 50.9), (0.45, 0.08, 0.13))]
CNT = 128


def enc(x: float) -> str:
    """Exact formula text: x = H 2^(e+26) + L 2^e with integer H, L < 2^27."""
    m, e = math.frexp(x)
    mant = int(m * 2**53)
    hi, lo = mant >> 26, mant & ((1 << 26) - 1)
    return f"({hi}*2^{e - 27}+{lo}*2^{e - 53})"


def build():
    cells = {}
    j = 0
    for s, ((a, b), vs) in enumerate(PLAN):
        for t, v in enumerate(vs):
            x = (a - 1) * (1 - v) / (1 + v) / (a + b - 2) * (1 + 0.000173 * (3 * s + t) + 0.0000419)
            for _ in range(CNT):
                j += 1
                f = f"=BETA.DIST({enc(x)},{a!r},{b!r},FALSE)"
                assert V.parse_call(f)[:3] == (x, a, b), f
                cells[f"A{j}"] = f
                x = math.nextafter(x, 2)
    return {"format": "rustsheet-case/v1",
            "description": "T-2513 verifier hold-out for r3-loader-bd0: BETA.DIST pdf runs of 128 consecutive "
                           f"doubles, shapes and p-side ratios {PLAN}",
            "tags": ["M9", "probe", "statistics", "T-2513", "incomplete-beta"],
            "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}


if __name__ == "__main__":
    c = build()
    p = V.CASES / f"{CID}.case.json"
    print(CID, len(c["sheets"][0]["cells"]), "exists" if p.exists() else "")
    if "--write" in sys.argv:
        assert not p.exists()
        p.write_text(json.dumps(c, indent=1) + "\n")
