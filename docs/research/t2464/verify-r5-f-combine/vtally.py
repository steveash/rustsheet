"""verify-r5-f-combine: own tally (bit-exact, within 4 ulp of Excel's value, within half a unit of the 15th
significant digit of Excel's value)."""

from __future__ import annotations

import math
from decimal import Decimal


def w15(m: float, e: float) -> bool:
    if not math.isfinite(m):
        return False
    de = Decimal(e)
    half = Decimal(5).scaleb(de.adjusted() - 15)
    return abs(Decimal(m) - de) <= half


def tally(pairs) -> tuple[int, int, int, int]:
    n = ex = u4 = d15 = 0
    for m, e in pairs:
        n += 1
        ex += m == e
        u4 += math.isfinite(m) and abs(m - e) <= 4 * math.ulp(e)
        d15 += w15(m, e)
    return n, ex, u4, d15


def fmt(t) -> str:
    n, a, b, c = t
    return f"{a}/{b}/{c} of {n}"
