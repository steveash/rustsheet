"""bgrat at b = 1/2 as the leading identity I = C(n) Q(1/2, z) (1 + corr), every factor but Excel's Q exact:
model = RN(F(lnx) * Q_xl / Q_exact(z_exact)), F(lnx) = exact I_y(a, 1/2) at ln y = lnx, z = fl(-nu lnx),
Q_xl = Excel's ERFC(fl(sqrt(z))) (z >= 0.25; else 1 - ERF), Q_exact = erfc(sqrt(-nu lnx)).
This isolates how Excel's value depends on lnx, z and its ERFC, with a perfect constant and a perfect series.
usage: hx.py   (bgrat-only cells: branch bgrat/sw under the cut0.01 spec)"""
import math
import sys
from collections import defaultdict

import mpmath

import englib
import tlib
import tscore
import xlk

bx = tlib.bx
SPEC = "args=yz,alnrel=xl,erfc=xl,bg_lnx=cut0.01"
rows = tscore.rows_L()
vals, brs = tscore.evaluate(rows, SPEC)
idx = [i for i, b in enumerate(brs) if b == "bgrat/sw"]
F = xlk.F


def lnx_of(r, mode, cut=0.01):
    """bgrat's ln x: "cut" LN(y) for z > cut else Excel log1p(-z); "LN" LN(y); "lnfl" LN(fl(1 - z)) for z > cut;
    "y1x" Excel log1p(-fl(1 - y)) for 1 - y <= 0.375; "cr" correctly rounded log1p(-z)."""
    tt, u, y, z = tlib.targs(abs(r["t"]), r["n"])
    if mode == "LN" or (mode == "cut" and z > cut):
        return bx.LN(y)
    if mode == "lnfl" and z > cut:
        return bx.LN(1.0 - z)
    if mode == "y1x":
        return xlk.excel_log1p.log1p_series(-(1.0 - y)) if 1.0 - y <= 0.375 else bx.LN(y)
    if mode == "cr":
        return float(mpmath.log1p(-mpmath.mpf(z)))
    return xlk.excel_log1p.log1p_series(-z)


modes = sys.argv[1:] or ["cut"]
CUT = float(modes.pop(0)[4:]) if modes and modes[0].startswith("cut=") else 0.01
sq = {}
for m in modes:
    for zs in ("stored", "x87"):
        pairs, out = [], defaultdict(list)
        need = []
        for i in idx:
            r = rows[i]
            a = r["n"] / 2
            nu = a - 0.25
            lnx = lnx_of(r, m, CUT)
            zz = -nu * lnx if zs == "stored" else bx.sx("*", -nu, lnx)
            s = math.sqrt(zz)
            need.append(s)
        englib.eng("ERFC.PRECISE", need)
        for i, s in zip(idx, need):
            r = rows[i]
            a = r["n"] / 2
            nu = a - 0.25
            lnx = lnx_of(r, m, CUT)
            with mpmath.workdps(50):
                qx = mpmath.erfc(mpmath.sqrt(-mpmath.mpf(nu) * mpmath.mpf(lnx)))
                q = englib.eng("ERFC.PRECISE", [s])[0] if s > 0.5 else 1 - englib.eng("ERF.PRECISE", [s])[0]
                v = float(F(a, lnx) * q / qx) * (1 if r["kind"] == "2t" else 0.5)
            pairs.append((v, r["excel"]))
            out[r["n"]].append((v, r["excel"]))
        print(m, zs, tlib.line("bgrat-only", pairs))
        for n in sorted(out):
            if len(out[n]) > 10:
                print("   ", n, tlib.line("", out[n]))
xlk.save_cache()
