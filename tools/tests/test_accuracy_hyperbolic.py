"""Gate A definitions of SINH, COSH, TANH, COTH, SECH and CSCH (T-3670) against published values and Excel's domain rules."""

from __future__ import annotations

import math
import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.hyperbolic import (
    COSH,
    COTH,
    CSCH,
    EXP_CUT,
    RECIPROCAL_LIMIT,
    SECH,
    SINH,
    TANH,
    UNPROBED,
)
from rustsheet_tools.accuracy.model import Definition, DomainError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf
ALL = (SINH, COSH, TANH, COTH, SECH, CSCH)


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(definition: Definition, x: float) -> mpmath.mpf:
    result = definition.evaluate([mp(x)])
    assert isinstance(result, mpmath.mpf)
    return result


def test_the_math_is_the_true_function_of_the_double() -> None:
    # published values: sinh(1) = 1.1752011936438014569, cosh(1) = 1.5430806348152437785, tanh(1) = 0.76159415595576488812
    assert mpmath.almosteq(
        value(SINH, 1), mp("1.1752011936438014568823818505956008151557"), mp("1e-38"), abs_eps=0
    )
    assert mpmath.almosteq(
        value(COSH, 1), mp("1.5430806348152437784779056207570616826015"), mp("1e-38"), abs_eps=0
    )
    assert mpmath.almosteq(
        value(TANH, 1), mp("0.7615941559557648881194582826047935904128"), mp("1e-38"), abs_eps=0
    )
    x = 0.7
    assert mpmath.almosteq(value(TANH, x) * value(COTH, x), 1, mp("1e-38"), abs_eps=0)
    assert mpmath.almosteq(value(SINH, x) * value(CSCH, x), 1, mp("1e-38"), abs_eps=0)
    assert mpmath.almosteq(value(COSH, x) * value(SECH, x), 1, mp("1e-38"), abs_eps=0)
    # tiny x keeps its digits: sinh(x) ≈ x (1 + x²/6), relative, not absolute
    assert mpmath.almosteq(value(SINH, 1e-200), mp(1e-200), mp("1e-38"), abs_eps=0)
    assert value(SINH, 0) == 0
    assert value(TANH, 0) == 0
    assert value(COSH, 0) == 1
    assert value(SECH, 0) == 1


def test_sinh_and_cosh_overflow_where_e_to_the_x_does() -> None:
    # numbers.md (T-2169): SINH(710) and COSH(-710) are #NUM!; SINH(709.78) is a number
    assert math.isclose(EXP_CUT, 709.782712893384)
    for definition in (SINH, COSH):
        for x in (710.0, -710.0, 1e300):
            with pytest.raises(DomainError):
                definition.evaluate([mp(x)])
        assert value(definition, 709.78) > mp("8.9e307")
    # odd and even: the -709.78 side is a number too
    assert value(SINH, -709.78) == -value(SINH, 709.78)
    assert value(COSH, -709.78) == value(COSH, 709.78)


def test_the_reciprocals_have_no_overflow_cut() -> None:
    # numbers.md: CSCH(710) and SECH(710) are 0 in Excel, an underflow of a true value inside the absolute floor
    # sech(710) = csch(710) = 2e^-710 (to 1e-600 relative) = 8.95257245135...E-309, kept, not cut to a domain error
    for definition in (SECH, CSCH):
        expected = 2 * mpmath.exp(-710)
        assert mpmath.almosteq(value(definition, 710.0), expected, mp("1e-38"), abs_eps=0)
        assert mpmath.almosteq(value(definition, 1000.0), 2 * mpmath.exp(-1000), mp("1e-38"), abs_eps=0)
    assert mpmath.nstr(value(SECH, 710.0), 12) == "8.95257245135e-309"
    assert value(TANH, 1e300) <= 1
    assert value(COTH, 1e300) >= 1


def test_coth_and_csch_have_a_pole_at_zero_only() -> None:
    # gen-m5 4a971c8636db937d Calc!A289: CSCH(0) is #DIV/0!; COTH(0) is 1/TANH(0)
    for definition in (COTH, CSCH):
        with pytest.raises(DomainError):
            definition.evaluate([mp(0)])
        assert value(definition, 1e-300) > mp("1e299")
    for definition in (SINH, COSH, TANH, SECH):
        definition.evaluate([mp(0)])


def test_draws_avoid_the_unprobed_cut_and_huge_reciprocal_arguments() -> None:
    rng = random.Random(7)
    for definition in ALL:
        assert definition.draw is not None
        for stratum in STRATA:
            for _ in range(200):
                (x,) = definition.draw(stratum, rng)
                assert isinstance(x, float)
                assert not UNPROBED[0] < abs(x) < UNPROBED[1]
                if definition in (SECH, CSCH):
                    assert abs(x) <= RECIPROCAL_LIMIT
                if definition in (SINH, COSH) and stratum != "edge":
                    assert abs(x) <= UNPROBED[0]
    sinh_draw = SINH.draw
    assert sinh_draw is not None
    edges = {abs(sinh_draw("edge", rng)[0]) for _ in range(300)}
    assert {0.0, 709.78, 710.0, 1e300} <= edges
