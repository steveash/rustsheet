"""T-3568 r5-bup: does the n >= 2 bup form carry to the min <= 1 bup+bgrat branch? (T, n <= 30; r2-tdist p1 part A)

Cell = RN(B + G) times the tail scale, G = r2-tdist's solved bgrat at b = 1/2 (bgrat_half_ideal at a + 20), and
B = bup(a, 1/2, y, z, 20) on Excel's own density P = BETA.DIST(y, n/2, 1/2, FALSE) (probe t2464-r2-tdist-p1 part A).
bup's sum W (b = 1/2 <= 1, so TOMS's k = 0 and the eps break applies from the first term):
  exact      r2-tdist's: sum_k (a+1/2)_k/(a+1)_k y^k exactly, B = P y z / a W exact
  rec        TOMS's recurrence in doubles, d = fl(fl(ratio) * y) * d
  pow        model.py's: t_k = fl(R_k EXP(fl(k LN(y)))), R_k the stored ratio product, sum in doubles
with B = fl(x87(P fl(y z) / a) * W) for rec and pow. This is a check on cells model.py was not chosen on.
usage: tbup.py"""
from __future__ import annotations

import sys
from collections import defaultdict
from pathlib import Path

import mpmath
import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-tdist"))
import p1  # noqa: E402
import tlib  # noqa: E402
import tscore  # noqa: E402
import xlk  # noqa: E402

bx = tlib.bx
LD = np.longdouble
EPS = 1e-15


def w_sum(a: float, y: float, form: str) -> float:
    apb, ap1 = a + 0.5, a + 1.0
    lny = bx.LN(y)
    d, r, w = 1.0, 1.0, 1.0
    for i in range(1, 20):
        l = i - 1
        if form == "rec":
            d = (apb + l) / (ap1 + l) * y * d
        else:
            r = r * ((apb + l) / (ap1 + l))
            d = r * bx.EXP(i * lny)
        w = w + d
        if d <= EPS * w:
            break
    return w


P = {(y, n): p1.excel(i) for i, part, (y, n) in p1.META if part == "A"}
rows = tscore.rows_L()
vals, brs = tscore.evaluate(rows, p1.FINAL)
out = defaultdict(list)
for r, v, b in zip(rows, vals, brs):
    if b != "bup+bgrat/sw":
        continue
    n = r["n"]
    a = n / 2
    sc = 1 if r["kind"] == "2t" else 0.5
    _tt, _u, y, z = tlib.targs(abs(r["t"]), n)
    p = P[(y, n)]
    half = "hold" if r["hold"] else "tune"
    with mpmath.workdps(50):
        G = xlk.bgrat_half_ideal(a + 20, bx.LN(y))
        Y = mpmath.mpf(y)
        We = sum(mpmath.rf(a + 0.5, k) / mpmath.rf(a + 1, k) * Y**k for k in range(20))
        cand = {"exact W, exact B": mpmath.mpf(p) * Y * z / a * We}
        ret = LD(p) * LD(y * z) / LD(a)
        for form in ("rec", "pow"):
            cand[f"{form}: fl(x87 ret * W)"] = mpmath.mpf(float(ret * LD(w_sum(a, y, form))))
        for name, B in cand.items():
            val = float((B + G) * sc)
            out[(name, "all")].append((val, r["excel"]))
            out[(name, half)].append((val, r["excel"]))
    out[("FINAL model (r2-tdist)", "all")].append((v, r["excel"]))
for k in sorted(out, key=str):
    print(tlib.line(f"{k[0]} [{k[1]}]", out[k]))
