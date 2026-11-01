"""T-3568 r5-bup: Gate A envelope of the frozen model on the post-freeze sweep (decision 0188 R4.5, check 7(c)).

A point is inside when the model's cell is within half a unit of the 15th significant digit of Excel's, or within 1e-9
relative of the 50-digit I_x0(a0, b0) at the oriented call. Two D variants: the committed D column (Excel's
BETA.DIST(x0, a0, c, TRUE)) and D = RN(x0^a0) at c = 1. Points outside are listed. usage: gatea.py [probe]"""
from __future__ import annotations

import sys

import mpmath

import model
import score as s
from common import sl

half = sys.argv[1] if len(sys.argv) > 1 else "fresh"
rows = s.load(half)
for label, dfun in (("D column", lambda r: None),
                    ("D = RN(x0^a0) at c = 1", lambda r: float(mpmath.mpf(r["x0"]) ** r["a0"]) if r["c"] == 1.0 else None)):
    out = []
    for r in rows:
        v = model.cell(r, dfun(r))
        if sl.w15(v, r["excel"]):
            continue
        t = s.exact(r)
        if t != 0 and abs(v / t - 1) <= 1e-9:
            continue
        out.append((r["k"], r["region"], r["a0"], r["b0"], r["x0"], r["excel"], v, t))
    print(f"{half}, {label}: {len(rows) - len(out)} of {len(rows)} inside")
    for k, reg, a0, b0, x0, e, v, t in out:
        print(f"  outside row {k} {reg}: a0={a0:g} b0={b0:g} x0={x0!r} excel={e!r} model={v!r} exact={t!r}")
