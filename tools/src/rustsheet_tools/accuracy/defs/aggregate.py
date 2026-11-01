"""SUM and AVERAGE (Gate A slice 3f, T-3549, decision 0237).

The math: the exact sum of the doubles of the written literals; AVERAGE divides it by the count. Two Excel rules
that depend only on those doubles are part of the definition (decision 0237):
- The near-zero snap (numbers.md, "Near-zero snapping in addition and subtraction"): every SUM call, and AVERAGE's
  sum before it divides, is exactly 0 when the final addition's two addends (the running total before the last
  argument, and the last argument) have opposite signs and the result is below 8 ULP of the running total,
  2^(e-49) for its binary exponent e, a bound itself flushed to 0 when it is subnormal. There the math is 0.
- Overflow at an intermediate point (numbers.md, "Overflow in `+ - * /` is #NUM! where it happens", aggregate
  overflow, T-0192): a running total beyond the double range is `#NUM!` even when the exact final sum is finite
  (`SUM(9E+307,9E+307,-9E+307)`), so such a point is a domain error.
Both read Excel's running total: each addition rounded to a 64-bit significand and then to a double ("Division
rounds twice", the same for `+`; `fitlab.exact.double_round`), added left to right.

The arity is drawn per stratum (`variadic`): 2 to 10 scalars (typical), 2 to 7 (tail), 1 to 6 or 255 (edge), 50 to
255 (extreme; Excel's argument limit) and log-uniform 1 to 255 (log-uniform). No array literals.
"""

from __future__ import annotations

import math
import random
import sys
from collections.abc import Callable, Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs.annuity import scalars
from rustsheet_tools.accuracy.literal import MAX_LITERAL, Arg, round15
from rustsheet_tools.accuracy.model import Definition, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform
from rustsheet_tools.fitlab.exact import double_round

SNAP_SOURCE = "numbers.md#near-zero-snapping-in-addition-and-subtraction"
OVERFLOW_SOURCE = "numbers.md#overflow-in------is-num-where-it-happens"
MAX_ARGS = 255  # Excel's argument limit
SNAP_ULPS_LOG2 = 49  # 8 ULP of a double of exponent e is 2^(e - 52 + 3)


def running_total(xs: Sequence[float]) -> tuple[float, float]:
    """(the total before the last addend, the total): Excel's left-to-right sum, each addition x87-rounded. A total
    beyond the double range is `math.inf` (with its sign)."""
    before, total = 0.0, 0.0
    for i, x in enumerate(xs):
        before = total
        if i == 0:
            total = x
            continue
        try:
            total = double_round(Fraction(total) + Fraction(x))
        except OverflowError:
            return before, math.copysign(math.inf, total + x)
    return before, total


def snaps(before: float, last: float, total: float) -> bool:
    """Whether a final addition `before + last = total` is snapped to 0 (numbers.md: opposite signs and |total|
    below 8 ULP of `before`, a subnormal bound flushed to 0)."""
    if before == 0 or last == 0 or (before < 0) == (last < 0):
        return False
    exponent = math.frexp(abs(before))[1] - 1
    bound = math.ldexp(1.0, exponent - SNAP_ULPS_LOG2)
    if bound < sys.float_info.min:  # 2^-1022: a subnormal bound is flushed (T-2714)
        return False
    return abs(total) < bound


def sum_value(xs: Sequence[mpmath.mpf]) -> mpmath.mpf:
    """The exact sum of `xs`, or 0 where Excel's final addition is snapped."""
    doubles = [float(x) for x in xs]
    if len(doubles) >= 2:
        before, total = running_total(doubles)
        if snaps(before, doubles[-1], total):  # the final addition: `before + doubles[-1]`
            return mpmath.mpf(0)
    return mpmath.fsum(xs)


def overflow(args: Sequence[MathArg]) -> str | None:
    """Why the point is a domain error: a running total beyond the double range."""
    _, total = running_total([float(x) for x in scalars(args)])
    return None if math.isfinite(total) else "a running total is beyond the double range"


