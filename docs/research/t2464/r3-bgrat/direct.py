"""T-2512 r3-bgrat: Excel's direct bgrat cells (probe t2512-r3-bgrat-split) against bgrat models.

Models of D = BETA.DIST(x, a, b0) (direct bgrat branch, w = 0):
  M0      r2-tdist FINAL kernel (TOMS bgrat in doubles, Excel's EXP/LN/GAMMALN/log1p)
  CR      exact F at the double lnx
And the error distribution of D against F in ulps of D, per (a, b0) shape group."""
from __future__ import annotations

import json
import statistics as st
import sys
from collections import defaultdict
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-tdist"))
import xlk  # noqa: E402

import branches  # noqa: E402
import bscore as score  # noqa: E402
import score_lib as sl  # noqa: E402

NAME = "t2512-r3-bgrat-split"
out = json.load(open(HERE.parents[3] / "eval/suites/probes-numeric/cases" / f"{NAME}.oracle.json"))["sheets"]["Calc"]
meta = json.load(open(HERE / f"{NAME}.meta.json"))
pts = [(a, b0, x, 1.0 - x) for _k, a, b0, x, _c, _e in meta]
res = xlk.run(pts, branches.FINAL)
F = {(c["a"], c["b"], c["x"]): c["F"] for c in score.cells()}
grp = defaultdict(list)
for (k, a, b0, x, case, cell), (w, w1, br) in zip(meta, res):
    v = out.get(f"A{k}")
    if not v or v.get("t") != "n":
        continue
    dx = v["v"]
    f = F[(a, b0, x)]
    u = float((mpmath.mpf(dx) - f) / mpmath.mpf(sl.math.ulp(dx)))
    grp[(case.replace("t2464-", ""), round(a, 1), round(b0, 3))].append((w, dx, float(f), u))
print("| case | a~ | b0~ | n | M0 ex/w4/w15 | CR ex/w4/w15 | D - F (ulp of D) median, sd |")
print("|---|---|---|---|---|---|---|")
allm, allc = [], []
for k in sorted(grp):
    v = grp[k]
    m = sl.tally((t[0], t[1]) for t in v)
    c = sl.tally((t[2], t[1]) for t in v)
    allm += [(t[0], t[1]) for t in v]
    allc += [(t[2], t[1]) for t in v]
    us = [t[3] for t in v]
    print(f"| {k[0]} | {k[1]} | {k[2]} | {len(v)} | {m[1]}/{m[2]}/{m[3]} | {c[1]}/{c[2]}/{c[3]} | "
          f"{st.median(us):.1f}, {st.pstdev(us):.1f} |")
m, c = sl.tally(allm), sl.tally(allc)
print(f"| ALL | | | {m[0]} | {m[1]}/{m[2]}/{m[3]} | {c[1]}/{c[2]}/{c[3]} | |")
