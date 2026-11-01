"""T-3570 r5-bpser-lg: signed ulp error (model - Excel) per a0 step (rows) and x (columns) for one (center, b0) group,
at g = Excel's GAMMALN(a0); also the fractional position of the model's E*s product between doubles (the rounding
phase), to see whether misses sit at near-ties.
usage: grid.py center=0.15 b0=3.37"""
import math
import sys

from mpmath import mpf

import pin
from lib import LN, fl64, sx, EXPX

kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
c, b0 = float(kw.get("center", "0.15")), float(kw.get("b0", "3.37"))
for p in pin.load():
    if abs(p["a0"] - c) > 1e-12 or p["b0"] != b0:
        continue
    row = []
    for x, e in p["cells"]:
        L = sx("+", p["gl"], sx("-", p["glb"], p["glab"]))
        alx = mpf(sx("*", p["a0"], LN(x)))
        E = EXPX(fl64(alx - mpf(L)))
        s = pin.lib.Model.series_w3(p["a0"], p["b0"], x)
        prod = mpf(E) * mpf(s)
        m = sx("*", E, s)
        ph = float((prod - mpf(m)) / mpf(math.ulp(m)))
        row.append(f"{round((m - e) / math.ulp(e)):+d}({ph:+.2f})")
    print(f"{p['a0']!r:22s} " + " ".join(f"{v:>11s}" for v in row))
