"""bup + bgrat at (n/2, 1/2), n <= 30: what Excel's bup part looks like once bgrat's part is the solved identity.
I = RN(B + G), G = bgrat_half_ideal(a + 20, lnx) (lnx = LN(y)). Candidates for B = I_y(a,1/2) - I_y(a+20,1/2):
  w4        r2-bratio BEST's bup (Excel density * y * z / a, w4's stored W arithmetic)
  pdfW      RN-free: P * y * z / a * W_exact, P = Excel density (w1 M1 via r2-bratio) as a double, W exact
  pdfW_1y   the same with z -> fl(1 - y)
  exactpre  exact prefactor y^a z^(1/2) / (a B(a, 1/2)) at the double y, z, W exact
  lnpre     prefactor exp(a LN(y) + LN(z)/2 - lnB) exact arithmetic on Excel's LN values
usage: bup_lab.py"""
from collections import defaultdict

import mpmath

import tlib
import tscore
import xlk

bx = tlib.bx
SPEC = "args=t2,alnrel=xl,erfc=xl,bg_lnx=cut0.01,bg=ideal"
rows = tscore.rows_L()
vals, brs = tscore.evaluate(rows, SPEC)
idx = [i for i, b in enumerate(brs) if b == "bup+bgrat/sw"]
cfg = tscore.parse(SPEC)[1]
K = bx._KERNELS[cfg]
out = defaultdict(list)
for i in idx:
    r = rows[i]
    n = r["n"]
    a = n / 2
    sc = 1 if r["kind"] == "2t" else 0.5
    tt, u, y, z = tlib.targs(abs(r["t"]), n)
    with mpmath.workdps(50):
        G = xlk.bgrat_half_ideal(a + 20, bx.LN(y))
        Y, Z = mpmath.mpf(y), mpmath.mpf(z)
        W = sum(mpmath.rf(a + 0.5, k) / mpmath.rf(a + 1, k) * Y**k for k in range(20))  # sum_{k<20} (a+b)_k/(a+1)_k y^k
        P, _ = K.density(y, a, 0.5)
        lnB = mpmath.log(mpmath.beta(a, 0.5))
        c = {
            "w4": mpmath.mpf(K.bup(a, 0.5, y, z, 20, bx.Kernel.EPS)),
            "pdfW": mpmath.mpf(P) * Y * Z / a * W,
            "pdfW_1y": mpmath.mpf(P) * Y * mpmath.mpf(1.0 - y) / a * W,
            "exactpre": mpmath.exp(a * mpmath.log(Y) + mpmath.log(Z) / 2 - lnB) / a * W,
            "lnpre": mpmath.exp(a * mpmath.mpf(bx.LN(y)) + mpmath.mpf(bx.LN(z)) / 2 - lnB) / a * W,
        }
        for k, B in c.items():
            v = float((B + G) * sc)
            out[k].append((v, r["excel"]))
            out[(k, "n=%g" % n)].append((v, r["excel"]))
xlk.save_cache()
for k in sorted(out, key=str):
    print(tlib.line(str(k), out[k]))
