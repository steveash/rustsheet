"""T-2586 r5-f-final: the integrated frozen model of Excel's F.DIST(x, d1, d2, FALSE) (M-final; frozen 2026-10-24
after the hold-out t2586-r5-f-final-hold was filled and before it was read).

Every piece is imported from the round that verified it; this module only fixes their order, so an engine port
can follow `value` line by line.
  rules     r5-f-edges (verified on two hold-outs, every error / 0 / 1 right): df truncated toward zero, a truncated
            df < 1 or > 1E10 is #NUM!; x < 0 #NUM!; x = 0 -> #NUM! / 1 / 0 for trunc(d1) = 1 / 2 / >= 3;
            0 < x < 2^-1022 #DIV/0!; fl(m * x) overflowing #NUM!; EXP(E) stored as a double (a subnormal kept,
            E < -800 gives 0); an F below 2^-1022 flushed to 0.
  side, r   r4 (verified): mx = fl(m x); side 1 (mx <= n): r = fl(mx / n), c = a; side 2: r = fl(n / mx), c = -b.
  P         r4: fl(c * LN(x)) stored (Excel's LN, r2/r3 `R3.LN`).
  H         r4: -h * L(r) kept extended, L = Excel's log1p (14-term atanh series for r <= 0.375, LN(fl(1 + r))
            above; r3-tdensity `log1p_xl`). r5-f-smallr's alternatives (xs1, dd) were refuted by their verifier.
  K         r5-f-bigconst (verified for bit-exact counts): K = K_exact(m, n, side) + delta units of 2^-53, with
            delta = M-sd's side bracket - 0.42 when min(m, n) >= 16 and r4-f-small-df's M-sd delta (Loader
            stirlerr error table + bracket + 5.33) otherwise (`bcmodel.K_of`).
  final     E = K + P + H (extended); Y = fl(EXP_V6(E)); F = fl(Y / x)  (fin "div", the default)
            or F = fl(Y * fl(1 / x))                                 (fin "mulr", most bit-exact in round 4/5).
r5-f-combine's stored-sum order (hkp-sd) was refuted and is not used.
`value(x, d1, d2, fin)` takes Excel's argument values (after literal parsing) and returns a float or an error string.
"""

from __future__ import annotations

import math
import sys
from pathlib import Path

T = Path(__file__).resolve().parent.parent
for _p in (T / "r5-f-bigconst", T / "r5-f-edges"):
    if str(_p) not in sys.path:
        sys.path.insert(0, str(_p))

import bcmodel as BC  # noqa: E402  (r5-f-bigconst: the constant K)
import edgemodel as EM  # noqa: E402  (r5-f-edges: the argument / range rules)

SL = BC.SL  # r4-f-small-df slib: Excel's LN, log1p, EXP V6, side and r
LDB, st = SL.LDB, SL.st
TINY = EM.TINY


def rule(x: float, m: float, n: float):
    """Excel's argument / range verdict for truncated df (m, n): an error string, an exact value, or None (compute)."""
    if m < 1 or n < 1 or m > EM.DF_MAX or n > EM.DF_MAX or x < 0:
        return "#NUM!"
    if x == 0:
        return "#NUM!" if m == 1 else (1.0 if m == 2 else 0.0)
    if x < TINY:
        return "#DIV/0!"
    if EM.mx_overflows(x, m):
        return "#NUM!"
    return None


def exponent(x: float, m: float, n: float) -> LDB:
    """E = K + fl(c LN x) - h L(r), extended (x87 64-bit)."""
    side, r = SL.side_r(x, m, n)
    a, b, h = m / 2, n / 2, (m + n) / 2
    c = a if side == 1 else -b
    P = LDB(st(LDB(c) * SL.LN(x)))
    H = -LDB(h) * SL.L_xl(r)
    return BC.K_of(m, n, side) + P + H


def value(x: float, d1: float, d2: float, fin: str = "div"):
    m, n = float(math.trunc(d1)), float(math.trunc(d2))
    verdict = rule(x, m, n)
    if verdict is not None:
        return verdict
    E = exponent(x, m, n)
    if E < -800:
        return 0.0
    y = SL.EXP(E)  # a double (subnormal kept)
    if fin == "div":
        f = st(LDB(y) / LDB(x))
    else:
        f = st(LDB(y) * LDB(st(LDB(1) / LDB(x))))
    return 0.0 if abs(f) < TINY else f
