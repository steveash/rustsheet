"""Gate A definitions of INT, EVEN, ODD, MROUND, TRUNC, ECMA.CEILING and ISO.CEILING (T-3674, decision
0246) against the values numbers.md cites."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy import literal
from rustsheet_tools.accuracy.defs.ceiling_floor import CEILING, CEILING_PRECISE, ECMA_CEILING, ISO_CEILING
from rustsheet_tools.accuracy.defs.integer import EVEN, INT, MROUND, ODD
from rustsheet_tools.accuracy.defs.rounding import ROUNDDOWN, TRUNC
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler

mp = mpmath.mpf


def value(definition: Definition, *args: float) -> mpmath.mpf:
    with mpmath.workdps(50):
        return definition.evaluate([mp(a) for a in args])


def equals(got: mpmath.mpf, expected: str, rel: float = 0.0) -> bool:
    """Whether `got` is the decimal `expected`, read at 50 digits (within `rel` relative)."""
    with mpmath.workdps(50):
        want = mp(expected)
        return bool(abs(got - want) <= rel * abs(want))


@pytest.mark.parametrize(
    ("number", "expected"),
    [
        # numbers.md, "INT cuts to 15 significant digits first": the cut, then the floor, below 1E+15.
        (1 - 2**-53, "1"),
        (-(1 - 2**-53), "-1"),
        (-100000000000000.25, "-100000000000000"),
        (-999999999999999.25, "-999999999999999"),
        (-999999999999999.5, "-1000000000000000"),
        (100000000000000.5, "100000000000001"),
        (999999999999999.9, "1000000000000000"),
        # From 1E+15 up, the double is floored as it is.
        (-1000000000000000.1, "-1000000000000001"),
        (1000000000000005.5, "1000000000000005"),
        (-4503599627370495.5, "-4503599627370496"),
        (2.5, "2"),
        (-2.5, "-3"),
        (-1e-300, "-1"),
    ],
)
def test_int_reproduces_numbers_md(number: float, expected: str) -> None:
    assert equals(value(INT, number), expected)


@pytest.mark.parametrize(
    ("definition", "number", "expected"),
    [
        # numbers.md, "ODD above 2^53 adds one in binary": exact below 2^53; above it the math adds
        # the 1 exactly where Excel's binary add ties (9007199254740996 for ...994) or is a no-op.
        # Excel's cells: probes-numeric/trig-odd-even-sumxmy2-sweep Calc!F1:H66.
        (ODD, 0.0, "1"),
        (ODD, 2.0, "3"),
        (ODD, -2.5, "-3"),
        (ODD, 1.0, "1"),
        (ODD, 9007199254740994.0, "9007199254740995"),
        (EVEN, 0.0, "0"),
        (EVEN, 1.0, "2"),
        (EVEN, -2.5, "-4"),
        (EVEN, 2.0, "2"),
        (EVEN, 1e-300, "2"),
    ],
)
def test_odd_and_even(definition: Definition, number: float, expected: str) -> None:
    assert equals(value(definition, number), expected)


@pytest.mark.parametrize(
    ("number", "multiple", "expected"),
    [
        # numbers.md, "MROUND rounds the quotient up within 5E-15 of a half": the step count from the
        # double quotient, the product exact (Excel's binary products are an ulp off: 1.2000000000000002).
        (0.3, 0.2, "0.4"),
        (1.15, 0.1, "1.2"),
        (0.5, 0.2, "0.6"),
        (1234.4999999999998, 1, "1234"),
        (98765432.49999999, 1, "98765432"),
        (1.4999999999999956, 1, "2"),
        (0.49999999999999495, 1, "0"),
        (-0.3, -0.2, "-0.4"),
        (5, 0, "0"),
    ],
)
def test_mround_reproduces_numbers_md(number: float, multiple: float, expected: str) -> None:
    # The exact product of the step count and the double multiple: within 1e-16 of the decimal.
    assert equals(value(MROUND, number, multiple), expected, rel=1e-15)


def test_mround_domain_errors() -> None:
    with pytest.raises(DomainError):
        value(MROUND, 5, -2)
    with pytest.raises(DomainError):
        value(MROUND, -5, 2)
    assert value(MROUND, 0, -2) == 0
    with pytest.raises(NoValueError):
        value(MROUND, 1e300, 1e-300)


def test_shared_rules() -> None:
    # TRUNC is ROUNDDOWN (numbers.md, "TRUNC is ROUNDDOWN"); ECMA.CEILING is CEILING
    # (numbers.md, "ECMA.CEILING"); ISO.CEILING is CEILING.PRECISE.
    for args in ((123456789.123, -5), (-0.8, 1), (0.29, 2), (123.456, -400)):
        assert value(TRUNC, *args) == value(ROUNDDOWN, *args)
    for args in ((2.5, 2), (-2.5, -1), (-2.5, 1), (2.5, 0), (100, 1.01010101010101)):
        assert value(ECMA_CEILING, *args) == value(CEILING, *args)
    with pytest.raises(DomainError):
        value(ECMA_CEILING, 2.5, -1)
    for args in ((2.5, -1), (-2.5, 1), (-166666666666633.25, 0.5)):
        assert value(ISO_CEILING, *args) == value(CEILING_PRECISE, *args)


@pytest.mark.parametrize("definition", [INT, EVEN, ODD, MROUND])
def test_draws_are_writable_literals(definition: Definition) -> None:
    sampler = Sampler(definition, random.Random(11))
    for stratum in STRATA:
        for i in range(60):
            for a in sampler.draw(stratum, i):
                f = float(a)
                assert f == 0 or literal.MIN_LITERAL <= abs(f) <= literal.MAX_LITERAL
                assert float(literal.text(a)) == f
