"""Gate A definitions of ASIN, ACOS, ATAN, ATAN2, ACOT, DEGREES and RADIANS (T-3669)."""

from __future__ import annotations

import math
import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.inverse_trig import (
    ACOS,
    ACOT,
    ASIN,
    ATAN,
    ATAN2,
    DEGREES,
    RADIANS,
    _atan2_draw,
    _real_draw,
    _unit_draw,
)
from rustsheet_tools.accuracy.model import DomainError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(definition: object, *args: float) -> mpmath.mpf:
    return definition.evaluate([mp(a) for a in args])  # type: ignore[attr-defined, no-any-return]


def test_published_values() -> None:
    # numbers.md "ACOS is PI()/2 - ASIN(x)": the true ACOS(0.5) is π/3, whose nearest double is 1.0471975511965979
    # (Excel's 1.0471975511965976, through its ASIN, is one ulp below: inside the envelope)
    assert value(ACOS, 0.5) == mpmath.pi / 3
    assert float(value(ACOS, 0.5)) == 1.0471975511965979
    assert value(ASIN, 1) == mpmath.pi / 2
    assert value(ATAN, 1) == mpmath.pi / 4
    # gen-m5 A278: ACOT(-0.1) is 1.6704649792860584, in (0, π) (π/2 - atan(-0.1); Excel's double is one ulp from the
    # math, inside the envelope); ACOT(0) is π/2
    assert value(ACOT, -0.1) == mpmath.pi / 2 - mpmath.atan(mp(-0.1))
    assert abs(value(ACOT, -0.1) - mp(1.6704649792860584)) < mp("3e-16")
    assert value(ACOT, 0) == mpmath.pi / 2
    assert value(DEGREES, math.pi) == mp(math.pi) * 180 / mpmath.pi
    assert value(RADIANS, 180) == mpmath.pi


def test_asin_and_acos_take_minus_one_to_one() -> None:
    for definition in (ASIN, ACOS):
        for x in (1.000000000001, -1.5, 2.0):
            with pytest.raises(DomainError):
                value(definition, x)
        value(definition, -1)
        value(definition, 1)


def test_atan2_follows_excels_quotient() -> None:
    # numbers.md "ATAN2 is ATAN(y/x) plus or minus π"
    with pytest.raises(DomainError):
        value(ATAN2, 0, 0)  # #DIV/0!
    with pytest.raises(DomainError):
        value(ATAN2, 1e-300, 1e300)  # the quotient overflows: #NUM!
    assert value(ATAN2, 0, 1) == mpmath.pi / 2
    assert value(ATAN2, 0, -2.5) == -mpmath.pi / 2
    assert value(ATAN2, -1, 0) == mpmath.pi
    assert value(ATAN2, -1, -1) == -3 * mpmath.pi / 4
    # an underflowing quotient (Excel's 0, or ±π for x < 0) stays within Gate A's absolute floor of the math
    assert abs(value(ATAN2, 1e300, -1e-10)) < mp("1e-300")
    assert mpmath.almosteq(value(ATAN2, -1e300, 1e-10), mpmath.pi, mp("1e-38"))
    assert mpmath.almosteq(value(ATAN2, -1e300, -1e-10), -mpmath.pi, mp("1e-38"))
    # a normal quotient keeps the true atan2
    assert mpmath.almosteq(value(ATAN2, 1e300, 1e-10), mp(1e-10) / mp(1e300), mp("1e-38"))


def test_draws() -> None:
    rng = random.Random(9)
    for draw, arity in ((_unit_draw, 1), (_real_draw, 1), (_atan2_draw, 2)):
        for stratum in STRATA:
            for _ in range(30):
                args = draw(stratum, rng)
                assert len(args) == arity
                assert all(isinstance(a, float) for a in args)
