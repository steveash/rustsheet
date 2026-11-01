"""Gate A definitions of ASINH, ACOSH, ATANH and ACOTH (T-3671) against published values and Excel's domain rules."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.inverse_hyperbolic import (
    ACOSH,
    ACOTH,
    ASINH,
    ATANH,
    SQUARE_BAND,
    _acosh_draw,
    _acoth_draw,
    _asinh_draw,
    _atanh_draw,
)
from rustsheet_tools.accuracy.model import Definition, DomainError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(definition: Definition, x: float) -> mpmath.mpf:
    result = definition.evaluate([mp(x)])
    assert isinstance(result, mpmath.mpf)
    return result


def close(a: mpmath.mpf, b: mpmath.mpf | str) -> bool:
    # abs_eps=0: mpmath otherwise uses rel_eps as an absolute tolerance too, which a tiny wrong value would pass
    return bool(mpmath.almosteq(a, mp(b), mp("1e-36"), abs_eps=0))


def test_the_math_is_the_true_function_of_the_double() -> None:
    # published: asinh(1) = ln(1 + √2), acosh(2) = ln(2 + √3), atanh(1/2) = ln(3)/2, acoth(2) = ln(3)/2
    assert close(value(ASINH, 1), mpmath.log(1 + mpmath.sqrt(2)))
    assert close(value(ACOSH, 2), mpmath.log(2 + mpmath.sqrt(3)))
    assert close(value(ATANH, 0.5), mpmath.log(3) / 2)
    assert close(value(ACOTH, 2), mpmath.log(3) / 2)
    assert close(value(ACOTH, -2), -mpmath.log(3) / 2)
    assert value(ASINH, 0) == 0
    assert value(ACOSH, 1) == 0
    assert value(ATANH, 0) == 0
    # numbers.md: ATANH(0.9999999999999999), read at 15 digits as 0.999999999999999, is 17.616361586450413
    assert abs(value(ATANH, 0.999999999999999) - mp(17.616361586450413)) < mp("4e-15")


def test_tiny_and_huge_arguments_keep_their_digits() -> None:
    # Excel's log formula gives ASINH(1E-20) = 0; the math is 1E-20 (1 - 1E-40/6), relative, not absolute
    assert close(value(ASINH, 1e-20), mp(1e-20) * (1 - mp(1e-40) / 6))
    assert close(value(ASINH, -1e-300), -mp(1e-300))
    # ACOTH(x) is about 1/x for a huge x, not a cancelled 0
    assert close(value(ACOTH, 1e300), mpmath.atanh(1 / mp(1e300)))
    assert close(value(ACOTH, 1e300), 1 / mp(1e300))
    # ACOSH just above 1: sqrt(2 (x - 1)) to first order, x - 1 being the double's own offset
    x = 1.00000000000001
    assert close(value(ACOSH, x), mpmath.acosh(mp(x)))
    assert abs(value(ACOSH, x) / mpmath.sqrt(2 * (mp(x) - 1)) - 1) < mp("1e-14")


def test_domains_follow_numbers_md() -> None:
    # ACOSH below 1, ATANH at or beyond ±1, ACOTH at or inside ±1 are #NUM! (T-1202, T-1569)
    bad = {
        ACOSH: (0.999999999999999, 0.0, -1.0),
        ATANH: (1.0, -1.0, 1.00000000000001, 2.0),
        ACOTH: (1.0, -1.0, 0.5, 0.0),
    }
    for definition, xs in bad.items():
        for x in xs:
            with pytest.raises(DomainError):
                definition.evaluate([mp(x)])


def test_asinh_and_acosh_are_num_where_x_squared_overflows() -> None:
    # numbers.md (T-2062): ASINH(±1.3E154) is finite, ±1.35E154 and 1E200 are #NUM!; ACOSH the same
    for definition in (ASINH, ACOSH):
        assert value(definition, 1.3e154) > 355
        for x in (1.35e154, 1e200, 1.7976931348623157e308):
            with pytest.raises(DomainError):
                definition.evaluate([mp(x)])
    assert close(value(ASINH, -1.3e154), -value(ASINH, 1.3e154))
    with pytest.raises(DomainError):
        ASINH.evaluate([mp(-1.35e154)])


def test_draws_avoid_the_unprobed_band() -> None:
    rng = random.Random(3)
    for draw in (_asinh_draw, _acosh_draw, _atanh_draw, _acoth_draw):
        for stratum in STRATA:
            for _ in range(200):
                (x,) = draw(stratum, rng)
                assert isinstance(x, float)
                assert not SQUARE_BAND[0] < abs(x) < SQUARE_BAND[1]
                if stratum != "edge" and draw is _acoth_draw:
                    assert abs(x) > 1
                if stratum != "edge" and draw is _atanh_draw:
                    assert abs(x) < 1
                if stratum != "edge" and draw is _acosh_draw:
                    assert x >= 1
