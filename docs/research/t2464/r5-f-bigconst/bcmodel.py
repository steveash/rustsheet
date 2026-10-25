"""T-2586 r5-f-bigconst: the frozen model M-bc = M-sd with the large-shape constant re-pinned (frozen 2026-10-24
before the hold-out t2586-r5-f-bigconst-hold was read).

M-sd's constant offset (r4-f-small-df sdmodel.delta) is e(h) - e(a) - e(b) + C0 + side_err, where side_err is the
rounding of (h - 1/2) ln(h/o) formed as (h - 1/2) * log1p(fl(w/o)) with Excel's log1p. Above v = 28 the e-table is
extrapolated linearly, so e(h) - e(a) - e(b) + C0 is the constant +1.54 units when a, b, h > 28. On 361 shapes with min(d) >= 16
(the x = 1 +- k ulp runs of probe t2586-r5-f-bigconst-a, b4_table.py) the measured offset is side_err - 0.42 (rms 1.27
units over 380 (shape, side) groups), with no e-table part at all: M-bc uses
  delta_bc = side_err + C_BIG (min(d1, d2) >= 16),  M-sd's delta otherwise.
value(x, d1, d2, fin) has M-sd's final steps ('div' = fl(EXP(E)/x), 'mulr' = fl(EXP(E) * fl(1/x)))."""

from __future__ import annotations

import functools

import blib as B

SL, SD = B.SL, B.SD
LDB, st = B.LDB, B.st
C_BIG = -0.42
BIG_MIN = 16


@functools.lru_cache(maxsize=None)
def side_err(m: float, n: float, side: int) -> float:
    """M-sd's bracket alone (units)."""
    return SD.delta(m, n, side) - (SD.e_of((m + n) / 2) - SD.e_of(m / 2) - SD.e_of(n / 2) + SD.C0)


def delta(m: float, n: float, side: int) -> float:
    if min(m, n) >= BIG_MIN:
        return side_err(m, n, side) + C_BIG
    return SD.delta(m, n, side)


@functools.lru_cache(maxsize=None)
def K_of(m: float, n: float, side: int) -> LDB:
    return SL.ld(SL.K_exact(m, n, side)) + LDB(delta(m, n, side)) * LDB(2.0) ** -53


def value(x: float, m: float, n: float, fin: str = "div") -> float:
    side, r = SL.side_r(x, m, n)
    if fin == "div":
        return SL.mxd_K(x, m, n, K_of(m, n, side))
    a, b, h = m / 2, n / 2, (m + n) / 2
    c = a if side == 1 else -b
    E = K_of(m, n, side) + LDB(st(LDB(c) * SL.LN(x))) - LDB(h) * SL.L_xl(r)
    return st(LDB(SL.EXP(E)) * LDB(st(LDB(1) / LDB(x))))
