"""Controls for hx.py on the bgrat-only cells: which ingredient carries the fit.
  F_lnx      RN(F(lnx)) with Excel's lnx (no ERFC ratio)
  F_lny      RN(F(ln y)) exact log of the rounded y (no LN rounding, no ERFC)
  Qcr        RN(F(lnx) * Qcr(fl sqrt fl z) / Q_exact): correctly rounded erfc in place of Excel's
  Qnosqrt    RN(F(lnx) * Q_xl(fl sqrt(z_exact))/Q_exact): z not rounded
  HX         hx.py's model
  even/odd   hold-out halves of HX"""
import math

import mpmath

import englib
import hx
import tlib

rows, idx, F = hx.rows, hx.idx, hx.F
res = {k: [] for k in ("F_lnx", "F_lny", "Qcr", "Qnosqrt", "HX", "HX_hold", "HX_tune")}
for i in idx:
    r = rows[i]
    a = r["n"] / 2
    nu = a - 0.25
    sc = 1 if r["kind"] == "2t" else 0.5
    lnx = hx.lnx_of(r, "cut")
    tt, u, y, z = tlib.targs(abs(r["t"]), r["n"])
    zz = -nu * lnx
    s = math.sqrt(zz)
    with mpmath.workdps(50):
        qx = mpmath.erfc(mpmath.sqrt(-mpmath.mpf(nu) * mpmath.mpf(lnx)))
        qxl = englib.eng("ERFC.PRECISE", [s])[0] if s > 0.5 else 1 - englib.eng("ERF.PRECISE", [s])[0]
        qcr = float(mpmath.erfc(mpmath.mpf(s)))
        s2 = float(mpmath.sqrt(-mpmath.mpf(nu) * mpmath.mpf(lnx)))
        q2 = englib.eng("ERFC.PRECISE", [s2])[0] if s2 > 0.5 else 1 - englib.eng("ERF.PRECISE", [s2])[0]
        f = F(a, lnx)
        lny = mpmath.log(mpmath.mpf(y)) if z > 0.01 else mpmath.log1p(-mpmath.mpf(z))
        res["F_lnx"].append((float(f * sc), r["excel"]))
        res["F_lny"].append((float(F(a, float(lny)) * sc) if False else float(
            (mpmath.betainc(a, 0.5, 0, mpmath.exp(lny), regularized=True)) * sc), r["excel"]))
        res["Qcr"].append((float(f * qcr / qx) * sc, r["excel"]))
        res["Qnosqrt"].append((float(f * q2 / qx) * sc, r["excel"]))
        v = float(f * qxl / qx) * sc
        res["HX"].append((v, r["excel"]))
        res["HX_hold" if r["hold"] else "HX_tune"].append((v, r["excel"]))
for k, p in res.items():
    print(tlib.line(k, p))
