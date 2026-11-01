"""ASINH, ACOSH, ATANH and ACOTH (Gate A slice 4d, T-3671).

The math: each function of the double of the written literal, at the working precision.

Sources:
- numbers.md, "Out-of-domain ACOS/ASIN/ACOSH/ATANH are `#NUM!` at once" (T-1202, T-1569): ACOSH below 1 is `#NUM!`;
  `ATANH(1)`, `ATANH(-1)`, `ATANH(1.0000000000000002)`, `ACOTH(1)` and `ACOTH(-1)` are `#NUM!`. So ATANH takes (-1, 1)
  and ACOTH |x| > 1.
- numbers.md, "Inverse hyperbolics, COTH/CSCH and the SINH/TANH thresholds" (T-1005, T-2062): ASINH and ACOSH are
  `#NUM!` where `x*x` overflows a double (|x| above about 1.3407807929942596E154; 1.3E154 is finite, 1.35E154 is
  `#NUM!`), so that overflow is part of their domain. Its exact position is inferred, not probed, so the draws keep out
  of [1.3E154, 1.35E154].
- The same section: ASINH and ACOSH are the plain log formulas `ln(x + sqrt(x*x +- 1))` in doubles, so small ASINH
  arguments lose digits (`ASINH(1E-10)` is 8e-8 relative off), and below about 1E-16 the sum rounds to 1 and Excel's
  value is 0. That is the inaccuracy of Excel's formula, not a semantic rule, so the definition is the true asinh. Such
  points are R3-eligible under decision 0188 but left unmarked (no owner ruling; the engine keeps Excel's formula,
  T-2031). ATANH's and ACOTH's x87 half-log and series kernels (T-0917) are likewise kernel detail.
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs.annuity import scalars
from rustsheet_tools.accuracy.literal import MAX_LITERAL, Arg
from rustsheet_tools.accuracy.model import Definition, Interval, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

DOMAIN_SOURCE = "numbers.md#out-of-domain-acosasinacoshatanh-are-num-at-once-even-nested-t-1202"
FORMULA_SOURCE = "numbers.md#inverse-hyperbolics-cothcsch-and-the-sinhtanh-thresholds-oracle-sweep-t-1005"
SQUARE_BAND = (1.3e154, 1.35e154)  # x*x overflows in here (inferred, unprobed); the draws keep out
SQUARE_SAFE = SQUARE_BAND[0]


def _square_overflows(args: Sequence[MathArg]) -> str | None:
    """ASINH and ACOSH: `#NUM!` once x*x overflows a double (the formula's own square)."""
    (x,) = scalars(args)
    return "x*x overflows a double" if math.isinf(float(x) * float(x)) else None


def _outside_unit(args: Sequence[MathArg]) -> str | None:
    """ACOTH: |x| > 1 (`ACOTH(1)`, `ACOTH(-1)` and |x| < 1 are `#NUM!`)."""
    (x,) = scalars(args)
    return "ACOTH needs |x| > 1" if abs(x) <= 1 else None


def _signed(rng: random.Random, magnitude: float) -> float:
    return -magnitude if rng.random() < 0.5 else magnitude


def _outside_band(x: float) -> float:
    """Move a draw out of the unprobed band of the x*x overflow, to its probed lower end."""
    return math.copysign(SQUARE_SAFE, x) if SQUARE_BAND[0] < abs(x) < SQUARE_BAND[1] else x


def _near_one(rng: random.Random) -> float:
    """1 + 10^-k or 1 - 10^-k territory, k up to 14 (15 significant digits)."""
    return 10 ** -rng.uniform(1, 14)


def _asinh_draw(stratum: str, rng: random.Random) -> list[Arg]:
    if stratum == "tail":  # tiny arguments, where Excel's log formula loses digits, and toward the overflow
        x = log_uniform(rng, 1e-300, 1e-4) if rng.random() < 0.7 else log_uniform(rng, 1e100, SQUARE_SAFE)
    elif stratum == "edge":
        x = rng.choice((0.0, 1.0, 1e-10, 1e-20, 1e154, 1.3e154, 1.35e154, 1e200, MAX_LITERAL))
    elif stratum == "extreme":
        x = log_uniform(rng, 1e10, SQUARE_SAFE)
    elif stratum == "log-uniform":
        x = log_uniform(rng, 1e-300, SQUARE_SAFE)
    else:
        x = rng.uniform(0, 10)
    return [_outside_band(_signed(rng, x))]


def _acosh_draw(stratum: str, rng: random.Random) -> list[Arg]:
    if stratum == "tail":  # just above 1, where the log formula cancels, and toward the overflow
        x = 1 + _near_one(rng) if rng.random() < 0.7 else log_uniform(rng, 1e100, SQUARE_SAFE)
    elif stratum == "edge":
        x = rng.choice(
            (1.0, 0.999999999999999, 0.0, -1.0, 1.00000000000001, 1e154, 1.3e154, 1.35e154, MAX_LITERAL)
        )
    elif stratum == "extreme":
        x = log_uniform(rng, 1e10, SQUARE_SAFE)
    elif stratum == "log-uniform":
        x = log_uniform(rng, 1.000001, SQUARE_SAFE)
    else:
        x = rng.uniform(1, 10)
    return [_outside_band(x)]


def _atanh_draw(stratum: str, rng: random.Random) -> list[Arg]:
    if stratum == "tail":  # near ±1, and the series/log switch near 1E-4 (T-0917)
        x = 1 - _near_one(rng) if rng.random() < 0.7 else rng.uniform(5e-5, 2e-4)
    elif stratum == "edge":
        x = rng.choice((1.0, 0.0, 0.999999999999999, 1.00000000000001, 1e-4, 1.1e-4, 2.0, 1e-300))
    elif stratum in ("extreme", "log-uniform"):
        x = log_uniform(rng, 1e-300, 0.999999999999999)
    else:
        x = rng.uniform(0, 1)
    return [_signed(rng, x)]


def _acoth_draw(stratum: str, rng: random.Random) -> list[Arg]:
    if stratum == "tail":  # near ±1, and the log/series switch near 3.69665 (T-0917)
        x = 1 + _near_one(rng) if rng.random() < 0.7 else rng.uniform(3.6, 3.8)
    elif stratum == "edge":
        x = rng.choice((1.0, 0.5, 0.0, 1.00000000000001, 3.69665, MAX_LITERAL, 2.0))
    elif stratum == "extreme":
        x = log_uniform(rng, 1e10, MAX_LITERAL)
    elif stratum == "log-uniform":
        x = log_uniform(rng, 1.000001, MAX_LITERAL)
    else:
        x = rng.uniform(1.000001, 10)
    return [_signed(rng, x)]


def _definition(
    function: str,
    kernel: Callable[[mpmath.mpf], mpmath.mpf],
    domain: Interval,
    draw: Callable[[str, random.Random], list[Arg]],
    sources: tuple[str, ...],
    *,
    domain_check: Callable[[Sequence[MathArg]], str | None] | None = None,
) -> Definition:
    def value(args: Sequence[MathArg]) -> mpmath.mpf:
        (x,) = scalars(args)
        return kernel(x)

    return register(
        Definition(
            function=function,
            params=(Param("number", "real", domain, source=sources[0]),),
            value=value,
            domain=domain_check,
            draw=draw,
            sources=sources,
        )
    )


ASINH = _definition(
    "ASINH", mpmath.asinh, Interval(), _asinh_draw, (FORMULA_SOURCE,), domain_check=_square_overflows
)
ACOSH = _definition(
    "ACOSH",
    mpmath.acosh,
    Interval(1, lo_open=False),
    _acosh_draw,
    (DOMAIN_SOURCE, FORMULA_SOURCE),
    domain_check=_square_overflows,
)
ATANH = _definition("ATANH", mpmath.atanh, Interval(-1, 1), _atanh_draw, (DOMAIN_SOURCE,))
ACOTH = _definition(
    "ACOTH", mpmath.acoth, Interval(), _acoth_draw, (DOMAIN_SOURCE,), domain_check=_outside_unit
)
