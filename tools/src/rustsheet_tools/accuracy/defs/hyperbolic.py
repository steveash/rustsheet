"""SINH, COSH, TANH, COTH, SECH and CSCH (Gate A slice 4c, T-3670).

The math: each function of the double of the written literal, at the working precision (COTH, SECH and CSCH are 1 over
TANH, COSH and SINH).

Sources:
- numbers.md, "SINH and COSH overflow is `#NUM!` at once; SECH there is 0" (T-2169): SINH and COSH are `#NUM!` once
  e^|x| overflows a double (`SINH(710)`, `COSH(-710)`), although the true sinh(710) is finite, while `SINH(709.78)` is
  a number. So their domain is e^|x| <= DOUBLE_MAX. The cut lies in (709.78, 710) and only its exact position is
  unprobed, so the draws keep out of [709.78, 710].
- numbers.md, "Inverse hyperbolics, COTH/CSCH and the SINH/TANH thresholds" (T-1005): past the overflow `CSCH(710)` is
  0 and `SECH(710)` is 0 (T-2169), not errors; `TANH(x)` is 1 and `COTH(x)` is 1 for large x. Those 0s are underflows
  of a true value below 1e-300, inside Gate A's absolute floor, so SECH and CSCH have no domain cut. Their draws stop
  at |x| = 1000 (the math 1e-434 at most), because a true value like sech(1E+300) has an exponent of 300 digits.
- numbers.md, "COTH, CSCH and SECH are 1 over Excel's own TANH, SINH and COSH" (T-0917): COTH(0) is 1/TANH(0) = 1/0.
  gen-m5 (excel-graph) has `CSCH(0)` = `#DIV/0!` (4a971c8636db937d Calc!A289), so 0 is a pole of COTH and CSCH.
- numbers.md, "SINH / COSH / TANH are exp-based, not libm" and "SINH and TANH below 1 are built from expm1" (T-0914,
  T-2134): Excel's kernels are up to about 2.6 ulp from the true value. That is kernel inaccuracy, not a semantic rule,
  so the definitions stay the true functions; the tail stratum draws below 1, where it shows.
"""

from __future__ import annotations

import math
import random
import sys
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs.annuity import scalars
from rustsheet_tools.accuracy.literal import MAX_LITERAL, Arg
from rustsheet_tools.accuracy.model import Definition, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

OVERFLOW_SOURCE = "numbers.md#sinh-and-cosh-overflow-is-num-at-once-sech-there-is-0-t-2169"
RECIPROCAL_SOURCE = "numbers.md#coth-csch-and-sech-are-1-over-excels-own-tanh-sinh-and-cosh-t-0917"
POLE_SOURCE = "gen-m5 CSCH(0) #DIV/0! 4a971c8636db937d Calc!A289 (excel-graph)"
KERNEL_SOURCE = "numbers.md#sinh--cosh--tanh-are-exp-based-not-libm"
EXP_CUT = math.log(sys.float_info.max)  # 709.782712893384: e^|x| overflows a double above it
UNPROBED = (709.78, 710.0)  # the exact cut lies in here; the draws keep out
RECIPROCAL_LIMIT = 1000.0  # SECH and CSCH draws stop here: the math is about 1e-434 there
OVERFLOW_LIMIT = UNPROBED[0]  # SINH and COSH magnitudes; the edge stratum still draws the #NUM! side


def _overflow(args: Sequence[MathArg]) -> str | None:
    """SINH and COSH: `#NUM!` once e^|x| overflows a double."""
    (x,) = scalars(args)
    return "e^|x| overflows a double" if abs(x) > EXP_CUT else None


def _zero_pole(args: Sequence[MathArg]) -> str | None:
    """COTH and CSCH at 0: Excel's `#DIV/0!`."""
    (x,) = scalars(args)
    return "a pole at 0" if x == 0 else None


def _signed(rng: random.Random, magnitude: float) -> float:
    return -magnitude if rng.random() < 0.5 else magnitude


def _outside_unprobed(x: float) -> float:
    """Move a draw out of the unprobed band of the overflow cut, to its probed lower end."""
    return math.copysign(UNPROBED[0], x) if UNPROBED[0] < abs(x) < UNPROBED[1] else x


def _drawer(limit: float, edge_max: float) -> Callable[[str, random.Random], list[Arg]]:
    """The draw of a function whose magnitudes end at `limit`, with edge points up to `edge_max`."""

    def draw(stratum: str, rng: random.Random) -> list[Arg]:
        if stratum == "tail":  # below 1, where Excel's expm1 forms show; and toward the overflow
            if rng.random() < 0.7:
                x = log_uniform(rng, 1e-12, 1.0)
            else:
                x = rng.uniform(600.0, 709.78) if limit > 700 else rng.uniform(15.0, 25.0)
        elif stratum == "edge":
            x = rng.choice(
                [e for e in (0.0, 1.0, 1e-300, 709.78, 710.0, 1e300, 20.0, 19.0, 0.5) if e <= edge_max]
            )
        elif stratum == "extreme":
            x = log_uniform(rng, 10.0, limit)
        elif stratum == "log-uniform":
            x = log_uniform(rng, 1e-300, limit)
        else:
            x = rng.uniform(0.0, 5.0)
        return [_outside_unprobed(_signed(rng, x))]

    return draw


def _definition(
    function: str,
    kernel: Callable[[mpmath.mpf], mpmath.mpf],
    *,
    limit: float,
    edge_max: float = MAX_LITERAL,
    domain: Callable[[Sequence[MathArg]], str | None] | None = None,
    sources: tuple[str, ...],
) -> Definition:
    def value(args: Sequence[MathArg]) -> mpmath.mpf:
        (x,) = scalars(args)
        return kernel(x)

    return register(
        Definition(
            function=function,
            params=(Param("number", "real", source=sources[0]),),
            value=value,
            domain=domain,
            draw=_drawer(limit, edge_max),
            sources=sources,
        )
    )


SINH = _definition(
    "SINH", mpmath.sinh, limit=OVERFLOW_LIMIT, domain=_overflow, sources=(OVERFLOW_SOURCE, KERNEL_SOURCE)
)
COSH = _definition(
    "COSH", mpmath.cosh, limit=OVERFLOW_LIMIT, domain=_overflow, sources=(OVERFLOW_SOURCE, KERNEL_SOURCE)
)
TANH = _definition("TANH", mpmath.tanh, limit=MAX_LITERAL, sources=(KERNEL_SOURCE,))
COTH = _definition(
    "COTH", mpmath.coth, limit=MAX_LITERAL, domain=_zero_pole, sources=(RECIPROCAL_SOURCE, POLE_SOURCE)
)
SECH = _definition(
    "SECH",
    mpmath.sech,
    limit=RECIPROCAL_LIMIT,
    edge_max=RECIPROCAL_LIMIT,
    sources=(OVERFLOW_SOURCE, RECIPROCAL_SOURCE),
)
CSCH = _definition(
    "CSCH",
    mpmath.csch,
    limit=RECIPROCAL_LIMIT,
    edge_max=RECIPROCAL_LIMIT,
    domain=_zero_pole,
    sources=(RECIPROCAL_SOURCE, POLE_SOURCE, OVERFLOW_SOURCE),
)
