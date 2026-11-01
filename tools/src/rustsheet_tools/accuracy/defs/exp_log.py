"""EXP, LN, LOG, LOG10, POWER, SQRT and SQRTPI (Gate A slice 4e, T-3672).

The math: each function of the doubles of the written literals, at the working precision. LOG is the two-argument
form (one-argument LOG is LOG10's double, numbers.md "LOG with one argument is LOG10").

Sources (numbers.md):
- "EXP is an x87 2^(a*log2 e) evaluation": Excel's EXP kernel, and `SINH and COSH overflow is #NUM! at once`: EXP's
  result overflows to `#NUM!` (`EXP(2958465)` and `ISODD(EXP(2^53+1))` are `#NUM!`), so its domain is x <= ln(DOUBLE_MAX).
  The exact cut in (709.78, 709.79) is not pinned, so the draws keep out of that band.
- "LN is FLDLN2 then FYL2X" and "LOG with one argument is LOG10": the kernels; the domain is a positive argument.
  Two-argument LOG: base 1 is `#DIV/0!` even for `LOG(1,1)`, base 0 or a negative base `#NUM!` (C36-C39).
- "RADIANS, SQRTPI and MUNIT": SQRTPI is `sqrt(x * PI)` to the bit, `SQRTPI(-1)` is `#NUM!` and `SQRTPI(10^308)` is
  `#NUM!` (x·π overflows), while `SQRTPI(1E+307)` is a number. The draws keep out of the band around DOUBLE_MAX/π.
- POWER, from "POWER and `^`: odd roots of negatives, zero and overflow", "Non-integer exponents: exp(y·ln x)",
  "Integer exponents beyond 360, subnormal results, base 10" and "POWER when y·ln x overflows":
  - `POWER(0,0)` is `#NUM!`, 0 to a negative power `#DIV/0!`, 0 to a positive power 0;
  - a negative base with a non-integer exponent is `#NUM!` unless 1/y, rounded to 15 significant digits, is an odd
    integer below 2^31 - 1 in magnitude; then the value is minus the power of the magnitude (an odd root);
  - a negative base with an integer exponent of magnitude 2^32 - 1 or more is `#NUM!`;
  - the positive power |x|^|y| decides the rest: if it overflows, a positive exponent is `#NUM!` (a negative one is
    1 over it, below the absolute floor); if it is below the smallest normal double, a negative exponent is `#DIV/0!`
    (a positive one is flushed to 0, inside the floor);
  - for a positive base, a double product |y|·ln x that overflows counts as an overflow of the positive power, so a
    positive exponent is `#NUM!` even for a base below 1 (`POWER(0.1,9.99999999999999E+307)`).
  Excel's kernels (square-and-multiply, exp(y·ln x), the base-10 rule) are kernel detail, not semantic rules. The
  draws keep the positive power away from the two range edges, where Excel's own rounding decides the side.
"""

from __future__ import annotations

import math
import random
import sys
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs.annuity import scalars
from rustsheet_tools.accuracy.literal import MAX_LITERAL, Arg
from rustsheet_tools.accuracy.model import Definition, Interval, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

EXP_SOURCE = "numbers.md#exp-is-an-x87-2a-log2-e-evaluation-not-correctly-rounded-t-1778"
EXP_OVERFLOW_SOURCE = "numbers.md#sinh-and-cosh-overflow-is-num-at-once-sech-there-is-0-t-2169"
LN_SOURCE = "numbers.md#ln-is-fldln2-then-fyl2x"
LOG_SOURCE = "numbers.md#log-with-one-argument-is-log10"
SQRTPI_SOURCE = "numbers.md#radians-sqrtpi-and-munit"
POWER_SOURCE = "numbers.md#power-and--odd-roots-of-negatives-zero-and-overflow"
POWER_RANGE_SOURCE = "numbers.md#integer-exponents-beyond-360-subnormal-results-base-10-t-1311"
POWER_HUGE_SOURCE = "numbers.md#power-when-yln-x-overflows-t-2661"

