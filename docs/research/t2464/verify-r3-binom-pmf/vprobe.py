"""The verifier's fresh hold-out t2513-verify-r3-binom-pmf-h (built after reading the claims, before scoring).

Blocks (p are random doubles from a fixed seed, not short decimals; consecutive doubles inside interior runs):
  K0   BINOM.DIST(0,n,p,FALSE)  n in N0, p in (0.002, 0.1) (R would use bd0), (0.1, 0.375) (series), (0.375, 0.9)
  KN   BINOM.DIST(n,n,p,FALSE)  p in (0.9, 0.998) (R would use bd0) and (0.1, 0.9)
  N1   NEGBINOM.DIST(f,1,p,FALSE) at the (f, p) of the first K0 rows (pairs: same dbinom_raw(0, f, p, q))
  INT  interior BINOM.DIST(k,n,p), BETA.DIST(p,k+1,n-k+1) and NEGBINOM.DIST(n-k,k,p) at shapes not in any
       earlier probe, runs of 28 consecutive doubles at bd0 ratio v = -0.2 and +0.2 on the smaller side
  echo `=<literal>` for every 16th distinct p (parser misreads)
Literals: (hi*2^e1+lo*2^e0) with hi < 2^27 and lo < 2^26, so no literal has more than 9 digits.
Usage: vprobe.py [--write]"""

from __future__ import annotations

import json
import math
import random
import sys

import vlib as V

CID = "t2513-verify-r3-binom-pmf-h"
N0 = [3, 7, 23, 64, 150, 999, 5000, 40000]
NN = [2, 6, 17, 45, 128, 700, 3000, 20000]
SHAPES = [(5, 14), (21, 70), (70, 300), (2, 9), (33, 41)]
VS = (-0.2, 0.2)


def lit(p: float) -> str:
    m, e = math.frexp(p)  # p = m 2^e, 0.5 <= m < 1
    M = int(m * 2**53)
    hi, lo = divmod(M, 2**26)
    e0 = e - 53
    s = f"({hi}*2^{e0 + 26}+{lo}*2^{e0})"
    assert float(V.excel_literal(s)) == p and V.excel_literal(s) == p
    return s


def rnd(rng: random.Random, a: float, b: float) -> float:
    return rng.uniform(a, b)


def p_at(k: int, n: int, v: float) -> float:
    if k <= n - k:
        return k * (1 - v) / (1 + v) / n
    return 1 - (n - k) * (1 - v) / (1 + v) / n


def build():
    rng = random.Random(25132513)
    cells: dict[str, str] = {}
    ps: list[float] = []
    row = 0

    def put(col: str, f: str, p: float) -> None:
        nonlocal row
        cells[f"{col}{row}"] = f
        ps.append(p)

    k0 = []
    for n in N0:
        for a, b, cnt in ((0.002, 0.1, 16), (0.1, 0.375, 12), (0.375, 0.9, 12)):
            k0 += [(n, rnd(rng, a, b)) for _ in range(cnt)]
    for row, (n, p) in enumerate(k0, 1):
        put("A", f"=BINOM.DIST(0,{n},{lit(p)},FALSE)", p)
        if row <= 240:
            put("B", f"=NEGBINOM.DIST({n},1,{lit(p)},FALSE)", p)
    kn = []
    for n in NN:
        for a, b, cnt in ((0.9, 0.998, 26), (0.1, 0.9, 14)):
            kn += [(n, rnd(rng, a, b)) for _ in range(cnt)]
    for row, (n, p) in enumerate(kn, 1):
        put("C", f"=BINOM.DIST({n},{n},{lit(p)},FALSE)", p)
    row = 0
    for k, n in SHAPES:
        for v in VS:
            p = p_at(k, n, v) * (1 + rng.uniform(-0.01, 0.01))
            for _ in range(28):
                row += 1
                put("D", f"=BINOM.DIST({k},{n},{lit(p)},FALSE)", p)
                put("E", f"=BETA.DIST({lit(p)},{k + 1},{n - k + 1},FALSE)", p)
                put("F", f"=NEGBINOM.DIST({n - k},{k},{lit(p)},FALSE)", p)
                p = math.nextafter(p, 1.0)
    uniq = sorted(set(ps))
    for j, p in enumerate(uniq[::16], 1):
        cells[f"H{j}"] = f"={lit(p)}"
    return {
        "format": "rustsheet-case/v1",
        "description": "T-2513 verify-r3-binom-pmf fresh hold-out: BINOM.DIST k=0 / k=n closed forms, NEGBINOM.DIST r=1, "
        "and interior BINOM/BETA/NEGBINOM runs at new shapes (docs/research/t2464/verify-r3-binom-pmf/vprobe.py)",
        "tags": ["M9", "probe", "statistics", "T-2513", "incomplete-beta"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }


if __name__ == "__main__":
    case = build()
    n = len(case["sheets"][0]["cells"])
    print(CID, n, "cells")
    assert n <= 2000
    if "--write" in sys.argv:
        (V.CASES / f"{CID}.case.json").write_text(json.dumps(case, indent=1) + "\n")
