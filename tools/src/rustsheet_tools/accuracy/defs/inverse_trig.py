"""ASIN, ACOS, ATAN, ATAN2, ACOT, DEGREES and RADIANS (Gate A slice 4b, T-3669).

The math: each function of the doubles of the written literals, at the working precision.

Sources:
- numbers.md, "Out-of-domain ACOS/ASIN/ACOSH/ATANH are `#NUM!` at once": ASIN and ACOS take [-1, 1].
- numbers.md, "ACOS is PI()/2 - ASIN(x)": Excel's ACOS goes through its ASIN in doubles. That is a formulation of
  Excel's kernel, not a semantic rule, so the definition is the true arccosine (Excel's `ACOS(0.5)` is one ulp below
  the correctly rounded π/3); the cancellation near x = 1 costs at most about 1e-8 relative at the last double below
  1, inside G.
- gen-m5 (excel-graph): `ACOT(-0.1)` is 1.6704649792860584 (Calc!A278), π/2 - atan(-0.1): ACOT's range is (0, π), not
  mpmath's (-π/2, π/2]; numbers.md "Negative zero" has `ACOT(0)` = π/2.
- numbers.md, "ATAN2 is ATAN(y/x) plus or minus π": `ATAN2(x_num, y_num)` forms Excel's quotient y/x. Three rules on
  the doubles follow, all in the definition:
  - `ATAN2(0,0)` is `#DIV/0!`, and a quotient beyond the double range is `#NUM!` (both domain errors);
  - a quotient that underflows below the smallest normal double is 0 (numbers.md "Subnormal results become 0"), so
    the result is 0, or ±π for x < 0 by the sign of y;
  - otherwise the result is the true atan2(y, x) (x = 0 gives ±π/2).
- numbers.md, "RADIANS, SQRTPI and MUNIT": RADIANS is x·π/180; DEGREES is x·180/π (a result beyond the double range
  is redrawn by the generator, as for any overflowing math).
"""

from __future__ import annotations

import random
import sys
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs.annuity import scalars
from rustsheet_tools.accuracy.literal import MAX_LITERAL, Arg
from rustsheet_tools.accuracy.model import Definition, Interval, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

DOMAIN_SOURCE = "numbers.md#out-of-domain-acosasinacoshatanh-are-num-at-once-even-nested-t-1202"
ACOS_SOURCE = "numbers.md#acos-is-pi2---asinx"
ACOT_SOURCE = "gen-m5 ACOT(-0.1) Calc!A278 (excel-graph)"
ATAN2_SOURCE = "numbers.md#atan2-is-atanyx-plus-or-minus-π"
SUBNORMAL_SOURCE = "numbers.md#subnormal-results-become-0"
RADIANS_SOURCE = "numbers.md#radians-sqrtpi-and-munit"
DOUBLE_MAX = mpmath.mpf(sys.float_info.max)
DOUBLE_MIN_NORMAL = mpmath.mpf(sys.float_info.min)


def _one(args: Sequence[MathArg]) -> mpmath.mpf:
    (x,) = scalars(args)
    return x


def _atan2_domain(args: Sequence[MathArg]) -> str | None:
    x, y = scalars(args)
    if x == 0:
        return "ATAN2(0,0) is #DIV/0!" if y == 0 else None
    return "the quotient y/x is beyond the double range" if abs(y / x) > DOUBLE_MAX else None


def _atan2(args: Sequence[MathArg]) -> mpmath.mpf:
    x, y = scalars(args)
    if x != 0 and abs(y / x) < DOUBLE_MIN_NORMAL:  # the quotient flushes to 0
        if x > 0:
            return mpmath.mpf(0)
        return mpmath.pi if y >= 0 else -mpmath.pi
    return mpmath.atan2(y, x)


def _signed(rng: random.Random, lo: float, hi: float) -> float:
    x = log_uniform(rng, lo, hi)
    return -x if rng.random() < 0.5 else x


