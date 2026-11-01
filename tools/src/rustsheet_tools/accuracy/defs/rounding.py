"""ROUND, ROUNDUP and ROUNDDOWN (Gate A slice 3e, T-3555).

The math (decision 0223): the 15-digit decimal of the argument, rounded at `num_digits`: half away
from zero for ROUND, away from zero for ROUNDUP, toward zero for ROUNDDOWN. That is Excel's own
two-stage rule (numbers.md, "ROUND family rounds in two decimal stages", VERIFIED), and it is not
SPEC-0029's "the math at the double nearest the written decimal": the exact double of 2.675 is
2.67499999999999982236431605997495353221893310546875, which rounds to 2.67, while Excel's
`ROUND(2.675,2)` is 2.68. Every table literal has at most 15 significant digits, so the 15-digit cut
of its double is the written decimal again (`literal.decimal15`). The stage-1 tie Excel breaks
inconsistently, an exact half at the 16th digit, needs a 16th digit, which no literal has: it is an
eval case (an argument computed by a formula), not a table point.

Points the draws meet, all inside the envelope of a correct engine:
- ROUNDUP adds the unit in binary (numbers.md, "ROUNDUP adds the unit in binary"):
  `ROUNDUP(123.456,2)` is 123.46000000000001, 1 ulp from the decimal math, far inside ε.
- A `num_digits` past the digits the value has gives the value back (`ROUND(1.5,400)` = 1.5); one
  past its leading digit gives 0 for ROUND and ROUNDDOWN (`ROUND(1.5,-400)` = 0) and the unit
  10^-num_digits for ROUNDUP, which is a math domain error once it overflows the double range
  (`ROUNDUP(1.5,-400)` is `#NUM!`; numbers.md, "ROUND family: huge num_digits wrap").
- `num_digits` is drawn within ±400, the widest `num_digits` numbers.md's probes evidence as plain
  (`ROUND(1.5,±400)`, `ROUNDUP(1.5,-400)`), and wide enough for every digit of every double. Past
  it Excel narrows a huge `num_digits` (numbers.md, "ROUND family: huge num_digits wrap"), which is
  argument semantics the eval suites own, as SPEC-0029 leaves integer truncation to them; and this
  slice's first draw found `ROUNDUP(6.4421516092447E+265,-43376)` = 6.4421516092447E+265 where the
  wrap rule says `#NUM!` (numbers.md, "ROUNDUP of a huge value at a huge negative num_digits",
  T-3627).
- Excel flushes any decimal it converts to a double to 0 when that double would be subnormal: the
  result, and ROUNDUP's rounded-down part and unit (numbers.md, "ROUNDUP and ROUNDDOWN of a number
  whose 15-digit cut is subnormal"). So `ROUND(-2.22507385850721E-308,320)` is 0. And
  `ROUNDUP(8.90029543402884E-308,309)` is 8.9E-308, a normal value smaller than its argument,
  because the unit 1E-309 is flushed. The math keeps the decimal there, so Excel is more than G
  off it. Every such point is within Gate A's absolute floor 1e-300 of the math, and the engine
  matches Excel (decision 0223 lists the points).
"""

from __future__ import annotations

import math
import random
import sys
from collections.abc import Callable, Sequence
from decimal import ROUND_DOWN, ROUND_HALF_UP, ROUND_UP, Context, Decimal

import mpmath

from rustsheet_tools.accuracy.literal import MAX_LITERAL, MIN_LITERAL, Arg, decimal15
from rustsheet_tools.accuracy.model import (
    Definition,
    DomainError,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
)
from rustsheet_tools.accuracy.sampler import log_uniform

