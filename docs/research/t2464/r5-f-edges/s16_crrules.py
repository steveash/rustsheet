"""Baseline for the probe tables: Excel's argument rules (edgemodel) with the correctly rounded exact density as the
value (subnormal results flushed to 0, as Excel does). Usage: s16_crrules.py a|hold"""
import math
import sys

import mpmath

import edgemodel as MD
import elib as E
import s4_probe as S4


def cr_rules(x, d1, d2):
    v = MD.value(x, d1, d2)
    if isinstance(v, str) or x == 0:
        return v
    f = float(mpmath.mpf(MD.L.F.f_exact(x, float(math.trunc(d1)), float(math.trunc(d2)))))
    return 0.0 if f < MD.TINY else f


rows = [o for o in S4.MK.rows(sys.argv[1]) if S4.excel_of(o) is not None]
res = [(cr_rules(S4.x_of(o), o["d1"], o["d2"]), S4.excel_of(o)) for o in rows]
print(f"CR+rules all {E.tally(res)}")
for k in dict.fromkeys(o["kind"] for o in rows):
    print(f"   {k:7s} {E.tally([p for p, o in zip(res, rows) if o['kind'] == k])}")
