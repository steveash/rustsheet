"""Gate A definitions of SIN, COS, TAN, COT, SEC and CSC (T-3668) against published values and Excel's domain rules."""

from __future__ import annotations

import math
import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.trig import COS, COT, CSC, CUT, SEC, SIN, TAN, _draw
from rustsheet_tools.accuracy.model import DomainError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def test_the_math_is_the_true_function_of_the_double() -> None:
    # numbers.md "Trig of a huge angle": the correctly rounded SIN(PI()) is 1.2246467991473532E-16 (Excel's
    # 1.22514845490862E-16 is the π₆₄ reduction, which the engine reproduces; it passes on Excel's side)
    assert float(SIN.evaluate([mp(math.pi)])) == 1.2246467991473532e-16
    assert float(COS.evaluate([mp(math.pi / 2)])) == 6.123233995736766e-17
    assert SIN.evaluate([mp(0)]) == 0
    assert TAN.evaluate([mp(0)]) == 0
    assert COS.evaluate([mp(0)]) == 1
    assert SEC.evaluate([mp(0)]) == 1  # gen-m5 A175: SEC(0) is 1
    x = mp(0.7)
    assert mpmath.almosteq(TAN.evaluate([x]) * COT.evaluate([x]), 1, mp("1e-38"))
    assert mpmath.almosteq(SIN.evaluate([x]) * CSC.evaluate([x]), 1, mp("1e-38"))
    assert mpmath.almosteq(COS.evaluate([x]) * SEC.evaluate([x]), 1, mp("1e-38"))


def test_cot_and_csc_have_a_pole_at_zero_only() -> None:
    # gen-m5 (excel-graph): COT(0) and CSC(0) are #DIV/0!
    for definition in (COT, CSC):
        with pytest.raises(DomainError):
            definition.evaluate([mp(0)])
        assert definition.evaluate([mp(1e-300)]) > 0
    for definition in (SIN, COS, TAN, SEC):
        definition.evaluate([mp(0)])


def test_the_argument_cut_is_two_to_the_27() -> None:
    # numbers.md "Trig of a huge angle": #NUM! from |x| = 2^27, fractions included
    for definition in (SIN, COS, TAN, COT, SEC, CSC):
        for x in (CUT, -CUT, CUT + 0.25, 1e300):
            with pytest.raises(DomainError):
                definition.evaluate([mp(x)])
        definition.evaluate([mp(CUT - 1)])
        definition.evaluate([mp(-(CUT - 0.25))])


def test_draws_cover_the_multiples_of_half_pi_and_the_cut() -> None:
    rng = random.Random(5)
    for stratum in STRATA:
        for _ in range(40):
            (x,) = _draw(stratum, rng)
            assert isinstance(x, float)
    tails = [_draw("tail", rng)[0] for _ in range(200)]
    # every tail point lies within 0.1 of a multiple of π/2
    assert all(abs(t / (math.pi / 2) - round(t / (math.pi / 2))) * math.pi / 2 <= 0.1 for t in tails)
    edges = [_draw("edge", rng)[0] for _ in range(200)]
    assert any(abs(e) >= CUT for e in edges)
    assert any(e == 0 for e in edges)