LN_MAX = math.log(sys.float_info.max)  # 709.782712893384
LN_MIN = math.log(sys.float_info.min)  # -708.3964185322641, the smallest normal double
EXP_BAND = (709.78, 709.79)  # EXP's overflow cut lies in here; the draws keep out
SQRTPI_CUT = sys.float_info.max / math.pi  # 5.72E+307: x·π overflows above it
SQRTPI_BAND = (SQRTPI_CUT * (1 - 1e-3), SQRTPI_CUT * (1 + 1e-3))  # Excel's product rounding decides inside
ROOT_LIMIT = 2**31 - 1  # 1/y must be an odd integer below this for a negative base's root
NEGATIVE_BASE_LIMIT = 2**32 - 1  # a negative base with an integer exponent from here is #NUM!
EDGE_MARGIN = 1e-6  # relative distance of ln|x|^|y| from the range edges that a drawn point keeps


def _exp_domain(args: Sequence[MathArg]) -> str | None:
    (x,) = scalars(args)
    return "EXP overflows" if x > LN_MAX else None


def _log_domain(args: Sequence[MathArg]) -> str | None:
    _, base = scalars(args)
    return "base 1 is #DIV/0!" if base == 1 else None


def _sqrtpi_domain(args: Sequence[MathArg]) -> str | None:
    (x,) = scalars(args)
    return "x·π overflows" if x > SQRTPI_CUT else None


def _odd_root(y: float) -> bool:
    """A negative base's non-integer exponent that Excel takes as an odd root: 1/y at 15 digits is odd and small."""
    q = float(f"{1 / y:.15g}")
    return q.is_integer() and abs(q) < ROOT_LIMIT and int(q) % 2 == 1


def _positive_power_log(x: float, y: float) -> mpmath.mpf:
    """ln(|x|^|y|), the positive power's logarithm, or +inf for the T-2661 product overflow."""
    if x > 0 and x != 1 and math.isinf(abs(y) * math.log(x)):
        return mpmath.inf  # numbers.md (T-2661): counts as an overflow even for a base below 1
    return abs(mpmath.mpf(y)) * mpmath.log(abs(mpmath.mpf(x)))


def _zero_or_negative_base(x: float, y: float) -> str | None:
    """POWER's rules for a zero or negative base (numbers.md, T-0109, T-0959, T-1311)."""
    if x == 0:
        if y == 0:
            return "POWER(0,0) is #NUM!"
        return "0 to a negative power is #DIV/0!" if y < 0 else None
    if x < 0 and not y.is_integer() and not _odd_root(y):
        return "a negative base with a non-integer exponent that is not an odd root"
    if x < 0 and y.is_integer() and abs(y) >= NEGATIVE_BASE_LIMIT:
        return "a negative base with an exponent of magnitude 2^32 - 1 or more"
    return None


def _out_of_range(x: float, y: float) -> str | None:
    """POWER's overflow (`#NUM!`) and the underflow of a negative exponent's positive power (`#DIV/0!`)."""
    if x == 0 or y == 0 or abs(x) == 1:
        return None
    log_positive_power = _positive_power_log(x, y)
    if log_positive_power > LN_MAX and y > 0:
        return "the power overflows"
    if log_positive_power < LN_MIN and y < 0:
        return "the positive power underflows, so 1 over it is #DIV/0!"
    return None


def _power_domain(args: Sequence[MathArg]) -> str | None:
    xm, ym = scalars(args)
    x, y = float(xm), float(ym)
    return _zero_or_negative_base(x, y) or _out_of_range(x, y)


def _power(args: Sequence[MathArg]) -> mpmath.mpf:
    xm, ym = scalars(args)
    x, y = float(xm), float(ym)
    if x == 0:
        return mpmath.mpf(0)
    magnitude = mpmath.power(abs(xm), ym)
    if x > 0:
        return magnitude
    if y.is_integer():
        return magnitude if int(y) % 2 == 0 else -magnitude
    return -magnitude  # an odd root of a negative base


def _signed(rng: random.Random, magnitude: float) -> float:
    return -magnitude if rng.random() < 0.5 else magnitude