TWO_STAGE_SOURCE = "numbers.md#round-family-rounds-in-two-decimal-stages"
WRAP_SOURCE = "numbers.md#round-family-huge-num_digits-wrap"
DECISION = "decision 0223"
MAX_DIGITS = 400  # |num_digits| drawn up to here (see the module docstring)
DOUBLE_MAX = Decimal(sys.float_info.max)
# Wide enough for any quantize below: a 15-digit coefficient and exponents down to -400 - 323.
_CONTEXT = Context(prec=40, Emax=10**6, Emin=-(10**6))
_MODES = {"ROUND": ROUND_HALF_UP, "ROUNDUP": ROUND_UP, "ROUNDDOWN": ROUND_DOWN}


def sixteenth_digit_tie(number: float) -> bool:
    """Whether the exact double `number` is a half at its 16th significant digit, the stage-1 tie
    Excel breaks toward zero for ROUND and away from zero for ROUNDUP and ROUNDDOWN
    (`ROUND(999999999999957.5,0)`; TWO_STAGE_SOURCE). No 15-digit literal is one."""
    if number == 0:
        return False
    exact = Decimal(number)
    sixteen = exact.quantize(Decimal(1).scaleb(exact.adjusted() - 15), context=_CONTEXT)
    return sixteen == exact and sixteen.as_tuple().digits[-1] == 5


def decimal_round(function: str, number: float, num_digits: int) -> Decimal:
    """`number`'s 15-digit decimal rounded at `num_digits` by `function`'s rule (ROUND_HALF_UP is
    half away from zero). DomainError when the result is beyond the double range; NoValueError
    outside what the rule covers: |`num_digits`| above `MAX_DIGITS` (Excel narrows it first) and a
    16th-digit tie. Neither can be drawn; an eval cell (`rustsheet-accuracy math`) gets no value."""
    if not math.isfinite(number):
        msg = "a non-finite number"
        raise NoValueError(msg)
    if abs(num_digits) > MAX_DIGITS:
        msg = f"num_digits beyond ±{MAX_DIGITS}: Excel narrows it first ({WRAP_SOURCE})"
        raise NoValueError(msg)
    if sixteenth_digit_tie(number):
        msg = f"a tie at the 16th digit: Excel's stage-1 tie rule ({TWO_STAGE_SOURCE})"
        raise NoValueError(msg)
    cut = decimal15(number)
    unit = Decimal(1).scaleb(-num_digits, _CONTEXT)
    exponent = cut.as_tuple().exponent  # an int: `cut` is finite
    if isinstance(exponent, int) and exponent >= -num_digits:  # nothing is dropped
        result = cut
    else:
        result = cut.quantize(unit, rounding=_MODES[function], context=_CONTEXT)
    if abs(result) > DOUBLE_MAX:
        msg = f"the rounded value {result:.3E} overflows the double range"
        raise DomainError(msg)
    return result


def _value(function: str) -> Callable[[Sequence[MathArg]], mpmath.mpf]:
    def value(args: Sequence[MathArg]) -> mpmath.mpf:
        number, digits = args
        assert not isinstance(number, bool | tuple) and not isinstance(digits, bool | tuple)
        return mpmath.mpf(str(decimal_round(function, float(number), int(digits))))

    return value


def _literal(rng: random.Random, digits: int, exponent: int, sign: float) -> float:
    """A number with `digits` significant digits (the last one nonzero) and decimal exponent `exponent`."""
    mantissa = rng.randrange(10 ** (digits - 1), 10**digits)
    if mantissa % 10 == 0:
        mantissa += rng.randrange(1, 10)
    return sign * float(Decimal(mantissa).scaleb(exponent - digits + 1))


def _exponent(x: float) -> int:
    return decimal15(x).adjusted()


def _sign(rng: random.Random) -> float:
    return 1.0 if rng.random() < 0.5 else -1.0


def _dropping_digits(rng: random.Random, x: float) -> int:
    """A `num_digits` from one past the leading digit (all dropped) to the 15th digit (none)."""
    e = _exponent(x) if x != 0 else 0
    return rng.randint(-e - 2, 14 - e)


