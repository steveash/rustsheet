"""T-3568 r5-bup: score the frozen model (model.py) on t3568-r5-bup-tune, -hold and -fresh (post-freeze), every region.

Columns: FROZEN (model.cell), REC (r3-bgrat's n = 1 form carried to n terms: x87 ret times the exact recurrence sum,
score.py), M0 (r5-bgrat-fdf1's frozen kernel as is), CR (50 digits at the kernel's x0). At c = 1 the D column is
Excel's a0 = 1 closed form, not its bgrat at b = 1; FROZEN+pow uses D = RN(x0^a0) there instead.
usage: score_hold.py"""
from __future__ import annotations

from collections import defaultdict

import mpmath

import model
import score as s
from score import fmodel
from common import sl, xlk

for half in ("tune", "hold", "fresh"):
    rows = s.load(half)
    k0 = xlk.run([(r["a0"], r["b0"], r["x0"], r["y0"]) for r in rows], fmodel.CFG)
    g = defaultdict(lambda: defaultdict(list))
    for r, o in zip(rows, k0):
        m0 = o[0]
        vals = {"FROZEN": model.cell(r), "REC": s.model(r, "x87", "ex", "x87"), "M0": m0, "CR": s.exact(r)}
        if r["c"] == 1.0:
            vals["FROZEN, D = RN(x0^a0)"] = model.cell(r, float(mpmath.mpf(r["x0"]) ** r["a0"]))
        reg = r["region"] + (" c=1" if r["c"] == 1.0 else "")
        for key in (reg, "ALL"):
            for m, v in vals.items():
                g[key][m].append((v, r["excel"]))
    names = ["FROZEN", "FROZEN, D = RN(x0^a0)", "REC", "M0", "CR"]
    print(f"\n## {half}: exact/w4/w15 (cells)\n")
    print("| region | cells | " + " | ".join(names) + " |")
    print("|---|---|" + "---|" * len(names))
    for key in sorted(g, key=lambda k: (k == "ALL", k)):
        n = len(g[key]["FROZEN"])
        line = f"| {key} | {n} |"
        for m in names:
            nn, e, f, w = sl.tally(g[key][m])
            line += f" {e}/{f}/{w} |" if nn else " - |"
        print(line)