def _away_from(x: float, band: tuple[float, float]) -> float:
    """Move a draw out of an unpinned band to its probed lower end."""
    return math.copysign(band[0], x) if band[0] < abs(x) < band[1] else x


def _exp_draw(stratum: str, rng: random.Random) -> list[Arg]:
    if stratum == "tail":  # toward the overflow and the underflow
        x = rng.uniform(600, 709.78) if rng.random() < 0.5 else -rng.uniform(700, 760)
    elif stratum == "edge":
        x = rng.choice((0.0, 1.0, -1.0, 709.78, 710.0, -745.0, -1000.0, 1e-300, 2958465.0, MAX_LITERAL))
    elif stratum == "extreme":
        x = _signed(rng, log_uniform(rng, 10, 709.78))
    elif stratum == "log-uniform":
        x = _signed(rng, log_uniform(rng, 1e-300, 709.78))
    else:
        x = rng.uniform(-20, 20)
    return [_away_from(x, EXP_BAND)]


def _log_like_draw(stratum: str, rng: random.Random, *, near_one: bool) -> float:
    """LN, LOG10, SQRT and SQRTPI's argument (and LOG's number): positive, except the edge stratum's 0 and -1."""
    if stratum == "tail":
        if near_one:
            return 1 + _signed(rng, 10 ** -rng.uniform(1, 14))
        return log_uniform(rng, 1e-300, 1e-100)
    if stratum == "edge":
        return rng.choice((0.0, -1.0, 1.0, 1e-300, MAX_LITERAL, 1e307, 1e-15, 10.0))
    if stratum in ("extreme", "log-uniform"):
        return log_uniform(rng, 1e-300, MAX_LITERAL)
    return rng.uniform(0, 100)


def _ln_draw(stratum: str, rng: random.Random) -> list[Arg]:
    return [_log_like_draw(stratum, rng, near_one=True)]


def _sqrt_draw(stratum: str, rng: random.Random) -> list[Arg]:
    # SQRT and SQRTPI share it; SQRT has no cut near SQRTPI's band, so keeping out of it costs SQRT nothing
    return [_away_from(_log_like_draw(stratum, rng, near_one=False), SQRTPI_BAND)]


def _log_draw(stratum: str, rng: random.Random) -> list[Arg]:
    if stratum == "edge":
        return list(
            rng.choice(((100.0, 1.0), (1.0, 1.0), (8.0, 0.0), (8.0, -2.0), (1000.0, 10.0), (8.0, 2.0)))
        )
    number = _log_like_draw(stratum, rng, near_one=True)
    if stratum == "tail" and rng.random() < 0.5:  # a base near 1
        return [number, 1 + _signed(rng, 10 ** -rng.uniform(1, 14))]
    if stratum in ("extreme", "log-uniform"):
        base = log_uniform(rng, 1e-300, MAX_LITERAL)
    else:
        base = rng.uniform(0.1, 20)
    return [number, base if base != 1 else 2.0]


def _near_power_edge(x: float, y: float) -> bool:
    """A positive power within EDGE_MARGIN (relative) of the overflow or the smallest normal double."""
    if x == 0 or y == 0 or abs(x) == 1:
        return False
    log_positive_power = abs(y) * math.log(abs(x))
    return any(abs(log_positive_power - edge) <= EDGE_MARGIN * abs(edge) for edge in (LN_MAX, LN_MIN))


_POWER_EDGES: tuple[tuple[float, float], ...] = (
    (0.0, 0.0),  # #NUM!
    (0.0, -1.0),  # #DIV/0!
    (0.0, 2.5),
    (-8.0, 1 / 3),  # an odd root: -2
    (-8.0, 0.5),  # #NUM!
    (-8.0, -1 / 3),
    (10.0, 309.0),  # overflow: #NUM!
    (2.0, 1024.0),
    (1e-10, -31.0),  # the positive power 1E-310 underflows: #DIV/0!
    (0.1, -400.0),
    (0.1, 9.99999999999999e307),  # T-2661: #NUM!
    (1.0, 1e300),
    (-1.0, 4294967295.0),  # #NUM!
    (-1.0, 4294967294.0),
    (-2.0, 3.0),
    (10.0, 23.0),
    (2.0, 0.5),
)


