"""Needed constant per cell under r4's power-term forms (slib.chain): D = (E_needed - A - K_exact) in units, the
centre of Excel's admissible interval, for rows of a probe kind. Constant D across a shape's x is the right form.
Usage: s7_forms.py KIND FORM [FORM ...]   (forms: xd, xn, xn64, x, r, rdiv, mx, mx1, exact)"""
import math
import sys

import edgeprobe as MK
import elib as E
import s4_probe as S4

kind, forms = sys.argv[1], sys.argv[2:]
for o in MK.rows("a"):
    if o["kind"] != kind:
        continue
    e, x = S4.excel_of(o), S4.x_of(o)
    if not isinstance(e, float) or e == 0 or x <= 0:
        continue
    m, n = float(math.trunc(o["d1"])), float(math.trunc(o["d2"]))
    out = []
    for f in forms:
        A, Kex, div = E.L.chain(x, m, n, f)
        iv = E.e_needed(x, e, "div" if div else "exp")
        if iv is None:
            out.append(f"{f}=  none")
            continue
        E0, lo, hi = iv
        Dc = (E.L.mpmath.mpf(E0.as_integer_ratio()[0]) / E0.as_integer_ratio()[1] - E.L.mpf(A.as_integer_ratio()[0]) / A.as_integer_ratio()[1] - Kex) / E.L.U
        out.append(f"{f}={float(Dc) + (lo + hi) / 2:8.1f}±{(hi - lo) / 2:5.1f}")
    print(f"{o['row']:5d} ({m:g},{n:g}) x={x:.3g} lnx={math.log(x):8.2f} " + " ".join(out))
