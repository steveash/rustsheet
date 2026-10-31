"""Build this worker's probes eval/suites/probes-numeric/cases/t2513-r3-m1-residual-<name>.case.json.

Each row: A = x, B = a, C = b (exact two-piece literals, immune to the 15-digit literal cut, read back from the
sidecar), D = BETA.DIST(A,B,C,FALSE). 4,096 rows (16,384 cells) per case. Blocks (min(a,b) <= 1 throughout):
  a1big   a = 1, b > 8: lnB = algdiv(1, b) (the exponent path; per-shape lnB pin)
  albig   a < 1, b > 8 with a/b in (0, 0.375]: algdiv(a, b), alnrel(a/b) of a positive argument
  gl      a < 1, 1 < b < 8: lnGamma(a0 < 1) inside the GL form
  w4      (0.6, 7.3), (0.5, 6.25) and neighbours (verify-r2-bpser-small's losing shapes)
  blt1    b < 1 with 1 < a < 8 and a > 8
  runs    consecutive-double x runs (the exponent's rounding structure)
Shapes and x of `hold` differ from `a` (made before any tuning on `a`; scored once, with the frozen model).
Usage: mkprobe.py a|hold"""

import json
import math
import random
import sys

import m1lib as M


# Shape specs: "a:b" pairs (a1big: b only, a = 1; runs: "a:b:x0"), kept as strings so the lists stay compact.
SPECS = {
    "a": dict(
        seed=11,
        a1big="8.5 9.0 9.7 10.3 12.25 15.0 17.9 23.4 31.0 47.3 64.0 99.9 150.5 333.0 1000.1 2500.0 7777.7 10000.3 "
        "30000.0 100000.5 1e6 3.3e6 2.5e7 1.1e9",
        albig="0.3:8.5 0.5:9.3 0.7:11.0 0.9:14.7 0.15:20.2 0.95:33.0 0.45:60.6 0.25:150.0 0.6:1000.5 0.8:8.25 "
        "0.35:9.9 0.05:12.5 0.99:25.0 0.55:45.5 0.2:99.0 0.75:333.3 0.65:3000.0 0.4:2.0e4 0.85:2.6 0.1:3.0",
        gl_a="0.05 0.2 0.3 0.45 0.55 0.65 0.8 0.95", gl_b="1.7 3.6 6.9",
        w4="0.6:7.3 0.5:6.25 0.6:6.25 0.5:7.3",
        blt1="1.7:0.4 2.5:0.9 4.4:0.25 7.5:0.6 9.5:0.5 20.5:0.3 61.0:0.8 500.0:0.7 1.2:0.05 3.3:0.99 12.0:0.15 "
        "88.8:0.95",
        runs="0.5:20.0:0.21 0.7:3.6:0.61 1.0:33.0:0.1 0.3:9.9:0.52 2.5:0.6:0.31 0.9:150.0:0.004",
    ),
    "hold": dict(
        seed=29,
        a1big="8.25 8.75 9.4 11.6 13.3 19.0 21.7 28.6 40.0 55.5 77.7 128.0 201.3 512.5 1500.0 4096.5 9999.0 2.0e4 "
        "6.6e4 2.5e5 7.0e5 9.1e6 6.0e7 4.0e8",
        albig="0.33:8.8 0.42:10.1 0.66:12.7 0.88:16.6 0.12:22.2 0.97:40.4 0.51:70.7 0.27:210.0 0.62:1999.0 0.77:8.1 "
        "0.38:9.6 0.07:13.9 0.93:27.5 0.58:50.5 0.18:120.0 0.73:444.4 0.68:5000.0 0.44:3.0e4 0.21:15.5 0.83:9.05",
        gl_a="0.08 0.17 0.33 0.42 0.58 0.71 0.86 0.99", gl_b="1.3 2.9 5.5",
        w4="0.6:7.3 0.5:6.25 0.6:5.8 0.5:7.7",
        blt1="1.4:0.45 2.2:0.85 5.1:0.3 6.7:0.65 8.8:0.55 15.5:0.35 44.0:0.75 900.0:0.6 1.1:0.1 3.9:0.97 10.5:0.2 "
        "70.0:0.9",
        runs="0.45:25.0:0.18 0.6:4.4:0.66 1.0:47.0:0.08 0.35:12.0:0.47 3.1:0.7:0.36 0.85:99.0:0.006",
    ),
}


