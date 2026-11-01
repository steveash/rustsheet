"""T-2464 shared helpers: Excel's own elementary functions, evaluated by the engine, and exact x87 rounding.

The engine's EXP (x87_exp.rs, model V6), LN (x87_log10.rs, FLDLN2/FYL2X), GAMMALN (gammaln.rs, T-1941) and GAMMA
(T-2129) are bit-exact models of Excel's. `excel(name, xs)` evaluates `=NAME(x)` for each x in one recalculation.
Run with bindings/python/.venv/bin/python (make py-bindings builds it).

x87 helpers: `fl64(v)` rounds an mpmath value to a 64-bit significand, `fl53(v)` to a double, and `x87(op, a, b)`
is fl53(fl64(a op b)), the stored x87 operation of the playbook (section 1, item 1).
"""
from __future__ import annotations

import math
from functools import lru_cache


import mpmath

mpmath.mp.prec = 300


def round_bits(v: mpmath.mpf, bits: int) -> mpmath.mpf:
    if v == 0:
        return mpmath.mpf(0)
    s, man, exp, bc = v._mpf_
    if bc <= bits:
        return v
    sh = bc - bits
    q = man >> sh
    r = man - (q << sh)
    half = 1 << (sh - 1)
    if r > half or (r == half and q & 1):
        q += 1
    return mpmath.mpf((s, q, exp + sh, q.bit_length()))


def fl64(v) -> mpmath.mpf:
    return round_bits(mpmath.mpf(v), 64)


def fl53(v) -> float:
    return float(round_bits(mpmath.mpf(v), 53))


def x87(op: str, a, b) -> float:
    """fl53(fl64(a op b)) for doubles a, b (op in + - * /)."""
    A, B = mpmath.mpf(a), mpmath.mpf(b)
    r = {"+": A + B, "-": A - B, "*": A * B, "/": A / B}[op]
    return fl53(fl64(r))


def ulps(got: float, want: float) -> float:
    """Signed distance in ulps of `want`."""
    if got == want:
        return 0.0
    return (got - want) / math.ulp(want) if want != 0 else math.inf


def excel(name: str, xs: list[float]) -> list[float]:
    """Excel's NAME(x) for each double x: NAME in EXP, LN, LOG10, GAMMALN, GAMMA (the engine's bit-exact models),
    evaluated as `=NAME(x)` through the engine's python bindings (`import rustsheet`), one workbook per call. Each x
    is set as a number, so the engine sees the exact double (T-2977 replaced the T-2464 test harness with this)."""
    import rustsheet

    m = rustsheet.create("excelmath")
    for i, x in enumerate(xs):
        m.update_cell_with_number(0, i + 1, 1, float(x))
        m.update_cell_with_formula(0, i + 1, 2, f"={name}(A{i + 1})")
    m.evaluate()
    values = [m.get_cell_value(0, i + 1, 2) for i in range(len(xs))]
    # An error (#NUM! at LN(0), say) comes back as text: NaN, as the harness had no error values either.
    return [float(v) if isinstance(v, (int, float)) else math.nan for v in values]


@lru_cache(maxsize=None)
def excel1(name: str, x: float) -> float:
    return excel(name, [x])[0]
