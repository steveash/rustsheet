"""Verifier (T-2586, verify-r5-f-final): build the verifier's hold-out t2586-verify-r5-f-final-hold and write the frozen
M-final's predictions (vhold_pred.json) BEFORE the oracle fill.

28 new shapes (literal and truncated (d1, d2) not used by any earlier F.DIST(.,FALSE) case, including the
integrator's hold-out): 7 with one df in 1..4 (partner log-uniform in [5, 2000]), 7 with min df in 5..15, 14 with
both df log-uniform in [16, 3000]; about half written as non-integer literals. Per shape: four runs of 6
consecutive doubles (side 1/2 x r <= 0.375 / r > 0.375) plus 8 random x = (d2/d1) exp(U(-7, 7)).
A = x (exact two-part encoding, echoed), B = F.DIST(A, d1, d2, FALSE). Column D: 40 literal-argument edge cells.
First fill (17:04) was rejected by Excel Online (unsupportedWorkbook) before any value existed: the literal
2.2250738585072014E-308 truncates to a subnormal; replaced by 2.22507385850739E-308; the second fill (17:04) was
rejected the same way, so literals below 2.2251E-308 are avoided (2.2251E-308 and a 18-digit one); the third fill
(17:05) was rejected too: the literal 1E308 is above Excel's largest typable number, replaced by 9.99999999999999E307.
Predictions were rewritten before each fill; no fill produced any value.
Usage: vmkhold.py [--write]"""

from __future__ import annotations

import functools
import hashlib
import json
import math
import random
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import vfparse as V  # noqa: E402

CASE = "t2586-verify-r5-f-final-hold"
SEED = 2586051
EDGES = [
    "0,1.5,9", "0,2.5,3", "0,2,1E10", "0,1,1", "0,4.2,11", "FALSE,2,5", "FALSE,3,5",
    "-1E-10,4,4", "-3,2,2", "1.5,0.999999999,7", "1.5,7,0", "2,9999999999.9,6", "0.4,10000000000,3",
    "0.4,6,10000000000.9", "0.4,6,10000000001", "2.2251E-308,3,4", "2.22510000000000009E-308,3,4",
    "2.2251E-308,1,1", "1E-307,2,5", "1E-307,5,2", "9.99999999999999E307,2,3", "9.99999999999999E307,1,3", "8.9E307,2,2",
    "0.333333333333333333,7,9", "2.71828182845904523,2.5,50.5", "123456789.123456789,1,3",
    "0.999999999999999999,40,40", "TRUE,1,1", "TRUE,9,2", "1,TRUE,5", '"0.5",3,8', '"1E-3",2,30',
    '"0.12345678901234567",4,6', '2,"7.9",12', "1E-12,1,6", "1E12,6,1", "1E-5,300,2", "1E5,2,300",
    "0.05,700,700", "1.02,2000,2000",
]


def enc(x: float) -> str:
    m, e = math.frexp(x)
    M, E = int(m * 2**53), e - 53
    hi, lo = M >> 26, M & (2**26 - 1)
    return f"=({hi}*2^{E + 26}+{lo}*2^{E})"


def lit(v: float, rnd: random.Random, nonint: bool) -> str:
    if nonint:
        return f"{v + rnd.choice([0.1, 0.25, 0.5, 0.75, 0.9, 0.37]):.2f}".rstrip("0").rstrip(".")
    return str(int(v))


@functools.lru_cache(maxsize=None)
def earlier_shapes():
    li, tr = set(), set()
    for c in V.all_cases():
        if c == CASE:
            continue
        for r in V.rows(c):
            li.add((r["d1"], r["d2"]))
            tr.add((math.trunc(r["d1"]), math.trunc(r["d2"])))
    return li, tr


def shapes(rnd):
    li, tr = earlier_shapes()
    out, used = [], set()

    def lu(a, b):
        return math.floor(math.exp(rnd.uniform(math.log(a), math.log(b))))

    plan = ["small"] * 7 + ["mid"] * 7 + ["large"] * 14
    for kind in plan:
        while True:
            if kind == "small":
                p, q = rnd.randint(1, 4), lu(5, 2000)
            elif kind == "mid":
                p, q = rnd.randint(5, 15), lu(5, 2000)
            else:
                p, q = lu(16, 3000), lu(16, 3000)
            if rnd.random() < 0.5:
                p, q = q, p
            s1 = lit(p, rnd, rnd.random() < 0.5)
            s2 = lit(q, rnd, rnd.random() < 0.5)
            d1, d2 = float(s1), float(s2)
            t = (math.trunc(d1), math.trunc(d2))
            if (d1, d2) not in li and t not in tr and t not in used:
                used.add(t)
                out.append((s1, s2))
                break
    return out


@functools.lru_cache(maxsize=None)
def plan():
    rnd = random.Random(SEED)
    rows = []
    for s1, s2 in shapes(rnd):
        m, n = math.trunc(float(s1)), math.trunc(float(s2))
        for side in (1, 2):
            for lo in (True, False):
                r = math.exp(rnd.uniform(math.log(1e-3), math.log(0.375))) if lo else rnd.uniform(0.3751, 0.9999)
                x = r * n / m if side == 1 else n / (m * r)
                for _ in range(6):
                    rows.append((f"s{side}{'lo' if lo else 'hi'}", s1, s2, x))
                    x = math.nextafter(x, math.inf)
        for _ in range(8):
            rows.append(("rand", s1, s2, (n / m) * math.exp(rnd.uniform(-7, 7))))
    return rows


def build():
    cells = {}
    for i, (_, s1, s2, x) in enumerate(plan(), 1):
        cells[f"A{i}"] = enc(x)
        cells[f"B{i}"] = f"=F.DIST(A{i},{s1},{s2},FALSE)"
    for i, a in enumerate(EDGES, 1):
        cells[f"D{i}"] = f"=F.DIST({a},FALSE)"
    return {
        "format": "rustsheet-case/v1",
        "description": "T-2586 verify-r5-f-final: verifier's hold-out for F.DIST(x,d1,d2,FALSE) on 28 new shapes "
        "(7 with one df < 5, 7 mid, 14 large up to 3000) x 4 regions x 6 consecutive doubles + 8 random x (A echoes x), "
        "and 40 literal-argument edge cells in column D.",
        "tags": ["M9", "probe", "statistics", "f-dist", "T-2586"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }


def predict(doc):
    sys.path.insert(0, str(HERE))
    import vflib as L

    M = L.load_model()
    cells = doc["sheets"][0]["cells"]
    pred = {}
    for k, f in cells.items():
        if k[0] == "A":
            continue
        inner = f[len("=F.DIST("):-1]
        args = V.split_args(inner)
        vals = [V.evaluate(a, lambda r: V.evaluate(cells[r])[0])[0] for a in args[:3]]
        pred[k] = M.value(*vals)
    P = plan()
    for k, f in cells.items():
        if k[0] == "A":
            assert V.evaluate(f)[0] == P[int(k[1:]) - 1][3]
    return pred


if __name__ == "__main__":
    doc = build()
    n = sum(len(s["cells"]) for s in doc["sheets"])
    pred = predict(doc)
    print(f"{n} cells, {len(pred)} predictions")
    if "--write" in sys.argv:
        cp = V.CASES / f"{CASE}.case.json"
        assert not cp.exists() and not (V.CASES / f"{CASE}.oracle.json").exists()
        pp = HERE / "vhold_pred.json"
        pp.write_text(json.dumps(pred, indent=0, sort_keys=True))
        cp.write_text(json.dumps(doc, indent=2) + "\n")
        print("pred sha256", hashlib.sha256(pp.read_bytes()).hexdigest())
