"""bup + bgrat at b = 1/2 (n <= 30): w1 = bup(a, 1/2, y, x0, 20) + bgrat(a + 20, 1/2, y, x0, w1).
bgrat part G = F(a+20, lnx) * Q_xl / Q_exact (hx.py's identity, nu = a + 19.75), lnx = LN(y) (x0 > 0.01).
bup part candidates:
  best      r2-bratio BEST bup (w4 arithmetic, Excel pdf prefactor), x0 = z
  best1y    the same with x0 = fl(1 - y)
  exact_y   exact I_y(a,1/2) - I_y(a+20,1/2) at the rounded y (1 - y exact)
  exact_lnx exact difference at ln y = lnx (F(a, lnx) - F(a+20, lnx))
Sums: RN(bup + G) (x87/exact sum of a double bup and the ideal G) and RN(F(a,lnx) * ... ) the all-ideal value
  ideal     RN(F(a, lnx) - F(a+20,lnx) + G)."""
import math
from collections import defaultdict

import mpmath

import englib
import hx
import tlib
import tscore
import xlk

bx = tlib.bx
rows = hx.rows
SPEC = hx.SPEC
vals, brs = hx.vals, hx.brs
idx = [i for i, b in enumerate(brs) if b == "bup+bgrat/sw"]
cfg = tscore.parse(SPEC)[1]
K = bx._KERNELS[cfg]


def Q(s):
    return englib.eng("ERFC.PRECISE", [s])[0] if s > 0.5 else 1 - englib.eng("ERF.PRECISE", [s])[0]


need = []
for i in idx:
    r = rows[i]
    a = r["n"] / 2 + 20
    lnx = hx.lnx_of(r, "cut")
    need.append(math.sqrt(-(a - 0.25) * lnx))
englib.eng("ERFC.PRECISE", need)
englib.eng("ERF.PRECISE", need)
res = defaultdict(list)
for i in idx:
    r = rows[i]
    n = r["n"]
    a = n / 2
    sc = 1 if r["kind"] == "2t" else 0.5
    tt, u, y, z = tlib.targs(abs(r["t"]), n)
    lnx = hx.lnx_of(r, "cut")
    nu = a + 20 - 0.25
    zz = -nu * lnx
    with mpmath.workdps(50):
        qx = mpmath.erfc(mpmath.sqrt(-mpmath.mpf(nu) * mpmath.mpf(lnx)))
        G = hx.F(a + 20, lnx) * Q(math.sqrt(zz)) / qx
        bups = {
            "best": K.bup(a, 0.5, y, z, 20, bx.Kernel.EPS),
            "best1y": K.bup(a, 0.5, y, 1.0 - y, 20, bx.Kernel.EPS),
            "exact_y": mpmath.betainc(a, 0.5, 0, y, regularized=True) - mpmath.betainc(a + 20, 0.5, 0, y, regularized=True),
            "exact_lnx": hx.F(a, lnx) - hx.F(a + 20, lnx),
        }
        for k, b in bups.items():
            v = float(mpmath.mpf(b) + G) * sc
            res[k].append((v, r["excel"]))
            res[(k, "hold" if r["hold"] else "tune")].append((v, r["excel"]))
            v2 = float(mpmath.mpf(b) + float(G)) * sc
            res[(k, "RN(G)")].append((v2, r["excel"]))
print("model in this branch:", tlib.line("cut0.01 spec", [(vals[i], rows[i]["excel"]) for i in idx]))
for k, p in res.items():
    print(tlib.line(str(k), p))