def _unit_draw(stratum: str, rng: random.Random) -> list[Arg]:
    """ASIN and ACOS: [-1, 1], with the tail near ±1 and 0."""
    if stratum == "tail":
        side = rng.choice((-1.0, 1.0))
        return [
            side * (1 - log_uniform(rng, 1e-15, 1e-2)) if rng.random() < 0.7 else _signed(rng, 1e-300, 1e-8)
        ]
    if stratum == "edge":
        return [rng.choice((-1.0, 1.0, 0.0, 0.5, -0.5, 1.000000000001, -1.5, 2.0))]
    if stratum in ("extreme", "log-uniform"):
        return [_signed(rng, 1e-300, 1.0)]
    return [rng.uniform(-1, 1)]


def _real_draw(stratum: str, rng: random.Random) -> list[Arg]:
    """ATAN, ACOT, DEGREES and RADIANS: any real."""
    if stratum == "tail":
        return [_signed(rng, 1e-300, 1e-6) if rng.random() < 0.5 else _signed(rng, 1e6, 1e300)]
    if stratum == "edge":
        return [rng.choice((0.0, 1.0, -1.0, MAX_LITERAL, -MAX_LITERAL, 2.2250738585072e-308))]
    if stratum == "extreme":
        return [_signed(rng, 1e100, MAX_LITERAL)]
    if stratum == "log-uniform":
        return [_signed(rng, 1e-300, MAX_LITERAL)]
    return [rng.uniform(-100, 100)]


def _atan2_draw(stratum: str, rng: random.Random) -> list[Arg]:
    """ATAN2(x, y): all four quadrants and the axes; the tail at extreme quotients."""
    if stratum == "tail":  # quotients near overflow and underflow
        big, small = _signed(rng, 1e200, MAX_LITERAL), _signed(rng, 1e-300, 1e-100)
        return [big, small] if rng.random() < 0.5 else [small, big]
    if stratum == "edge":
        return list(
            rng.choice(
                ((0.0, 0.0), (0.0, 1.0), (0.0, -2.5), (-1.0, 0.0), (1.0, 0.0), (-1.0, -1.0), (1e-300, 1e300))
            )
        )
    if stratum in ("extreme", "log-uniform"):
        return [_signed(rng, 1e-300, MAX_LITERAL), _signed(rng, 1e-300, MAX_LITERAL)]
    return [rng.uniform(-100, 100), rng.uniform(-100, 100)]


def _one_arg(
    function: str,
    kernel: Callable[[mpmath.mpf], mpmath.mpf],
    domain: Interval,
    draw: Callable[[str, random.Random], list[Arg]],
    sources: tuple[str, ...],
) -> Definition:
    return register(
        Definition(
            function=function,
            params=(Param("number", "real", domain, source=sources[0]),),
            value=lambda args: kernel(_one(args)),
            draw=draw,
            sources=sources,
        )
    )


_UNIT = Interval(-1, 1, lo_open=False, hi_open=False)
ASIN = _one_arg("ASIN", mpmath.asin, _UNIT, _unit_draw, (DOMAIN_SOURCE,))
ACOS = _one_arg("ACOS", mpmath.acos, _UNIT, _unit_draw, (DOMAIN_SOURCE, ACOS_SOURCE))
ATAN = _one_arg("ATAN", mpmath.atan, Interval(), _real_draw, ("catalog",))
ACOT = _one_arg("ACOT", lambda x: mpmath.pi / 2 - mpmath.atan(x), Interval(), _real_draw, (ACOT_SOURCE,))
DEGREES = _one_arg("DEGREES", lambda x: x * 180 / mpmath.pi, Interval(), _real_draw, (RADIANS_SOURCE,))
RADIANS = _one_arg("RADIANS", lambda x: x * mpmath.pi / 180, Interval(), _real_draw, (RADIANS_SOURCE,))

ATAN2 = register(
    Definition(
        function="ATAN2",
        params=(
            Param("x_num", "real", source=ATAN2_SOURCE),
            Param("y_num", "real", source=ATAN2_SOURCE),
        ),
        value=_atan2,
        domain=_atan2_domain,
        draw=_atan2_draw,
        sources=(ATAN2_SOURCE, SUBNORMAL_SOURCE),
    )
)
