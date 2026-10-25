"""The underflow runs (kind thresh): per row Y = Excel's F times x in units of 2^-1074 (exact rational), the model's
EXP(E) in the same units, and Excel/model - 1 in units of 2^-53. Usage: s5_thresh.py [KIND]"""
import sys
from fractions import Fraction

import edgemodel as MD
import edgeprobe as MK
import s4_probe as S4

kind = sys.argv[1] if len(sys.argv) > 1 else "thresh"
Q = Fraction(2) ** -1074
for o in MK.rows("a"):
    if o["kind"] != kind:
        continue
    e = S4.excel_of(o)
    m = MD.value(o["x"], o["d1"], o["d2"])
    if not isinstance(e, float) or e == 0:
        print(f"{o['row']} x={o['x']:.4g} {o['d1s']},{o['d2s']} excel={e!r} model={m!r}")
        continue
    Y = Fraction(e) * Fraction(o["x"]) / Q
    du = float((Fraction(e) / Fraction(m) - 1) * 2**53) if m else float("nan")
    ex = MK._lnf(o["x"], o["d1"], o["d2"])
    print(f"{o['row']} x={o['x']:.4g} {o['d1s']},{o['d2s']} Fx/2^-1074={float(Y):.6g} xl/model-1={du:9.1f}u  lnF={ex:.2f}")
