"""Gate A definitions of EXP, LN, LOG, LOG10, POWER, SQRT and SQRTPI (T-3672) against published values and numbers.md."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.exp_log import (
    EXP,
    EXP_BAND,
    LN,
    LOG,
    LOG10,
    POWER,
    SQRT,
    SQRTPI,
    SQRTPI_BAND,
    _exp_draw,
    _log_draw,
    _near_power_edge,
    _odd_root,
    _power_draw,
    _sqrt_draw,
)
from rustsheet_tools.accuracy.model import Definition, DomainError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(definition: Definition, *args: float) -> mpmath.mpf:
    result = definition.evaluate([mp(a) for a in args])
    assert isinstance(result, mpmath.mpf)
    return result


def close(a: mpmath.mpf, b: mpmath.mpf | str) -> bool:
    # abs_eps=0: mpmath otherwise uses rel_eps as an absolute tolerance too, which a tiny wrong value would pass
    return bool(mpmath.almosteq(a, mp(b), mp("1e-36"), abs_eps=0))


def raises(definition: Definition, *args: float) -> None:
    with pytest.raises(DomainError):
        definition.evaluate([mp(a) for a in args])


def test_published_values() -> None:
    assert close(value(EXP, 1), mpmath.e)
    assert close(value(LN, 10), "2.302585092994045684017991454684364207601")
    assert value(LOG10, 1000) == 3
    assert close(value(LOG, 8, 2), 3)
    assert close(value(LOG, 1000, 10), 3)
    assert value(SQRT, 16) == 4
    assert close(value(SQRTPI, 1), "1.772453850905516027298167483341145182798")
    # numbers.md: SQRTPI(1E+307) is 5.604991216397929E+153 in Excel, sqrt(x·π) to the bit
    assert abs(value(SQRTPI, 1e307) / mp(5.604991216397929e153) - 1) < mp("2e-16")
    # tiny arguments keep their digits (relative, not absolute)
    assert close(value(LN, 1.00000000000001), mpmath.log(mp(1.00000000000001)))
    assert close(value(EXP, -1000), mpmath.exp(-1000))
    assert close(value(SQRT, 1e-300), mpmath.sqrt(mp(1e-300)))


def test_exp_overflow_and_the_log_domains() -> None:
    raises(EXP, 710)
    raises(EXP, 2958465)
    assert value(EXP, 709.78) > mp("1.79e308")
    for definition in (LN, LOG10, SQRT, SQRTPI):
        raises(definition, -1)
    for definition in (LN, LOG10):
        raises(definition, 0)
    assert value(SQRT, 0) == 0
    assert value(SQRTPI, 0) == 0
    raises(SQRTPI, 9.99999999999999e307)  # x·π overflows: SQRTPI(10^308) is #NUM!
    # LOG: base 1 is #DIV/0! even for LOG(1,1); base 0 or negative #NUM!; a number <= 0 #NUM!
    for args in ((1, 1), (100, 1), (8, 0), (8, -2), (0, 10), (-1, 10)):
        raises(LOG, *args)


def test_power_zero_and_negative_bases() -> None:
    raises(POWER, 0, 0)  # #NUM!
    raises(POWER, 0, -1)  # #DIV/0!
    assert value(POWER, 0, 2.5) == 0
    # an odd root of a negative base: (-8)^(1/3) is minus the root of 8 at the double 1/3
    assert close(value(POWER, -8, 1 / 3), -mpmath.power(8, mp(1 / 3)))
    assert close(value(POWER, -8, -1 / 3), -mpmath.power(8, mp(-1 / 3)))
    raises(POWER, -8, 0.5)
    assert value(POWER, -2, 3) == -8
    assert value(POWER, -2, 4) == 16
    raises(POWER, -1, 4294967295)
    assert value(POWER, -1, 4294967294) == 1
    # numbers.md: 0.333333333333333 counts as 1/3, 0.33333333333333 does not; 1/7 at 15 digits does not
    assert _odd_root(0.333333333333333)
    assert not _odd_root(0.33333333333333)
    assert not _odd_root(0.142857142857143)
    assert _odd_root(0.0909090909090909)
    assert not _odd_root(0.25)


def test_power_range_rules() -> None:
    raises(POWER, 10, 309)  # overflow: #NUM!
    raises(POWER, 2, 1024)
    assert close(value(POWER, 10, -309), mp(10) ** -309)  # 1 over the overflowing power, below the floor
    raises(POWER, 1e-10, -31)  # the positive power 1E-310 underflows: #DIV/0!
    raises(POWER, 0.1, -400)
    assert close(value(POWER, 0.1, 400), mp(0.1) ** 400)  # flushed to 0 in Excel, inside the floor
    raises(POWER, 0.1, 9.99999999999999e307)  # T-2661: |y|·ln x overflows the doubles
    assert value(POWER, 1, 1e300) == 1
    assert close(value(POWER, 6, 2.5), mpmath.power(6, 2.5))  # 36·√6
    assert close(value(POWER, 6, 2.5), 36 * mpmath.sqrt(6))
    assert not _near_power_edge(10, 300)
    assert _near_power_edge(10, 308.2547155599167)


def test_draws_keep_out_of_the_unpinned_bands() -> None:
    rng = random.Random(11)
    for stratum in STRATA:
        for _ in range(200):
            (x,) = _exp_draw(stratum, rng)
            assert not EXP_BAND[0] < abs(x) < EXP_BAND[1]
            (s,) = _sqrt_draw(stratum, rng)
            assert not SQRTPI_BAND[0] < s < SQRTPI_BAND[1]
            number, base = _log_draw(stratum, rng)
            assert isinstance(number, float)
            assert isinstance(base, float)
            px, py = _power_draw(stratum, rng)
            assert isinstance(px, float)
            assert isinstance(py, float)
            if stratum != "edge":
                assert not _near_power_edge(px, py)
