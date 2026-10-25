"""T-2464 w2-forensics: generate oracle probe cases t2464-w2f-*.case.json (probes-numeric).

Every argument that is not a short decimal is written as exact arithmetic of exact pieces,
(H*2^e1+L*2^e2), which Excel and the fit lab reader both evaluate exactly.
Run: bindings/python/.venv/bin/python make_probes.py [--write]
"""
from __future__ import annotations

import json
import math
import sys
from pathlib import Path

CASES = Path("/home/user/rustsheet/eval/suites/probes-numeric/cases")


def enc(v: float) -> str:
    """Exact formula text for the double v (0 < v)."""
    m, e = math.frexp(v)  # v = m*2^e, 0.5 <= m < 1
    M = int(m * 2**53)
    E = e - 53
    while M % 2 == 0 and M:
        M //= 2
        E += 1
    if M < 10**14 and E >= -60:
        if E == 0:
            return str(M)
        return f"{M}*2^{E}"
    H, L = M >> 26, M & ((1 << 26) - 1)
    return f"({H}*2^{E + 26}+{L}*2^{E})"


def step(v: float, k: int) -> float:
    for _ in range(abs(k)):
        v = math.nextafter(v, math.inf if k > 0 else -math.inf)
    return v


def run(x0: float, k0: int, n: int) -> list[float]:
    v = step(x0, k0)
    out = []
    for _ in range(n):
        out.append(v)
        v = math.nextafter(v, math.inf)
    return out


def num(v: float) -> str:
    r = repr(v)
    if len(r.replace("0.", "").replace(".", "")) <= 15 and float(r) == v:
        return r
    return enc(v)


def case(name: str, desc: str, rows: list[tuple[float, float, float, bool]]) -> dict:
    assert len(rows) <= 16384, len(rows)
    cells = {}
    for i, (x, a, b, cum) in enumerate(rows, 1):
        cells[f"A{i}"] = f"=BETA.DIST({num(x)},{num(a)},{num(b)},{'TRUE' if cum else 'FALSE'})"
    return {"format": "rustsheet-case/v1", "description": desc,
            "tags": ["M9", "probe", "statistics", "T-2464", "incomplete-beta"],
            "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}


def both(xs, a, b):
    return [(x, a, b, c) for x in xs for c in (True, False)]


def build() -> dict[str, dict]:
    out = {}
    N = 1024
    # x runs: 1024 consecutive doubles from an irregular start, cdf and pdf at each.
    r1 = [(2.5, 3.5, 0.2113571), (2.5, 3.5, 0.3313571), (7.3, 11.6, 0.2731), (0.5, 6.25, 0.0431),
          (30.7, 1.7, 0.8117), (4.0, 4.0, 0.2317), (1.5, 0.5, 0.3511), (100.0, 120.0, 0.4013)]
    rows = []
    for a, b, x0 in r1:
        rows += both(run(x0, 0, N), a, b)
    out["t2464-w2f-xruns-1"] = case("xruns-1", "T-2464 w2-forensics: BETA.DIST cdf and pdf on 8 runs of 1024 "
                                    "consecutive doubles in x, direct branch (a,b,x0) = " + str(r1), rows)
    # reflected runs and runs straddling candidate branch boundaries (centred on the boundary double).
    r2 = [(2.5, 3.5, 0.6137, 0), (0.5, 6.25, 0.3113, 0), (7.3, 11.6, 0.6131, 0),
          (2.5, 3.5, 1 / 3.5, -512),          # b*x = 1 (Cephes power-series rule)
          (40.0, 1.03, 0.95, -512),           # x = 0.95 (Cephes) with b*x < 1
          (2.5, 3.5, 2.5 / 6.0, -512),        # the mean a/(a+b)
          (0.5, 0.7, 0.5 / 1.2, -512),        # the mean, a < 1
          (0.5, 0.7, 0.5, -512)]              # x = 0.5 (TOMS bratio rule for min(a,b) <= 1)
    rows = []
    for a, b, x0, k0 in r2:
        rows += both(run(x0, k0, N), a, b)
    out["t2464-w2f-xruns-2"] = case("xruns-2", "T-2464 w2-forensics: BETA.DIST cdf and pdf, 1024 consecutive x "
                                    "doubles: 3 reflected runs, then runs centred on b*x=1, x=0.95, the mean "
                                    "(a,b>1 and a<1) and x=0.5: (a,b,x0,k0) = " + str(r2), rows)
    # a and b sweeps: 512 consecutive doubles in the shape, x and the other shape fixed.
    rows = []
    sw = [("a", 2.5, 3.5, 0.2113571), ("a", 7.3, 11.6, 0.2731), ("a", 0.5, 6.25, 0.0431), ("a", 30.7, 1.7, 0.8117),
          ("b", 2.5, 3.5, 0.2113571), ("b", 7.3, 11.6, 0.2731), ("b", 0.5, 6.25, 0.0431), ("b", 30.7, 1.7, 0.8117)]
    for which, a, b, x in sw:
        if which == "a":
            for av in run(a, -256, 512):
                rows += both([x], av, b)
        else:
            for bv in run(b, -256, 512):
                rows += both([x], a, bv)
    out["t2464-w2f-absweep"] = case("absweep", "T-2464 w2-forensics: BETA.DIST cdf and pdf with x fixed and 512 "
                                    "consecutive doubles in a (first 4 blocks) or b (last 4), centred on "
                                    "(shape, a, b, x) = " + str(sw), rows)
    # closed-form switches: a or b = 1 +- k ulp.
    rows = []
    for other in (0.5, 2.5, 5.0):
        for x in (0.1, 0.5, 0.9):
            for k in range(-16, 17):
                rows.append((x, step(1.0, k), other, True))
                rows.append((x, other, step(1.0, k), True))
    out["t2464-w2f-unit-shape"] = case("unit-shape", "T-2464 w2-forensics: BETA.DIST cdf at a = 1 +- k ulp or "
                                       "b = 1 +- k ulp (k = -16..16), other shape 0.5, 2.5, 5, x 0.1, 0.5, 0.9", rows)
    return out


if __name__ == "__main__":
    cs = build()
    for name, c in cs.items():
        p = CASES / f"{name}.case.json"
        print(name, len(c["sheets"][0]["cells"]), "exists" if p.exists() else "")
        if "--write" in sys.argv:
            assert not p.exists()
            p.write_text(json.dumps(c, indent=1) + "\n")
