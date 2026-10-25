"""Verifier's fresh hold-out (T-2586 verify-r5-f-bigconst): F.DIST(x,d1,d2,FALSE) at large shapes that no earlier
probe used (checked against every F.DIST call in eval/suites/*/cases at build time).
Per shape: 5 consecutive doubles around x = 1, 5 around fl(d2/d1) (d1 != d2) and 2 seeded random x in [0.3, 3]
(d1 = d2: 7 random). Column A = x as an exact formula (hi*2^e1 + lo*2^e2, both < 2^27, so 15-digit literal
truncation cannot touch them), column B = the call.
Usage: vmkhold.py [--write]"""

from __future__ import annotations

import glob
import json
import math
import random
import re
import sys

from vparse import CASES, xval

CASE = "t2586-verify-r5-f-bigconst-hold"
D = (122, 134, 158, 264, 370, 450, 777, 1500)
MIXED = ((19, 264), (264, 19), (23, 777), (777, 23), (29, 134), (134, 29), (45, 450), (450, 45))


def enc(x: float) -> str:
    m, e = math.frexp(x)
    M, E = int(m * 2**53), e - 53
    hi, lo = M >> 26, M & (2**26 - 1)
    s = f"=({hi}*2^{E + 26}+{lo}*2^{E})" if lo else f"={hi}*2^{E + 26}"
    assert xval(s) == x, (x, s)
    return s


def run(c: float, k: int) -> list[float]:
    """k consecutive doubles centred on c."""
    out, lo = [c], c
    for _ in range(k // 2):
        lo = math.nextafter(lo, 0)
    out, v = [], lo
    for _ in range(k):
        out.append(v)
        v = math.nextafter(v, math.inf)
    return out


def used_pairs() -> set:
    got = set()
    for f in glob.glob(str(CASES.parents[1] / "*" / "cases" / "*.case.json")):
        if CASE in f:
            continue
        for sh in json.loads(open(f).read()).get("sheets", []):
            for v in sh.get("cells", {}).values():
                if isinstance(v, str):
                    for m in re.finditer(r"F\.DIST\([^,]+,([0-9.]+),([0-9.]+)", v):
                        got.add((float(m.group(1)), float(m.group(2))))
    return got


def plan() -> list[tuple[float, float, float]]:
    rng = random.Random(25860524)
    shapes = [(a, b) for a in D for b in D] + list(MIXED)
    out = []
    for d1, d2 in shapes:
        xs = run(1.0, 5)
        if d1 != d2:
            xs += run(d2 / d1, 5) + [math.exp(rng.uniform(math.log(0.3), math.log(3))) for _ in range(2)]
        else:
            xs += [math.exp(rng.uniform(math.log(0.3), math.log(3))) for _ in range(7)]
        out += [(float(d1), float(d2), x) for x in xs]
    return out


def build() -> dict:
    cells = {}
    for i, (d1, d2, x) in enumerate(plan(), 1):
        cells[f"A{i}"] = enc(x)
        cells[f"B{i}"] = f"=F.DIST(A{i},{d1:g},{d2:g},FALSE)"
    return {"format": "rustsheet-case/v1",
            "description": "T-2586 verify-r5-f-bigconst fresh hold-out: F.DIST(x,d1,d2,FALSE) at unused large shapes "
                           "around x = 1, x = d2/d1 and random x (column A echoes x).",
            "tags": ["M9", "probe", "statistics", "f-dist", "T-2586"],
            "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}


if __name__ == "__main__":
    clash = {(d1, d2) for d1, d2, _ in plan()} & used_pairs()
    assert not clash, sorted(clash)
    doc = build()
    print(len(doc["sheets"][0]["cells"]), "cells; no shape clash")
    path = CASES / f"{CASE}.case.json"
    if "--write" in sys.argv and not path.exists():
        path.write_text(json.dumps(doc, indent=1) + "\n")
        print("wrote", path)
