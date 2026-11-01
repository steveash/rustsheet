"""Gate A definitions of ROUND, ROUNDUP and ROUNDDOWN (T-3555, decision 0223) against numbers.md."""

from __future__ import annotations

import random
import re
from decimal import Decimal

import mpmath
import pytest

from rustsheet_tools.accuracy import literal
from rustsheet_tools.accuracy.defs.rounding import MAX_DIGITS, ROUND, ROUNDDOWN, ROUNDUP, sixteenth_digit_tie
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler

mp = mpmath.mpf


def value(definition: Definition, number: float, digits: int) -> str:
    """The math value as its shortest decimal (15 digits are exact for every value here)."""
    with mpmath.workdps(40):
        return mpmath.nstr(definition.evaluate([mp(number), mp(digits)]), 15, strip_zeros=True)


@pytest.mark.parametrize(
    ("definition", "number", "digits", "expected"),
    [
        # numbers.md, "ROUND family rounds in two decimal stages": the 15-digit decimal is rounded,
        # not the exact double (2.67499999999999982236431605997495353221893310546875 for 2.675).
        (ROUND, 2.675, 2, "2.68"),
        (ROUND, 1.005, 2, "1.01"),
        (ROUND, -2.5, 0, "-3"),
        (ROUND, 12345678901234.5, 0, "12345678901235"),
        (ROUNDDOWN, 123456789.123, -5, "123400000"),
        (ROUNDDOWN, -0.5, 0, "0"),
        (ROUND, 1.5, -400, "0"),
        (ROUND, 1.5, 400, "1.5"),
        (ROUND, 1.23456789012345e300, -299, "1.2e300"),
        # The decimal result; Excel's binary add (123.46000000000001) is 1 ulp from it.
        (ROUNDUP, 123.456, 2, "123.46"),
        (ROUNDUP, 0.29, 2, "0.29"),
        (ROUNDUP, -0.0012, -3, "-1000"),
    ],
)
def test_the_math_reproduces_numbers_md(
    definition: Definition, number: float, digits: int, expected: str
) -> None:
    assert mpmath.mpf(value(definition, number, digits)) == mpmath.mpf(expected)


def test_roundup_past_the_double_range_is_a_domain_error() -> None:
    with pytest.raises(DomainError):
        value(ROUNDUP, 1.5, -400)
    with pytest.raises(DomainError):
        value(ROUNDUP, -5.146556e307, -309)
    assert mpmath.mpf(value(ROUNDUP, 9.5e307, -308)) == mpmath.mpf("1e308")
    assert mpmath.mpf(value(ROUNDDOWN, 1.5, -400)) == 0


def test_cells_the_rule_does_not_cover_have_no_value() -> None:
    # A huge num_digits is narrowed by Excel first (numbers.md, "ROUND family: huge num_digits wrap").
    with pytest.raises(NoValueError):
        value(ROUND, 5.0, 2147483647)
    with pytest.raises(NoValueError):
        value(ROUNDUP, 6.4421516092447e265, -43376)
    # A 16th-digit tie, which only a computed argument can carry.
    assert sixteenth_digit_tie(999999999999957.5)
    assert not sixteenth_digit_tie(2.675)
    assert not sixteenth_digit_tie(0.5)
    with pytest.raises(NoValueError):
        value(ROUND, 999999999999957.5, 0)


@pytest.mark.parametrize("definition", [ROUND, ROUNDUP, ROUNDDOWN])
def test_draws_are_writable_literals_within_the_digit_range(definition: Definition) -> None:
    sampler = Sampler(definition, random.Random(7))
    for stratum in STRATA:
        for i in range(80):
            number, digits = sampler.draw(stratum, i)
            assert isinstance(digits, int)
            assert abs(digits) <= MAX_DIGITS
            f = float(number)
            assert f == 0 or literal.MIN_LITERAL <= abs(f) <= literal.MAX_LITERAL
            assert float(literal.text(number)) == f
            assert not sixteenth_digit_tie(f)


def test_tail_points_sit_at_a_decimal_half_or_beside_one() -> None:
    sampler = Sampler(ROUND, random.Random(3))
    for i in range(60):
        number, digits = sampler.draw("tail", i)
        assert isinstance(digits, int)
        dropped = abs(literal.decimal15(float(number))).scaleb(digits) % 1  # below the rounding digit
        # the half itself, or the 15-digit literal next to it (0.4999…9 or 0.5000…1)
        assert re.fullmatch(r"0\.5|0\.49+|0\.50*1", str(dropped.normalize())), (number, digits)