def _tuples(spec):
    return [tuple(float(v) for v in item.split(":")) for item in spec.split()]


def shapes(name):
    sp = SPECS[name]
    gl = [(a, b) for (a,) in _tuples(sp["gl_a"]) for (b,) in _tuples(sp["gl_b"])]
    return dict(a1big=[(1.0, b) for (b,) in _tuples(sp["a1big"])], albig=_tuples(sp["albig"]), gl=gl,
                w4=_tuples(sp["w4"]), blt1=_tuples(sp["blt1"]), runs=_tuples(sp["runs"])), sp["seed"]


def xs_for(a, b, k, rng):
    """k irregular x where the density is a normal double and the exponent is moderate (both log zones)."""
    out = []
    # x range: for large b keep (b-1)(-ln(1-x)) <= 300; for large a likewise for (a-1)(-ln x)
    hi = 1.0 - math.exp(-300.0 / max(b - 1.0, 1e-9)) if b > 1 else 0.999
    lo = math.exp(-300.0 / max(a - 1.0, 1e-9)) if a > 1 else 1e-4
    hi, lo = min(hi, 0.9995), max(lo, 1e-5)
    while len(out) < k:
        u = rng.random()
        x = lo * (hi / lo) ** u if rng.random() < 0.5 else lo + (hi - lo) * u
        x = float(f"{x:.17g}")
        out.append(x)
    return out


def rows_of(name):
    sh, seed = shapes(name)
    rng = random.Random(seed)
    rows, blk = [], []
    per = {"a1big": 40, "albig": 48, "gl": 40, "w4": 40, "blt1": 40}
    for bn, lst in sh.items():
        if bn == "runs":
            continue
        for a, b in lst:
            for x in xs_for(a, b, per[bn], rng):
                rows.append((x, a, b))
                blk.append(bn)
    left = 4096 - len(rows)
    per_run = left // len(sh["runs"])
    for a, b, x0 in sh["runs"]:
        x = x0 + rng.random() * 1e-3
        for _ in range(per_run):
            rows.append((x, a, b))
            blk.append("runs")
            x = math.nextafter(x, 1.0)
    return rows, blk


def beta_dist_cells(rows):
    """A = x, B = a, C = b as exact literals (read back to catch a literal cut), D = BETA.DIST(A,B,C,FALSE)."""
    cells = {}
    for i, (x, a, b) in enumerate(rows):
        r = i + 1
        cells[f"A{r}"] = "=" + M.lit(x)
        cells[f"B{r}"] = "=" + M.lit(a)
        cells[f"C{r}"] = "=" + M.lit(b)
        cells[f"D{r}"] = f"=BETA.DIST(A{r},B{r},C{r},FALSE)"
    return cells


def write_case(cid, description, task, cells):
    """Write probes-numeric case `cid` (one Calc sheet, no expectations) tagged M9 / `task`."""
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M9", "probe", "statistics", task, "incomplete-beta"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    (M.SUITE / f"{cid}.case.json").write_text(json.dumps(case, indent=1) + "\n")


def main(name):
    rows, blk = rows_of(name)
    assert len(rows) <= 4096, len(rows)
    cells = beta_dist_cells(rows)
    cid = f"t2513-r3-m1-residual-{name}"
    write_case(
        cid,
        f"T-2513 r3-m1-residual probe {name}: BETA.DIST(x,a,b,FALSE) with min(a,b) <= 1 (a = 1 and "
        "a < 1 with b > 8 for TOMS algdiv, a < 1 with 1 < b < 8 for lnGamma(a0 < 1), the (0.6, 7.3) and (0.5, 6.25) "
        "shapes, b < 1, consecutive-double runs); x, a, b are exact two-piece literals read back in A:C",
        "T-2513",
        cells,
    )
    (M.HERE / f"probe_{name}.meta.json").write_text(
        json.dumps({"case": cid, "rows": rows, "blk": blk}) + "\n"
    )
    print(cid, len(rows), "rows", len(cells), "cells", {b: blk.count(b) for b in dict.fromkeys(blk)})

if __name__ == "__main__":
    main(sys.argv[1])
