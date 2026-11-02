"""CEILING, CEILING.MATH, CEILING.PRECISE, FLOOR, FLOOR.MATH and FLOOR.PRECISE (Gate A slice 4f,
T-3673).

The math (decision 0244) is Excel's decimal rule read at the written decimals, as decision 0223 reads
the ROUND family: the exact ratio of the two 15-digit decimals `number / significance`, cut to 15
significant digits with a tie at the 16th digit going away from zero, then taken to the whole step
above (CEILING) or below (FLOOR) and multiplied by the significance exactly. The cut is Excel's
design, not an error (numbers.md, "FLOOR.PRECISE: a 16th-digit tie in the ratio cut goes away from
zero", VERIFIED for all seven functions of the family): without it the math of
`CEILING(100,1.01010101010101)` would be 101.010101010101, while the ratio 99.0000000000000099 cuts to
99 and Excel's rule gives 99.99999999999999. A plain ceiling would mark such points as Excel being a
step (here 1%) off the math, a false decision 0188 R3 mark.

The sign and argument rules, all cited from numbers.md and gen/probe cells:
- CEILING and FLOOR take the significance as given: a positive number with a negative significance
  is `#NUM!` (numbers.md, same entry, last sentence), and a negative significance with a negative
  number steps away from zero (`CEILING(-2.5,-1)` = -3, `FLOOR(-2.5,-1)` = -2: ceil and floor of the
  positive ratio, times the negative significance).
- A zero significance gives 0, except in FLOOR, where a nonzero number gives `#DIV/0!` (the 3 FLOOR/s1
  points of T-3673's tables). FLOOR(0,0) has no math value (`NoValueError`): no Excel cell evidences
  it (the engine gives 0), and the draws never pair a zero number with a zero significance.
- The .MATH and .PRECISE forms take |significance|; the .MATH forms' `mode`, when nonzero, turns a
  negative number's step toward zero into one away from zero (CEILING.MATH(-2.5,1,1) = -3). Modes
  are drawn as whole numbers: how Excel reads a fractional mode is not evidenced here.
- A subnormal ratio is 0, so the result is 0 (numbers.md, "CEILING and FLOOR of a subnormal ratio
  are 0", probes/t2715-ceiling-floor-subnormal-ratio): Excel never holds the ratio as a subnormal,
  which a literal cannot be either. The math reads the exact ratio against 2^-1022.
- A ratio of 48 times the double range or more is a math domain error (Excel's `#NUM!`); one between
  0.5563 and 48 times it has no math value, since the evidence does not pin where the boundary lies
  (see `ratio_overflows` and decision 0244).
"""

from __future__ import annotations

import random
import sys
from collections.abc import Callable, Sequence
from decimal import Context, Decimal
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy import model
from rustsheet_tools.accuracy.literal import MAX_LITERAL, MIN_LITERAL, Arg, decimal15, exponent10
from rustsheet_tools.accuracy.model import Definition, Interval, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import decimal_literal, log_uniform, random_sign

TIE_SOURCE = "numbers.md#floorprecise-a-16th-digit-tie-in-the-ratio-cut-goes-away-from-zero-t-2652"
SUBNORMAL_SOURCE = "numbers.md#ceiling-and-floor-of-a-subnormal-ratio-are-0-t-2715"
DECISION = "decision 0244"
ALIAS_DECISION = "decision 0246"
# ECMA.CEILING is two-argument CEILING (numbers.md, "ECMA.CEILING", probes/ecma-ceiling); ISO.CEILING
# is CEILING.PRECISE (every family entry above names it beside CEILING.PRECISE: ties, subnormal ratios).
ALIAS_SOURCES = {"ECMA.CEILING": "numbers.md#ecmaceiling", "ISO.CEILING": TIE_SOURCE}
SMALLEST_NORMAL = Fraction(2) ** -1022
DOUBLE_MAX = Fraction(sys.float_info.max)
# Exact for a whole number of steps (at most 16 digits) times a 15-digit significance.
_CONTEXT = Context(prec=60, Emax=10**6, Emin=-(10**6))

