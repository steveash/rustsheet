"""INT, EVEN, ODD and MROUND (Gate A slice 4g, T-3674, decision 0246).

Each reads one rule from numbers.md, VERIFIED on excel-graph cells:
- INT (numbers.md, "INT cuts to 15 significant digits first"): a value below 1E+15 in magnitude is cut
  to 15 significant digits, a tie at the 16th digit away from zero, and then floored, so
  `INT(1-2^-53)` is 1 and `INT(-100000000000000.25)` is -100000000000000. From 1E+15 up the double is
  floored as it is (it is already a whole number). For a table literal the cut is the written
  decimal, so the math is the floor of the written decimal.
- EVEN (numbers.md, "ODD above 2^53 adds one in binary", last lines): `ceil(|x|/2)*2` with the sign of
  `x`, exact, at the double.
- ODD (same entry): `c = ceil(|x|)`, plus 1 when `c` is even, with the sign of `x`. Excel adds the 1 in
  binary, so from 2^53 up it is a tie or a no-op (`ODD(9007199254740994)` is 9007199254740996); the
  math adds it exactly, one unit in 2^53 or less from Excel, far inside ε.
- MROUND (numbers.md, "MROUND rounds the quotient up within 5E-15 of a half, and multiplies in
  binary"): the step count is Excel's rule as the entry states it, on the double quotient
  `q = |number/multiple|`: `floor(q)`, plus 1 when `q - floor(q)` is at least `0.5 - 5E-15`. The
  tolerance is defined on that double, so the math takes the step from it too (decision 0246): an
  exact-ratio reading would put a decimal half such as 64.5 a whole step from Excel wherever the
  double quotient falls an ulp below it, a false decision 0188 R3 mark, as decision 0244 says of the
  CEILING family's cut. The result is the step count times |multiple| exactly (Excel multiplies in
  binary, within an ulp), with the sign of `number`. A zero multiple gives 0 (MROUND/s1
  `MROUND(1.1E-05,0)`, `MROUND(-0.005901617,0)`), and so does a zero number with a negative multiple
  (`MROUND(0,-0.0042017)`); a nonzero number and a multiple of opposite signs is `#NUM!` (9 MROUND/s1
  cells, `MROUND(17.9707,-0.0126)` among them; excel-graph 2026-11-02, numbers.md's MROUND entry).
  A double quotient that overflows has no math value: no Excel cell evidences what MROUND does there.
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence
from decimal import Decimal
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy import model
from rustsheet_tools.accuracy.defs.ceiling_floor import cut15
from rustsheet_tools.accuracy.literal import MAX_LITERAL, MIN_LITERAL, Arg, exponent10
from rustsheet_tools.accuracy.model import Definition, Interval, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import decimal_literal, log_uniform, random_sign

INT_SOURCE = "numbers.md#int-cuts-to-15-significant-digits-first-t-2171"
ODD_SOURCE = "numbers.md#odd-above-253-adds-one-in-binary-t-2173"
MROUND_SOURCE = (
    "numbers.md#mround-rounds-the-quotient-up-within-5e-15-of-a-half-and-multiplies-in-binary-t-2201"
)
DECISION = "decision 0246"
INT_CUT_BELOW = 1e15  # INT cuts below this magnitude and floors the double from here up
# Excel's cutoff is the double nearest 0.5 - 5E-15, compared with the double fraction of `q`.
MROUND_CUTOFF = Fraction(0.5 - 5e-15)


def int_value(x: float) -> int:
    """INT: the floor of the 15-digit cut (ties away from zero) below 1E+15, of the double above."""
    if abs(x) >= INT_CUT_BELOW:
        return math.floor(x)
    return math.floor(cut15(Fraction(x)))


def even_value(x: float) -> int:
    """EVEN: `ceil(|x|/2)*2` with the sign of `x`."""
    steps = 2 * math.ceil(abs(Fraction(x)) / 2)
    return steps if x >= 0 else -steps


def odd_value(x: float) -> int:
    """ODD: `ceil(|x|)`, the next odd number when it is even, with the sign of `x` (ODD(0) is 1)."""
    c = math.ceil(abs(Fraction(x)))
    if c % 2 == 0:
        c += 1
    return c if x >= 0 else -c


def mround_value(number: float, multiple: float) -> Fraction:
    """MROUND's math (see the module docstring). DomainError for opposite signs (`#NUM!`);
    NoValueError when the double quotient overflows."""
    if multiple == 0:
        return Fraction(0)
    if (number > 0 > multiple) or (number < 0 < multiple):
        msg = "MROUND of a number and a multiple of opposite signs (#NUM!)"
        raise model.DomainError(msg)
    quotient = abs(number / multiple)  # the double quotient; inf when it overflows
    if not math.isfinite(quotient):
        msg = "the double quotient number/multiple overflows: no Excel evidence"
        raise model.NoValueError(msg)
    q = Fraction(quotient)
    steps = math.floor(q)
    if q - steps >= MROUND_CUTOFF:
        steps += 1
    result = steps * abs(Fraction(multiple))
    return result if number >= 0 else -result


def _mpf(r: Fraction | int) -> mpmath.mpf:
    r = Fraction(r)
    return mpmath.mpf(r.numerator) / r.denominator


def _unary(rule: Callable[[float], int]) -> Callable[[Sequence[MathArg]], mpmath.mpf]:
    def value(args: Sequence[MathArg]) -> mpmath.mpf:
        (x,) = args
        assert not isinstance(x, bool | tuple)
        return _mpf(rule(float(x)))

    return value


def _mround(args: Sequence[MathArg]) -> mpmath.mpf:
    number, multiple = args
    assert not isinstance(number, bool | tuple) and not isinstance(multiple, bool | tuple)
    return _mpf(mround_value(float(number), float(multiple)))


def _near_whole(rng: random.Random) -> float:
    """A whole number (even or odd), a half, or one unit of the 15th digit beside a whole number
    (2.99999999999999, 3.00000000000001)."""
    whole = rng.randrange(0, 10 ** rng.randint(1, 13))
    shape = rng.randrange(3)
    if shape == 0 or whole == 0:
        x = Decimal(whole) + (Decimal("0.5") if shape == 0 and rng.random() < 0.5 else 0)
    else:
        digits = len(str(whole))
        unit = Decimal(1).scaleb(digits - 15)
        x = Decimal(whole) + unit * rng.choice((-1, 1))
    return random_sign(rng) * float(x)


def _whole_edge(rng: random.Random) -> float:
    """Zero, the smallest units, the literal range's ends and the whole numbers about 2^53 and 2^54
    as 15-digit literals."""
    return random_sign(rng) * rng.choice(
        (
            0.0,
            0.5,
            1.0,
            2.0,
            3.0,
            MIN_LITERAL,
            1e-300,
            0.999999999999999,
            1.00000000000001,
            9.00719925474099e15,
            9.00719925474100e15,
            1.80143985094820e16,
            MAX_LITERAL,
        )
    )


def _whole_draw(stratum: str, rng: random.Random) -> list[Arg]:
    """[number] by stratum: typical 1E-4 to 1E9 with 1 to 15 digits, tails at and beside a whole
    number or a half, the edges of `_whole_edge`, extreme magnitudes 1E-300 to 1E307, and the whole
    literal range log-uniform."""
    sign = random_sign(rng)
    if stratum == "tail":
        return [_near_whole(rng)]
    if stratum == "edge":
        return [_whole_edge(rng)]
    if stratum == "log-uniform":
        return [sign * log_uniform(rng, MIN_LITERAL, MAX_LITERAL)]
    span = (-300, 306) if stratum == "extreme" else (-4, 8)
    return [decimal_literal(rng, rng.randint(1, 15), rng.randint(*span), sign)]


def _mround_signs(rng: random.Random) -> tuple[float, float]:
    """Signs of (number, multiple): the same sign, or opposite signs (`#NUM!`) one time in ten."""
    xs = random_sign(rng)
    return (xs, -xs) if rng.random() < 0.1 else (xs, xs)


def _mround_near_half(rng: random.Random) -> list[Arg]:
    """A quotient at a half (0.3 over 0.2), at a whole number of steps, or a number one to four
    units of the 15th digit below a half of a multiple of 1 or 2 (0.499999999999997 over 1, inside
    Excel's 5E-15 tolerance, and 0.49999999999998 outside it). The cutoff itself is not drawn: its
    side depends on the last bit of the double quotient."""
    xs, ms = _mround_signs(rng)
    shape = rng.randrange(3)
    if shape == 2:
        unit = rng.choice((1, 2))
        below = rng.choice((1, 2, 3, 4, 6, 7, 8, 9, 10, 20, 100))
        x = (Decimal("0.5") - Decimal(below).scaleb(-15)) * unit
        return [xs * float(x), ms * float(unit)]
    m_digits = rng.randint(1, 6)
    multiple = Decimal(repr(decimal_literal(rng, m_digits, rng.randint(-6, 6), 1.0)))
    steps = Decimal(rng.randrange(0, 10 ** rng.randint(1, 15 - m_digits)))
    if shape == 0:
        steps += Decimal("0.5")
    x = steps * multiple
    if len(x.normalize().as_tuple().digits) > 15:
        x = Decimal(repr(float(x)))  # the double nearest: an exact draw would need a 16th digit
    return [xs * float(x), ms * float(multiple)]


def _mround_edge(rng: random.Random) -> list[Arg]:
    """A zero number or multiple, a multiple at or above the number, and the literal range's ends."""
    xs, ms = _mround_signs(rng)
    x = decimal_literal(rng, rng.randint(1, 15), rng.randint(-6, 12), xs)
    shape = rng.randrange(5)
    if shape == 0:
        return [0.0, decimal_literal(rng, rng.randint(1, 6), rng.randint(-6, 6), ms)]
    if shape == 1:
        return [x, 0.0]
    if shape == 2:  # at most two steps: |multiple| >= |number| / 2
        return [x, ms * abs(x) * rng.choice((0.5, 1.0, 1.5, 2.0, 2.5, 1e6))]
    if shape == 3:  # the smallest numbers over multiples near them
        return [
            xs * rng.choice((MIN_LITERAL, 1e-307, 1e-300)),
            ms * rng.choice((MIN_LITERAL, 1e-307, 2e-300, 1.0)),
        ]
    return [xs * MAX_LITERAL * rng.choice((1.0, 0.5, 0.1)), ms * rng.choice((1.0, 3.0, 0.7, 1e7, 1e300))]


def _mround_draw(stratum: str, rng: random.Random) -> list[Arg]:
    """[number, multiple] by stratum: typical quotients 1 to 1E6, tails at and beside a half, the
    edges of `_mround_edge`, extreme magnitudes 1E-300 to 1E307 with quotients up to 1E300, and both
    arguments log-uniform over the literal range (a pair whose quotient overflows is redrawn)."""
    if stratum == "tail":
        return _mround_near_half(rng)
    if stratum == "edge":
        return _mround_edge(rng)
    xs, ms = _mround_signs(rng)
    if stratum == "log-uniform":
        return [
            xs * log_uniform(rng, MIN_LITERAL, MAX_LITERAL),
            ms * log_uniform(rng, MIN_LITERAL, MAX_LITERAL),
        ]
    if stratum == "extreme":
        x = decimal_literal(rng, rng.randint(1, 15), rng.randint(-300, 306), xs)
        m_exp = max(-307, min(306, exponent10(x) - rng.randint(-5, 300)))
        return [x, decimal_literal(rng, rng.randint(1, 15), m_exp, ms)]
    x = decimal_literal(rng, rng.randint(1, 15), rng.randint(-4, 8), xs)
    return [x, decimal_literal(rng, rng.randint(1, 6), exponent10(x) - rng.randint(0, 6), ms)]


def _unary_definition(function: str, rule: Callable[[float], int], source: str) -> Definition:
    return register(
        Definition(
            function=function,
            params=(Param("number", "real", Interval(), source=source),),
            value=_unary(rule),
            draw=_whole_draw,
            sources=(DECISION, source),
        )
    )


INT = _unary_definition("INT", int_value, INT_SOURCE)
EVEN = _unary_definition("EVEN", even_value, ODD_SOURCE)
ODD = _unary_definition("ODD", odd_value, ODD_SOURCE)
MROUND = register(
    Definition(
        function="MROUND",
        params=(
            Param("number", "real", Interval(), source=MROUND_SOURCE),
            Param("multiple", "real", Interval(), source=MROUND_SOURCE),
        ),
        value=_mround,
        draw=_mround_draw,
        sources=(DECISION, MROUND_SOURCE, "arrays.md#ceilingfloor-family-and-mround-lift-over-arrays"),
    )
)
