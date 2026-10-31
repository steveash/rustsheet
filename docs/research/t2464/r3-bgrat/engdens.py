"""T-2512 r3-bgrat: what an engine port of the one-term L140 composition would score with the engine's own pieces.

RN(fl(x87(P * fl(xy) / b0)) + inc), P = the engine's BETA.DIST(y, b0, a, FALSE) (python bindings, today's main) or
Excel's (probe -bupdens column B); inc = exact F (an accurate bgrat) or Excel's D. Also the engine's own BETA.DIST cdf
at the L140 cell (today's kernel) for reference."""
from __future__ import annotations

import json
from collections import defaultdict
from pathlib import Path

import mpmath
import numpy as np
import rustsheet as ic

import bscore as score
import score_lib as sl

HERE = Path(__file__).resolve().parent
CASES = HERE.parents[3] / "eval/suites/probes-numeric/cases"
LD = np.longdouble
s2 = json.load(open(CASES / "t2512-r3-bgrat-split.oracle.json"))["sheets"]["Calc"]
s3 = json.load(open(CASES / "t2512-r3-bgrat-bupdens.oracle.json"))["sheets"]["Calc"]
D = {(a, b0, x): s2.get(f"A{k}", {}).get("v") for k, a, b0, x, _c, _e in json.load(open(HERE / "t2512-r3-bgrat-split.meta.json"))}
PX = {(a, b0, x): s3.get(f"B{k}", {}).get("v") for k, a, b0, x in json.load(open(HERE / "t2512-r3-bgrat-bupdens.meta.json"))}
cs = [c for c in score.cells() if c["br"] == "L140 bup+bgrat" and (c["a"], c["b"], c["x"]) in PX]


# exact doubles through cells, not literals
m = ic.create("w", "en", "UTC")
for i, c in enumerate(cs):
    a, b0, x = c["a"], c["b"], c["x"]
    for j, v in enumerate((1.0 - x, b0, a, c["row"]["x"], c["row"]["a"], c["row"]["b"])):
        m.update_cell_with_number(0, i + 1, j + 1, v)
    m.update_cell_with_formula(0, i + 1, 7, f"=BETA.DIST(A{i + 1},B{i + 1},C{i + 1},FALSE)")
    m.update_cell_with_formula(0, i + 1, 8, f"=BETA.DIST(D{i + 1},E{i + 1},F{i + 1},TRUE)")
m.evaluate()


def val(r, col):
    v = m.get_cell_value(0, r, col) if hasattr(m, "get_cell_value") else m.get_formatted_cell_value(0, r, col)
    return float(v[-1] if isinstance(v, tuple) else v)


mpmath.mp.dps = 40
g = defaultdict(list)
for i, c in enumerate(cs):
    k = (c["a"], c["b"], c["x"])
    a, b0, x = k
    xy = x * (1.0 - x)
    pe, cell_e = val(i + 1, 7), val(i + 1, 8)
    ex = c["row"]["excel"]
    g["engine today (BETA.DIST cdf)"].append((cell_e, ex))
    for pn, p in (("P engine", pe), ("P Excel", PX[k])):
        W = float(LD(p) * LD(xy) / LD(b0))
        g[f"{pn} + exact F"].append((float(mpmath.mpf(W) + c["F"]), ex))
        if D.get(k) is not None:
            g[f"{pn} + Excel D"].append((float(mpmath.mpf(W) + D[k]), ex))
print("| model | n | exact/w4/w15 |")
print("|---|---|---|")
for k in sorted(g):
    n, e, f, w = sl.tally(g[k])
    print(f"| {k} | {n} | {e}/{f}/{w} |")
