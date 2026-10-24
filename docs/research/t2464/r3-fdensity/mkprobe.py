"""T-2513 r3-fdensity probes: F.DIST(x, d1, d2, FALSE) on exact-input x runs, with Excel's beta density at the two
candidate transformed points beside it, plus df sweeps at fixed x.

Run rows (a, hold): A = x (w2-forensics' exact encoder: no 15-digit literal truncation), and
  B  =F.DIST(A,d1,d2,FALSE)
  C  =BETA.DIST(1/(1+d2/(d1*A)),d1/2,d2/2,FALSE)      (w5's F cdf argument)        [a only]
  D  =BETA.DIST(d1*A/(d1*A+d2),d1/2,d2/2,FALSE)       (the engine's argument)        [a only]
Sweep rows (sw, holdsw): one cell =F.DIST(x,d1,d2,FALSE) with a short decimal x.
  a      in-sample: 22 shapes x 3 runs of 48 consecutive doubles (x ~ 0.37, 1.9, 13)
  sw     in-sample: d1 in {1,2,3,4,7} x d2 = 1..120 and d2 in {1,2,3,4,7} x d1 = 1..120, at x = 1 and 2.75
  hold   hold-out (built and filled with a/sw, read once after the model is frozen): 12 other shapes x 3 runs of 40
  holdsw hold-out: d1 in {5, 9} x d2 = 1..100 and d2 in {6, 9} x d1 = 1..100 at x = 1.6
Rows are rebuilt from this generator (no meta files); Excel values come from the eval sidecars.
Usage: mkprobe.py [--write]"""

from __future__ import annotations

import json
import sys

import flib as F

MP = F.MP
PREFIX = "t2513-r3-fdensity"
A_SHAPES = [
    (1, 3), (1, 10), (1, 50), (2, 3), (2, 10), (2, 50),
    (3, 5), (4, 4), (4, 20), (5, 2), (5, 10), (6, 1),
    (8, 8), (10, 3), (10, 30), (20, 20), (20, 100), (30, 6),
    (50, 50), (50, 100), (100, 10), (100, 100),
]
H_SHAPES = [
    (1, 7), (2, 25), (3, 12), (6, 6), (7, 2), (12, 40),
    (25, 8), (40, 60), (80, 30), (3, 1), (15, 15), (60, 200),
]
XS = (0.37, 1.9, 13.0)


def runs(shapes, cnt):
    out = []
    i = 0
    for d1, d2 in shapes:
        for c in XS:
            x0 = c * (1 + 0.000137 * i + 0.0000731)
            i += 1
            out += [(f"run({d1},{d2},x{c:g})", float(d1), float(d2), x) for x in MP.run(x0, 0, cnt)]
    return out


def sweeps(fixed1, fixed2, top, xs):
    out = []
    for x in xs:
        for d1 in fixed1:
            out += [(f"sw(d1={d1},x{x:g})", float(d1), float(d2), x) for d2 in range(1, top + 1)]
        for d2 in fixed2:
            out += [(f"sw(d2={d2},x{x:g})", float(d1), float(d2), x) for d1 in range(1, top + 1)]
    return out


PROBES = {
    "a": lambda: runs(A_SHAPES, 48),
    "sw": lambda: sweeps((1, 2, 3, 4, 7), (1, 2, 3, 4, 7), 120, (1.0, 2.75)),
    "hold": lambda: runs(H_SHAPES, 40),
    "holdsw": lambda: sweeps((5, 9), (6, 9), 100, (1.6,)),
}


def row_cells(r, d1, d2, x, kind):
    a, b = f"{d1 / 2:g}", f"{d2 / 2:g}"
    d1s, d2s = f"{d1:g}", f"{d2:g}"
    if kind in ("sw", "holdsw"):
        return {f"A{r}": f"=F.DIST({x!r},{d1s},{d2s},FALSE)"}
    A = f"A{r}"
    c = {A: "=" + MP.enc(x), f"B{r}": f"=F.DIST({A},{d1s},{d2s},FALSE)"}
    if kind == "a":
        c[f"C{r}"] = f"=BETA.DIST(1/(1+{d2s}/({d1s}*{A})),{a},{b},FALSE)"
        c[f"D{r}"] = f"=BETA.DIST({d1s}*{A}/({d1s}*{A}+{d2s}),{a},{b},FALSE)"
    return c


def build(name):
    cells = {}
    for r, (_, d1, d2, x) in enumerate(PROBES[name](), 1):
        cells.update(row_cells(r, d1, d2, x, name))
    assert len(cells) <= 16384, len(cells)
    return {
        "format": "rustsheet-case/v1",
        "description": f"T-2513 r3-fdensity probe {name}: F.DIST(x,d1,d2,FALSE) on exact-input x runs "
        "(with BETA.DIST at 1/(1+d2/(d1 x)) and d1 x/(d1 x+d2) beside it) or df sweeps at fixed x.",
        "tags": ["M9", "probe", "statistics", "f-dist", "T-2513"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }


def rows(name):
    """Probe rows with Excel's values (F in 'excel', the beta columns in 'beta_w5' / 'beta_d', x read back)."""
    sh = F.sidecar(f"{PREFIX}-{name}")
    out = []
    for r, (blk, d1, d2, x) in enumerate(PROBES[name](), 1):
        if name in ("sw", "holdsw"):
            v = F.val(sh, f"A{r}")
            if v is not None and v > 0:
                out.append(dict(src=name, blk=blk, x=x, d1=d1, d2=d2, excel=v))
            continue
        xr, v = F.val(sh, f"A{r}"), F.val(sh, f"B{r}")
        if v is None or not v > 0:
            continue
        assert xr == x, (name, r, xr, x)
        out.append(
            dict(
                src=name,
                blk=blk,
                x=x,
                d1=d1,
                d2=d2,
                excel=v,
                beta_w5=F.val(sh, f"C{r}"),
                beta_d=F.val(sh, f"D{r}"),
            )
        )
    return out


if __name__ == "__main__":
    tot = 0
    for name in PROBES:
        c = build(name)
        k = len(c["sheets"][0]["cells"])
        tot += k
        p = MP.CASES / f"{PREFIX}-{name}.case.json"
        print(name, k, "cells", "exists" if p.exists() else "new")
        if "--write" in sys.argv and not p.exists():
            p.write_text(json.dumps(c, indent=1) + "\n")
    print("total", tot)