CEILINGS = ("CEILING", "CEILING.MATH", "CEILING.PRECISE")
WITH_MODE = ("CEILING.MATH", "FLOOR.MATH")
SIGNED = ("CEILING", "FLOOR")  # the forms that take the significance's sign


def cut15(ratio: Fraction) -> Fraction:
    """`ratio` cut to 15 significant digits, an exact tie at the 16th digit away from zero."""
    if ratio == 0:
        return ratio
    magnitude = abs(ratio)
    exponent = len(str(magnitude.numerator)) - len(str(magnitude.denominator))
    # 10^exponent <= magnitude < 10^(exponent + 1), correcting the digit-count estimate.
    while Fraction(10) ** exponent > magnitude:
        exponent -= 1
    while Fraction(10) ** (exponent + 1) <= magnitude:
        exponent += 1
    scale = Fraction(10) ** (14 - exponent)
    scaled = magnitude * scale
    whole = scaled.numerator // scaled.denominator
    if scaled - whole >= Fraction(1, 2):
        whole += 1
    cut = whole / scale
    return cut if ratio > 0 else -cut


# The evidenced bracket of the overflow boundary (numbers.md, "CEILING and FLOOR are #NUM! when
# number/significance overflows"): a ratio of 0.5563 x DBL_MAX is a number (CEILING/s1 A75, FLOOR/s1
# A72), one of 48.98 x DBL_MAX is #NUM! (FLOOR.PRECISE/s1 A119). Between them nothing is evidenced.
LARGEST_NUMBER_RATIO = DOUBLE_MAX * Fraction(5563, 10000)
# The exact A119 ratio, so the domain error covers only ratios at or past an evidenced #NUM!.
SMALLEST_OVERFLOW_RATIO = Fraction("3.85290564465378E+92") / Fraction("4.37553180951326E-218")


def ratio_overflows(ratio: Fraction) -> bool:
    """Whether the quotient is beyond the double range where Excel is evidenced `#NUM!`; NoValueError
    inside the unpinned band between the two evidenced ratios."""
    if abs(ratio) >= SMALLEST_OVERFLOW_RATIO:
        return True
    if abs(ratio) > LARGEST_NUMBER_RATIO:
        msg = "a ratio in the unpinned band of the overflow boundary (0.5563 to 48 x DBL_MAX)"
        raise model.NoValueError(msg)
    return False


