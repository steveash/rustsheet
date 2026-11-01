"""Gate A definitions of the CEILING/FLOOR family (T-3673, decision 0244) against numbers.md."""

from __future__ import annotations

import random
from fractions import Fraction

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.ceiling_floor import (
    CEILING,
    CEILING_MATH,
    CEILING_PRECISE,
    FLOOR,
    FLOOR_MATH,
    FLOOR_PRECISE,
    cut15,
)
from rustsheet_tools.accuracy.model import Definition, DomainError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler

ALL = (CEILING, CEILING_MATH, CEILING_PRECISE, FLOOR, FLOOR_MATH, FLOOR_PRECISE)


def value(definition: Definition, *args: float) -> mpmath.mpf:
    with mpmath.workdps(40):
        return definition.evaluate([mpmath.mpf(a) for a in args])


@pytest.mark.parametrize(
    ("definition", "args", "expected"),
    [
        # numbers.md "FLOOR.PRECISE: a 16th-digit tie in the ratio cut goes away from zero"
        # (probes/t2687-floor-ceiling-nonunit-ties): 666666666666533 / 2 = 333333333333266.5 cuts
        # away from zero to ...267, so FLOOR.MATH steps up to 666666666666534.
        (FLOOR_MATH, (666666666666533, -2, 0), "666666666666534"),
        # 333333333333333 / 0.2 = 1666666666666665, a tie at the 16th digit: up to ...670.
        (CEILING, (333333333333333, 0.2), "333333333333334"),
        (FLOOR, (333333333333333, 0.2), "333333333333334"),
        # 0.3 / 0.1 is 3 at the written decimals: no binary noise in the math.
        (FLOOR, (0.3, 0.1), "0.3"),
        (CEILING, (0.3, 0.1), "0.3"),
        # Signs: a negative significance steps away from zero in CEILING, toward it in FLOOR.
        (CEILING, (-2.5, -1), "-3"),
        (FLOOR, (-2.5, -1), "-2"),
        (CEILING, (-2.5, 1), "-2"),
        (FLOOR, (-2.5, 1), "-3"),
        (CEILING_PRECISE, (-2.5, -1), "-2"),
        (FLOOR_PRECISE, (2.5, -1), "2"),
        (CEILING_MATH, (-2.5, 1, 1), "-3"),
        (CEILING_MATH, (-2.5, 1, 0), "-2"),
        (FLOOR_MATH, (-2.5, 1, 1), "-2"),
        (FLOOR_MATH, (-2.5, -1, 0), "-3"),
        (CEILING_MATH, (2.5, 1, 1), "3"),
        # Zero significance: 0, and FLOOR of 0 by 0 is 0.
        (CEILING, (2.1, 0), "0"),
        (FLOOR, (0, 0), "0"),
        (CEILING_MATH, (5, 0, 1), "0"),
        # numbers.md "CEILING and FLOOR of a subnormal ratio are 0" (probes/t2715).
        (CEILING_MATH, (1e-307, 10, 0), "0"),
        (FLOOR_MATH, (-1e-307, 100, 0), "0"),
        (CEILING_MATH, (1e-307, 4, 0), "4"),
        # The cut swallows the distance to the step: 100/1.01010101010101 = 99.0000000000000099.
        (CEILING, (100, 1.01010101010101), "99.99999999999999"),  # 99 steps
    ],
)
def test_the_math_reproduces_numbers_md(
    definition: Definition, args: tuple[float, ...], expected: str
) -> None:
    with mpmath.workdps(40):
        assert value(definition, *args) == mpmath.mpf(expected)


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (CEILING, (2.5, -1)),  # numbers.md: #NUM! in FLOOR and CEILING only
        (FLOOR, (2.5, -1)),
        (FLOOR, (2.5, 0)),  # #DIV/0!
        (CEILING, (1e300, 1e-10)),  # the ratio overflows
        (FLOOR_PRECISE, (-1e300, 1e-10)),
    ],
)
def test_domain_errors(definition: Definition, args: tuple[float, ...]) -> None:
    with pytest.raises(DomainError):
        value(definition, *args)


def test_cut15_ties_away_and_carries() -> None:
    assert cut15(Fraction(1666666666666665)) == 1666666666666670
    assert cut15(Fraction(-1666666666666665)) == -1666666666666670
    assert cut15(Fraction(9999999999999995, 10)) == 1000000000000000
    assert cut15(Fraction(1, 3)) == Fraction(333333333333333, 10**15)
    assert cut15(Fraction(0)) == 0


@pytest.mark.parametrize("definition", ALL)
def test_draws_are_writable_literals(definition: Definition) -> None:
    sampler = Sampler(definition, random.Random(7))
    for stratum in STRATA:
        for index in range(30):
            args = sampler.draw(stratum, index)
            assert len(args) == len(definition.params)
            for a in args[:2]:
                assert isinstance(a, float)
                assert a == 0 or 2.2e-308 < abs(a) < 1e308
