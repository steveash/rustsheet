"""T-2586 r5-f-edges: Excel's F.DIST(x, d1, d2, FALSE) over its whole domain (frozen model M-edge).

M-edge = Excel's argument and range rules (measured on probe t2586-r5-f-edges-a, see RESULTS.md) around the value
chain of r4-f-small-df's frozen M-sd (`sdmodel`), which this module imports and does not change:
  1. d1, d2 truncated toward zero; a truncated df below 1 or above 1E10 is #NUM!;
  2. x < 0 is #NUM!;
  3. x = 0: #NUM! for trunc(d1) = 1, exactly 1 for trunc(d1) = 2, 0 for trunc(d1) >= 3 (any d2);
  4. 0 < x < 2^-1022 (a kept subnormal x) is #DIV/0!;
  5. mx = fl(trunc(d1) * x) overflowing a double is #NUM!;
  6. otherwise M-sd's chain: E = K + fl(c LN x) - h L(r) (extended), Y = EXP(E) stored as a double (a subnormal Y
     is kept, with its lost bits; EXP(E) below 2^-1075 is 0), F = fl(Y / x), and an F below 2^-1022 is 0 (flushed).
`value(x, d1, d2, fin)` returns a float or an error string; fin 'div' (M-sd) or 'mulr' (M-sdm)."""

from __future__ import annotations

import math

import elib as E

L, SD = E.L, E.SD
LDB, st = L.LDB, L.st
TINY = 2.0**-1022
DF_MAX = 1e10


def mx_overflows(x: float, m: float) -> bool:
    """fl(m*x) to a double overflows: the x87 product (64-bit, wide exponent) stored to a double."""
    return not math.isfinite(st(LDB(m) * LDB(x)))


def chain(x: float, m: float, n: float, fin: str = "div") -> float:
    """M-sd's value chain with the stored EXP kept as a (possibly subnormal) double and the subnormal flush."""
    side, r = L.side_r(x, m, n)
    a, b, h = m / 2, n / 2, (m + n) / 2
    c = a if side == 1 else -b
    Ex = SD.K_of(m, n, side) + LDB(st(LDB(c) * L.LN(x))) - LDB(h) * L.L_xl(r)
    if Ex < -800:  # far below 2^-1075: EXP V6 is 0 (the x87 scale would leave the int32 range)
        return 0.0
    y = L.EXP(Ex)  # a double; subnormal kept (EXP V6 stored)
    if fin == "div":
        f = st(LDB(y) / LDB(x))
    else:
        f = st(LDB(y) * LDB(st(LDB(1) / LDB(x))))
    return 0.0 if abs(f) < TINY else f


def value(x: float, d1: float, d2: float, fin: str = "div"):
    m, n = float(math.trunc(d1)), float(math.trunc(d2))
    if m < 1 or n < 1 or m > DF_MAX or n > DF_MAX:
        return "#NUM!"
    if x < 0:
        return "#NUM!"
    if x == 0:
        return "#NUM!" if m == 1 else (1.0 if m == 2 else 0.0)
    if x < TINY:
        return "#DIV/0!"
    if mx_overflows(x, m):
        return "#NUM!"
    return chain(x, m, n, fin)
