"""Score r2-tdist's xlk.bgrat_half_ld (imported; code read and checked against its description) with this
directory's dispatch, lnx and scorer on the cells.tsv T bgrat cells. usage: vld.py"""
import math
import sys

import vlib

sys.path.insert(0, str(vlib.T / "r2-tdist"))
import englib  # noqa: E402
import xlk  # noqa: E402

rows = [r for r in vlib.load_cells()]
sel = []
for r in rows:
    y, z = vlib.targs(r["t"], r["n"])
    if vlib.branch(r["n"], y, z) == "bgrat":
        r["y"], r["z"] = y, z
        sel.append(r)
vlib.LN([r["y"] for r in sel])
pts = []
for r in sel:
    lnx = vlib.C[("LN", r["y"])] if r["z"] > 0.01 else vlib.log1p_xl(-r["z"])
    pts.append((r["n"] / 2, lnx))
englib.eng("ERFC.PRECISE", [math.sqrt(-(a - 0.25) * l) for a, l in pts])
pairs = [(xlk.bgrat_half_ld(a, l) * r["sc"], r["excel"]) for (a, l), r in zip(pts, sel)]
print("bgrat_half_ld cells.tsv:", vlib.score(pairs))