def _sum(args: Sequence[MathArg]) -> mpmath.mpf:
    return sum_value(scalars(args))


def _average(args: Sequence[MathArg]) -> mpmath.mpf:
    xs = scalars(args)
    return sum_value(xs) / len(xs)


# -- draws -------------------------------------------------------------------------------------


def _signed(rng: random.Random, lo: float, hi: float, negative: float = 0.3) -> float:
    x = log_uniform(rng, lo, hi)
    return -x if rng.random() < negative else x


def _decimal(rng: random.Random) -> float:
    """A short decimal (0.1, 2.75, 0.0003): the literals whose doubles leave a residue when they cancel. The drawn
    double need not be the decimal's nearest double: the sampler writes `round15` of it (`Sampler.finish`), and the
    math reads the written literal."""
    return rng.randint(1, 9999) / 10.0 ** rng.randint(0, 5)


def _cancelling(rng: random.Random) -> list[float]:
    """A near-cancelling sum: short decimals, then a last addend that cancels them to within a few units of its
    15th digit, around the snap's 8-ULP bound (as probes-numeric `snap-addsub/*` does with k·2^-52); some points end
    with a 0, which is never snapped (numbers.md, "M2 pickers ... trailing numeric 0")."""
    head = [round15(_decimal(rng) * (-1 if rng.random() < 0.3 else 1)) for _ in range(rng.randint(1, 5))]
    _, total = running_total(head)  # the head as written, so `last` cancels the literals' doubles
    last = -total
    shift = rng.randint(-3, 3)
    if shift and last != 0:
        last += shift * 10 ** (math.floor(math.log10(abs(last))) - 14)
    values = [*head, last]
    if rng.random() < 0.2:
        values.append(0.0)
    return values


def _edge(rng: random.Random) -> list[float]:
    shape = rng.randrange(6)
    big = rng.uniform(5e307, MAX_LITERAL)
    if shape == 0:  # a running total overflows although the final sum is finite: #NUM! (T-0192)
        return [big, big, -big]
    if shape == 1:  # the same addends in an order that never overflows
        return [big, -big, big]
    if shape == 2:  # one argument: no addition, no snap
        return [_signed(rng, 1e-3, 1e6)]
    if shape == 3:  # zeros among the addends
        return [0.0 if rng.random() < 0.5 else _signed(rng, 1e-3, 1e6) for _ in range(rng.randint(2, 6))]
    if shape == 4:  # an exact cancellation
        x = _signed(rng, 1e-3, 1e6)
        return [x, -x]
    # shape 5, the argument limit: 255 mixed-sign addends
    return [_signed(rng, 1e-3, 1e6) for _ in range(MAX_ARGS)]


def _draw(stratum: str, rng: random.Random) -> list[Arg]:
    if stratum == "tail":
        return list(_cancelling(rng))
    if stratum == "edge":
        return list(_edge(rng))
    if stratum == "extreme":  # long sums of mixed magnitudes and signs: left-to-right order shows
        return [_signed(rng, 1e-20, 1e20, 0.5) for _ in range(rng.randint(50, MAX_ARGS))]
    if stratum == "log-uniform":
        return [_signed(rng, 1e-300, 1e300, 0.5) for _ in range(round(log_uniform(rng, 1, MAX_ARGS)))]
    return [_signed(rng, 1e-3, 1e6) for _ in range(rng.randint(2, 10))]


def _definition(function: str, value: Callable[[Sequence[MathArg]], mpmath.mpf]) -> Definition:
    return register(
        Definition(
            function=function,
            params=(Param("number", "real", source=SNAP_SOURCE),),
            value=value,
            domain=overflow,
            draw=_draw,
            variadic=True,
            sources=(SNAP_SOURCE, OVERFLOW_SOURCE, "decision 0237"),
        )
    )


SUM = _definition("SUM", _sum)
AVERAGE = _definition("AVERAGE", _average)
