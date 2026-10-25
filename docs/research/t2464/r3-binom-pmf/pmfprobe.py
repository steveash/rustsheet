"""T-2513 r3-binom-pmf probes: BINOM.DIST(k,n,p,FALSE) on runs of consecutive doubles p at fixed (k, n), placed by
Loader's bd0 ratio on the smaller side (v = (k - n p)/(k + n p) when k <= n - k, else the q-side ratio), with
companion columns BETA.DIST(p,k+1,n-k+1,FALSE) (= (n+1) dbinom_raw(k,n,p,q) if Excel follows R) and
NEGBINOM.DIST(n-k,k,p,FALSE) (= (k/n) dbinom_raw(k,n,p,q) in R).  Every 16th row echoes p in column D.

  a     in-sample: 14 (k, n) shapes (5 with n a power of two, so n p is exact) at v = -0.35, -0.12, +0.12, +0.35,
        96 doubles each; columns A (BINOM), B (BETA), D (echo).
  n     in-sample: the first 48 rows of each `a` run as NEGBINOM (column A), plus large-n BINOM runs (column B)
        at bd0 = 3 and 20 nats, 64 each.
  hold  hold-out, built and filled with a and n before any tuning, scored once: 9 other shapes at v = -0.25,
        -0.07, +0.06, +0.25, 96 each; columns A (BINOM), B (BETA), C (NEGBINOM), D (echo).
Run starts are irregular (target p times 1 + 0.000137 i + 0.0000731).  Literals: w2-forensics' exact encoder.
The row map is rebuilt from this file (blib.load_probe), so no meta file is kept.
Usage: pmfprobe.py [--write]"""

from __future__ import annotations

import importlib.util
import json
import math
import sys
from pathlib import Path

HERE = Path(__file__).parent
_spec = importlib.util.spec_from_file_location("w2_make_probes", HERE.parent / "w2-forensics" / "make_probes.py")
MP = importlib.util.module_from_spec(_spec)  # w2-forensics: num(), run() (loaded by path: other dirs share the name)
_spec.loader.exec_module(MP)

CASES = Path("/home/user/rustsheet/eval/suites/probes-numeric/cases")
A_SHAPES = [(3, 8), (5, 16), (10, 32), (40, 128), (100, 256), (3, 10), (7, 20), (13, 30), (30, 100), (37, 77),
            (150, 1000), (1, 10), (2, 50), (45, 50)]
A_VS = (-0.35, -0.12, 0.12, 0.35)
HOLD_SHAPES = [(4, 12), (6, 40), (17, 64), (25, 60), (60, 200), (400, 1500), (1, 30), (29, 30), (11, 23)]
HOLD_VS = (-0.25, -0.07, 0.06, 0.25)
BIG = [(3000, 10000), (30000, 100000), (300000, 1000000), (5, 100000), (1, 1000), (999, 1000), (500, 1000),
       (4000000, 10000000)]


def p_for(k: int, n: int, v: float) -> float:
    if k <= n - k:
        return k * (1 - v) / (1 + v) / n
    m = n - k
    return 1 - m * (1 - v) / (1 + v) / n


def runs(shapes, vs, cnt, tag, i0=0):
    out = []
    i = i0
    for k, n in shapes:
        for v in vs:
            p0 = p_for(k, n, v) * (1 + 0.000137 * i + 0.0000731)
            i += 1
            if not 0.002 < p0 < 0.998:
                continue
            out.append([f"{tag}({k},{n})v{v:+.2f}", k, n, p0, cnt])
    return out


def big_runs():
    out = []
    i = 100
    for k, n in BIG:
        m = min(k, n - k)
        for nats in (3.0, 20.0):
            v = min(0.35, math.sqrt(nats / (2 * m)))
            for s in (-1, 1):
                p0 = p_for(k, n, s * v) * (1 + 0.000137 * i + 0.0000731)
                i += 1
                if 0.0 < p0 < 1.0:
                    out.append([f"big({k},{n})v{s * v:+.4f}", k, n, p0, 64])
    return out


def expand(blocks):
    for blk, k, n, p0, cnt in blocks:
        for p in MP.run(p0, 0, cnt):
            yield blk, k, n, p


def binom(k, n, p):
    return f"=BINOM.DIST({k},{n},{MP.num(p)},FALSE)"


def beta(k, n, p):
    return f"=BETA.DIST({MP.num(p)},{k + 1},{n - k + 1},FALSE)"


def negb(k, n, p):
    return f"=NEGBINOM.DIST({n - k},{k},{MP.num(p)},FALSE)"


def build(name):
    cells, rows, echo = {}, [], []
    if name in ("a", "hold"):
        bl = runs(A_SHAPES, A_VS, 96, "a") if name == "a" else runs(HOLD_SHAPES, HOLD_VS, 96, "h", 50)
        for j, (blk, k, n, p) in enumerate(expand(bl), 1):
            cells[f"A{j}"] = binom(k, n, p)
            rows.append([f"A{j}", "BIN", k, n, p, blk])
            cells[f"B{j}"] = beta(k, n, p)
            rows.append([f"B{j}", "BETA", k, n, p, blk])
            if name == "hold":
                cells[f"C{j}"] = negb(k, n, p)
                rows.append([f"C{j}", "NEG", k, n, p, blk])
            if j % 16 == 1:
                cells[f"D{j}"] = f"={MP.num(p)}"
                echo.append([f"D{j}", p])
    else:
        j = 0
        for blk, k, n, p0, cnt in runs(A_SHAPES, A_VS, 96, "a"):
            for p in MP.run(p0, 0, 48):
                j += 1
                cells[f"A{j}"] = negb(k, n, p)
                rows.append([f"A{j}", "NEG", k, n, p, blk])
        bl = big_runs()
        for j, (blk, k, n, p) in enumerate(expand(bl), 1):
            cells[f"B{j}"] = binom(k, n, p)
            rows.append([f"B{j}", "BIN", k, n, p, blk])
            if j % 16 == 1:
                cells[f"D{j}"] = f"={MP.num(p)}"
                echo.append([f"D{j}", p])
    assert len(cells) <= 16384, len(cells)
    cid = f"t2513-r3-binom-pmf-{name}"
    case = {"format": "rustsheet-case/v1",
            "description": f"T-2513 r3-binom-pmf probe {name}: BINOM.DIST / BETA.DIST / NEGBINOM.DIST pmf runs of "
                           "consecutive p at Loader bd0 ratios (docs/research/t2464/r3-binom-pmf/pmfprobe.py)",
            "tags": ["M9", "probe", "statistics", "T-2513", "incomplete-beta"],
            "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
    return cid, case, dict(case=cid, rows=rows, echo=echo)


if __name__ == "__main__":
    for name in ("a", "n", "hold"):
        cid, case, meta = build(name)
        print(name, cid, len(case["sheets"][0]["cells"]), "cells")
        if "--write" in sys.argv:
            (CASES / f"{cid}.case.json").write_text(json.dumps(case, indent=1) + "\n")