def _tie(rng: random.Random) -> list[Arg]:
    """A decimal half at the rounding digit (2.675 at 2, -2.5 at 0), or a 15-digit near-half
    just below or above it (…49999… / …50000…1)."""
    sign, e = _sign(rng), rng.randint(-8, 12)
    if rng.random() < 0.5:
        kept = rng.randint(1, 14)  # digits kept before the dropped 5
        mantissa = rng.randrange(10 ** (kept - 1), 10**kept) * 10 + 5
        x = sign * float(Decimal(mantissa).scaleb(e - kept))
        return [x, kept - 1 - e]
    kept = rng.randint(1, 10)
    tail = 15 - kept - 1  # the digits after the 4 or the 5
    body = rng.randrange(10 ** (kept - 1), 10**kept)
    near = body * 10 ** (15 - kept) + (5 * 10**tail - 1 if rng.random() < 0.5 else 5 * 10**tail + 1)
    x = sign * float(Decimal(near).scaleb(e - 14))
    return [x, kept - 1 - e]


def _edge(rng: random.Random) -> list[Arg]:
    """Zero, the leading digit exactly, nothing dropped, the literal range's ends and the
    `num_digits` bounds."""
    shape = rng.randrange(6)
    x = _literal(rng, rng.randint(1, 15), rng.randint(-6, 12), _sign(rng))
    e = _exponent(x)
    if shape == 0:
        return [0.0, rng.randint(-6, 10)]
    if shape == 1:  # rounding at the leading digit, or one above it
        return [x, -e - rng.randint(0, 1)]
    if shape == 2:  # nothing dropped, up to the num_digits bound
        return [x, rng.choice((15 - e, 330, MAX_DIGITS))]
    if shape == 3:  # everything dropped: 0, or ROUNDUP's unit, which overflows below -308
        return [x, -rng.choice((e + 2, 330, MAX_DIGITS))]
    if shape == 4:  # the largest literals, rounded near the double range's top
        top = _literal(rng, rng.randint(1, 15), 307, _sign(rng))
        return [
            min(top, MAX_LITERAL) if top > 0 else max(top, -MAX_LITERAL),
            rng.choice((-306, -307, -308, -309)),
        ]
    low = _sign(rng) * MIN_LITERAL * rng.choice((1.0, 1.5, 4.0))
    return [low, rng.choice((300, 307, 308, 309, 320))]


def _draw(stratum: str, rng: random.Random) -> list[Arg]:
    """[number, num_digits] by stratum: typical numbers 1E-4 to 1E9 with 1 to 15 digits, tails at
    and beside a decimal half, the edges of `_edge`, extreme magnitudes 1E-300 to 1E307, and the
    whole domain (any literal, any `num_digits` within ±400) log-uniform."""
    if stratum == "tail":
        return _tie(rng)
    if stratum == "edge":
        return _edge(rng)
    sign = _sign(rng)
    if stratum == "log-uniform":
        x = sign * log_uniform(rng, MIN_LITERAL, MAX_LITERAL)
        return [x, int(_sign(rng) * round(log_uniform(rng, 1, MAX_DIGITS)))]
    span = (-300, 306) if stratum == "extreme" else (-4, 8)
    x = _literal(rng, rng.randint(1, 15), rng.randint(*span), sign)
    return [x, _dropping_digits(rng, x)]


def _definition(function: str) -> Definition:
    return register(
        Definition(
            function=function,
            params=(
                Param("number", "real", Interval(), source=TWO_STAGE_SOURCE),
                # Every integer is valid (Excel narrows a huge one); the draws stay within ±MAX_DIGITS.
                Param("num_digits", "int", Interval(), source=WRAP_SOURCE),
            ),
            value=_value(function),
            draw=_draw,
            sources=(DECISION, TWO_STAGE_SOURCE, WRAP_SOURCE, "numbers.md#roundup-adds-the-unit-in-binary"),
        )
    )


ROUND = _definition("ROUND")
ROUNDUP = _definition("ROUNDUP")
ROUNDDOWN = _definition("ROUNDDOWN")