def snapped(function: str, number: float, significance: float, mode: int = 0) -> Decimal:
    """The math of `function` at the written decimals of `number` and `significance` (and the
    .MATH forms' `mode`). DomainError for Excel's argument errors and an overflowing ratio."""
    x = Fraction(decimal15(number))
    s = Fraction(decimal15(significance))
    if function in SIGNED:
        if s == 0:
            if function == "FLOOR":
                if x == 0:
                    msg = "FLOOR(0,0): no Excel evidence (the engine gives 0)"
                    raise model.NoValueError(msg)
                msg = "FLOOR of a nonzero number by a zero significance (#DIV/0!)"
                raise model.DomainError(msg)
            return Decimal(0)
        if s < 0 < x:
            msg = f"{function} of a positive number by a negative significance (#NUM!)"
            raise model.DomainError(msg)
    else:
        s = abs(s)
        if s == 0:
            return Decimal(0)
    ratio = x / s
    if ratio_overflows(ratio):
        msg = "the ratio number/significance overflows the double range"
        raise model.DomainError(msg)
    if abs(ratio) < SMALLEST_NORMAL:
        return Decimal(0)
    cut = cut15(ratio)
    up = function in CEILINGS
    if function in WITH_MODE and x < 0 and mode != 0:
        up = not up  # away from zero for a negative number
    steps = -((-cut.numerator) // cut.denominator) if up else cut.numerator // cut.denominator
    result = Fraction(steps) * s
    if abs(result) > DOUBLE_MAX:
        msg = "the result overflows the double range: no Excel evidence (no literal draw reaches it)"
        raise model.NoValueError(msg)
    return _CONTEXT.multiply(Decimal(steps), Decimal(s.numerator) / Decimal(s.denominator))


def _value(function: str) -> Callable[[Sequence[MathArg]], mpmath.mpf]:
    def value(args: Sequence[MathArg]) -> mpmath.mpf:
        number, significance, *rest = args
        assert not isinstance(number, bool | tuple) and not isinstance(significance, bool | tuple)
        mode = 0
        if rest:
            assert not isinstance(rest[0], bool | tuple)
            mode = int(rest[0])
        return mpmath.mpf(str(snapped(function, float(number), float(significance), mode)))

    return value


def _signs(function: str, rng: random.Random) -> tuple[float, float]:
    """Signs of (number, significance). The signed forms rarely draw a positive number over a
    negative significance (their `#NUM!`); the others draw both signs freely."""
    xs, ss = random_sign(rng), random_sign(rng)
    if function in SIGNED and xs > 0 and ss < 0 and rng.random() < 0.8:
        ss = 1.0
    return xs, ss


def _mode(function: str, rng: random.Random) -> list[Arg]:
    return [rng.choice((0, 0, 1, 1, -1, 2))] if function in WITH_MODE else []


def _typical(function: str, rng: random.Random) -> list[Arg]:
    """A number 1E-4 to 1E9 with 1 to 15 digits over a significance of 1 to 6 digits, 1 to 1E6
    times smaller."""
    xs, ss = _signs(function, rng)
    x = decimal_literal(rng, rng.randint(1, 15), rng.randint(-4, 8), xs)
    s = decimal_literal(rng, rng.randint(1, 6), exponent10(x) - rng.randint(0, 6), ss)
    return [x, s]


def _near_step(function: str, rng: random.Random) -> list[Arg]:
    """A ratio at or beside a whole number: an exact multiple (0.3 over 0.1), a multiple off by one
    unit of the number's 15th digit, a ratio whose distance to the step the 15-digit cut swallows
    (100 over 1.01010101010101), and an exact tie at the ratio's 16th digit (333333333333333 over
    0.2)."""
    xs, ss = _signs(function, rng)
    shape = rng.randrange(4)
    if shape == 3:  # 16-digit ratio ending in 5: odd 15-digit number over 0.2 (or 2E-k)
        digits = rng.randint(14, 15)
        n = rng.randrange(10 ** (digits - 1), 10**digits) | 1
        k = rng.randint(-3, 3)
        x = xs * float(Decimal(n).scaleb(k))
        return [x, ss * float(Decimal(2).scaleb(k - 1))]
    s_digits = rng.randint(1, 15)
    s = decimal_literal(rng, s_digits, rng.randint(-6, 6), 1.0)
    steps = rng.randrange(1, 10 ** rng.randint(1, 15 - min(s_digits, 14)))
    exact = Decimal(steps) * Decimal(repr(s))
    if shape == 0 and len(exact.as_tuple().digits) <= 15:
        x = float(exact)
    elif shape == 2:  # 100 over 1.01010101010101 (100/99 cut): the cut swallows the distance
        k = rng.randint(2, 6)
        s = float(decimal15(10**k / (10**k - 1)))
        return [xs * float(10**k * rng.choice((1, 2, 5, 7))), ss * s]
    else:
        unit = Decimal(1).scaleb(exact.adjusted() - 14)
        x = float(exact.quantize(unit, context=_CONTEXT) + unit * rng.choice((-1, 1)))
    return [xs * x, ss * s]


def _edge(function: str, rng: random.Random) -> list[Arg]:
    """Zeros, a significance at or above the number, the subnormal-ratio boundary of
    probes/t2715 and the literal range's ends."""
    xs, ss = _signs(function, rng)
    x = decimal_literal(rng, rng.randint(1, 15), rng.randint(-6, 12), xs)
    shape = rng.randrange(6)
    if shape == 0:
        return [0.0, decimal_literal(rng, rng.randint(1, 6), rng.randint(-6, 6), ss)]
    if shape == 1:
        return [x, 0.0]
    if shape == 2:  # one step or less: |significance| >= |number|
        return [x, ss * abs(x) * rng.choice((1.0, 1.5, 10.0, 1e6))]
    if shape == 3:  # a ratio beside 2^-1022: 1E-307 over 4 to 100, 2.2250738585072E-308 over ~1
        return [xs * rng.choice((1e-307, 1e-300, MIN_LITERAL)), ss * rng.choice((1.0, 4.0, 10.0, 100.0, 1e8))]
    if shape == 4:  # the largest literals over significances near 1
        return [xs * MAX_LITERAL * rng.choice((1.0, 0.5, 0.1)), ss * rng.choice((1.0, 3.0, 0.7, 1e7))]
    return [xs * rng.choice((1.0, 0.5, 2.5, 1.5)), ss * rng.choice((1.0, 2.0, 0.5))]


def _draw(function: str) -> Callable[[str, random.Random], list[Arg]]:
    def draw(stratum: str, rng: random.Random) -> list[Arg]:
        """[number, significance(, mode)] by stratum: typical ratios 1 to 1E6, tails at and beside
        a step, the edges of `_edge`, extreme magnitudes 1E-300 to 1E307 with ratios up to 1E300,
        and both arguments log-uniform over the whole literal range."""
        if stratum == "tail":
            args = _near_step(function, rng)
        elif stratum == "edge":
            args = _edge(function, rng)
        elif stratum == "extreme":
            xs, ss = _signs(function, rng)
            x = decimal_literal(rng, rng.randint(1, 15), rng.randint(-300, 306), xs)
            e = exponent10(x)
            s_exp = max(-307, min(306, e - rng.randint(-5, 300)))
            args = [x, decimal_literal(rng, rng.randint(1, 15), s_exp, ss)]
        elif stratum == "log-uniform":
            xs, ss = _signs(function, rng)
            args = [
                xs * log_uniform(rng, MIN_LITERAL, MAX_LITERAL),
                ss * log_uniform(rng, MIN_LITERAL, MAX_LITERAL),
            ]
        else:
            args = _typical(function, rng)
        return args + _mode(function, rng)

    return draw


def _definition(function: str, rule: str | None = None) -> Definition:
    """`function`'s definition; `rule` names the family member whose rule it shares (ECMA.CEILING
    is CEILING, ISO.CEILING is CEILING.PRECISE)."""
    rule = rule or function
    params = [
        Param("number", "real", Interval(), source=TIE_SOURCE),
        Param("significance", "real", Interval(), source=TIE_SOURCE),
    ]
    if rule in WITH_MODE:
        params.append(Param("mode", "int", Interval(), source=DECISION))
    sources: tuple[str, ...] = (DECISION, TIE_SOURCE, SUBNORMAL_SOURCE)
    if rule != function:
        sources += (ALIAS_SOURCES[function], ALIAS_DECISION)
    return register(
        Definition(
            function=function,
            params=tuple(params),
            value=_value(rule),
            draw=_draw(rule),
            sources=sources,
        )
    )


CEILING = _definition("CEILING")
CEILING_MATH = _definition("CEILING.MATH")
CEILING_PRECISE = _definition("CEILING.PRECISE")
FLOOR = _definition("FLOOR")
FLOOR_MATH = _definition("FLOOR.MATH")
FLOOR_PRECISE = _definition("FLOOR.PRECISE")
ECMA_CEILING = _definition("ECMA.CEILING", "CEILING")
ISO_CEILING = _definition("ISO.CEILING", "CEILING.PRECISE")
