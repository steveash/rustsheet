"""T-2512 r3-bgrat: bgrat's stop rule on Excel's direct bgrat cells (probe t2512-r3-bgrat-split).

TOMS bgrat (r2-bratio's port, doubles, Excel's EXP/LN/log1p/GAMMALN, FINAL's lnx cut) called as the direct branch
does, bgrat(a, b0, x, y, w = 0, eps), with eps = 15e-15 (TOMS), 1e-15, 1e-16 and 1e-17 (eps 0 never ends grat1), scored against
Excel's D, plus the number of terms used."""
from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-tdist"))
import xlk  # noqa: E402

import branches  # noqa: E402
import score_lib as sl  # noqa: E402

NAME = "t2512-r3-bgrat-split"
out = json.load(open(HERE.parents[3] / "eval/suites/probes-numeric/cases" / f"{NAME}.oracle.json"))["sheets"]["Calc"]
meta = json.load(open(HERE / f"{NAME}.meta.json"))
xlk.run([(a, b0, x, 1.0 - x) for _k, a, b0, x, _c, _e in meta], branches.FINAL)  # fills GAMMALN/engine caches
K = xlk.bx._KERNELS[branches.FINAL]
EPS = [("toms 15e-15", 15e-15), ("1e-15", 1e-15), ("1e-16", 1e-16), ("1e-17", 1e-17)]
g = defaultdict(list)
for k, a, b0, x, case, _cell in meta:
    v = out.get(f"A{k}")
    if not v or v.get("t") != "n":
        continue
    for name, e in EPS:
        m = K.bgrat(a, b0, x, 1.0 - x, 0.0, e)
        g[(case.replace("t2464-", ""), name)].append((m, v["v"]))
        g[("ALL", name)].append((m, v["v"]))
print("| case | eps | n | exact/w4/w15 |")
print("|---|---|---|---|")
for k in sorted(g):
    n, e, f, w = sl.tally(g[k])
    print(f"| {k[0]} | {k[1]} | {n} | {e}/{f}/{w} |")