def _power_draw(stratum: str, rng: random.Random) -> list[Arg]:
    if stratum == "edge":
        return list(rng.choice(_POWER_EDGES))
    for _ in range(100):  # redraw a point that lands next to a range edge (a few draws at most)
        x, y = _power_pair(stratum, rng)
        if not _near_power_edge(x, y):
            return [x, y]
    return [2.0, 0.5]  # unreachable in practice: a stratum's pair lands near an edge with probability ~1e-6


def _power_pair(stratum: str, rng: random.Random) -> tuple[float, float]:
    if stratum == "tail":
        kind = rng.randrange(3)
        if kind == 0:  # an odd root of a negative base
            return -log_uniform(rng, 1e-3, 1e3), _signed(rng, 1 / rng.choice((3, 5, 7, 9, 11, 101)))
        if kind == 1:  # a negative base to an integer power
            return -rng.uniform(0.5, 10), float(rng.randint(-60, 60))
        # a base near 1 to a large power, the power within the double range
        d = 10 ** -rng.uniform(3, 14)
        y = _signed(rng, rng.uniform(0.1, 600) / d)
        return 1 + _signed(rng, d), y
    if stratum in ("extreme", "log-uniform"):
        x = log_uniform(rng, 1e-300, 1e300) if stratum == "extreme" else log_uniform(rng, 1e-10, 1e10)
        span = 700 / max(abs(math.log(x)), 1e-3)  # keeps |y·ln x| below about 700
        return x, _signed(rng, log_uniform(rng, min(1e-6, span), span))
    return rng.uniform(0, 10), rng.uniform(-10, 10)


def _one(
    function: str,
    kernel: Callable[[mpmath.mpf], mpmath.mpf],
    domain: Interval,
    draw: Callable[[str, random.Random], list[Arg]],
    sources: tuple[str, ...],
    *,
    check: Callable[[Sequence[MathArg]], str | None] | None = None,
) -> Definition:
    def value(args: Sequence[MathArg]) -> mpmath.mpf:
        (x,) = scalars(args)
        return kernel(x)

    return register(
        Definition(
            function=function,
            params=(Param("number", "real", domain, source=sources[0]),),
            value=value,
            domain=check,
            draw=draw,
            sources=sources,
        )
    )


_POSITIVE = Interval(0, lo_open=True)
_NON_NEGATIVE = Interval(0, lo_open=False)

EXP = _one("EXP", mpmath.exp, Interval(), _exp_draw, (EXP_OVERFLOW_SOURCE, EXP_SOURCE), check=_exp_domain)
LN = _one("LN", mpmath.log, _POSITIVE, _ln_draw, (LN_SOURCE,))
LOG10 = _one("LOG10", mpmath.log10, _POSITIVE, _ln_draw, (LOG_SOURCE,))
SQRT = _one("SQRT", mpmath.sqrt, _NON_NEGATIVE, _sqrt_draw, ("catalog",))
SQRTPI = _one(
    "SQRTPI",
    lambda x: mpmath.sqrt(x * mpmath.pi),
    _NON_NEGATIVE,
    _sqrt_draw,
    (SQRTPI_SOURCE,),
    check=_sqrtpi_domain,
)


def _log(args: Sequence[MathArg]) -> mpmath.mpf:
    number, base = scalars(args)
    return mpmath.log(number) / mpmath.log(base)


LOG = register(
    Definition(
        function="LOG",
        params=(
            Param("number", "real", _POSITIVE, source=LOG_SOURCE),
            Param("base", "real", _POSITIVE, source=LOG_SOURCE),
        ),
        value=_log,
        domain=_log_domain,
        draw=_log_draw,
        sources=(LOG_SOURCE,),
    )
)

POWER = register(
    Definition(
        function="POWER",
        params=(
            Param("number", "real", source=POWER_SOURCE),
            Param("power", "real", source=POWER_SOURCE),
        ),
        value=_power,
        domain=_power_domain,
        draw=_power_draw,
        sources=(POWER_SOURCE, POWER_RANGE_SOURCE, POWER_HUGE_SOURCE),
    )
)
