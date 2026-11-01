"""Gate A definitions of SUM and AVERAGE (T-3549, decision 0237) against numbers.md's snap and overflow rules."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.aggregate import AVERAGE, MAX_ARGS, SUM, _draw, running_total
from rustsheet_tools.accuracy.defs.student_t import T_DIST
from rustsheet_tools.accuracy.eval_math import fit_args
from rustsheet_tools.accuracy.model import DomainError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler
from rustsheet_tools.accuracy.tables import compute, parse_formula

mp = mpmath.mpf
ULP1 = 2.0**-52


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def total(*args: float) -> mpmath.mpf:
    return SUM.evaluate([mp(a) for a in args])


def average(*args: float) -> mpmath.mpf:
    return AVERAGE.evaluate([mp(a) for a in args])


def test_the_final_cancelling_addition_snaps_to_zero() -> None:
    # numbers.md, "Near-zero snapping in addition and subtraction": SUM(0.1,0.2,-0.3) is 0
    assert total(0.1, 0.2, -0.3) == 0
    assert average(0.1, 0.2, -0.3) == 0  # AVERAGE snaps the sum before dividing
    # a trailing 0 is the final addend, with no opposite sign: never snapped. The math is the exact sum of the doubles
    # (Excel's double total is 5.551115123125783e-17, a double rounding of 0.1 + 0.2 away: Excel's side of Gate A)
    assert total(0.1, 0.2, -0.3, 0) == mp(0.1) + mp(0.2) - mp(0.3)
    assert float(total(0.1, 0.2, -0.3, 0)) == 2.7755575615628914e-17


def test_the_bound_is_eight_ulp_of_the_running_total() -> None:
    # T-3145: after (0.5, 0.5) the running total is 1, so -(1 + k·2^-52) snaps for k <= 7 and keeps k = 8
    assert total(0.5, 0.5, -(1 + 7 * ULP1)) == 0
    assert float(total(0.5, 0.5, -(1 + 8 * ULP1))) == -1.7763568394002505e-15
    assert float(average(0.5, 0.5, -(1 + 8 * ULP1))) == -5.921189464667501e-16
    # T-3248: a running total 1 - k·2^-52 below the last addend's exponent: snaps for k <= 3, keeps k = 4
    assert total(0.5, 0.5 - 3 * ULP1, -1) == 0
    assert float(total(0.5, 0.5 - 4 * ULP1, -1)) == -8.881784197001252e-16
    assert float(average(0.5, 0.5 - 4 * ULP1, -1)) == -2.9605947323337506e-16


def test_a_subnormal_bound_is_flushed() -> None:
    # T-2714: with addends 1.5·2^-p and -(1.5·2^-p·(1+2^-52)), p = 973 snaps and p = 974 does not
    for p, snapped in ((973, True), (974, False)):
        x = 1.5 * 2.0**-p
        y = -(x * (1 + ULP1))
        assert (total(x, y) == 0) is snapped


def test_overflow_follows_the_running_total() -> None:
    # numbers.md, aggregate overflow (T-0192)
    with pytest.raises(DomainError):
        total(9e307, 9e307, -9e307)
    with pytest.raises(DomainError):
        average(9e307, 9e307, -9e307)
    assert total(9e307, -9e307, 9e307) == mp(9e307)
    assert running_total([9e307, 9e307]) == (9e307, float("inf"))


def test_one_argument_is_its_own_sum() -> None:
    assert total(-0.3) == mp(-0.3)
    assert average(2.5) == mp(2.5)


def test_draws_write_between_one_and_255_literals() -> None:
    for definition in (SUM, AVERAGE):
        sampler = Sampler(definition, random.Random(7))
        for stratum in STRATA:
            for index in range(30):
                args = sampler.draw(stratum, index)
                assert 1 <= len(args) <= MAX_ARGS
                assert all(isinstance(a, float) for a in args)
    rng = random.Random(3)
    tails = [_draw("tail", rng) for _ in range(200)]
    assert any(len(t) >= 2 and t[-1] == 0 for t in tails)  # the trailing-zero form is drawn


def test_params_for_repeats_only_a_variadic_last_parameter() -> None:
    (number,) = SUM.params
    assert SUM.params_for(0) == (number,)  # below the declared count: unchanged, so a count check still fires
    assert SUM.params_for(1) == (number,)
    assert SUM.params_for(3) == (number, number, number)
    assert T_DIST.params_for(5) == T_DIST.params  # fixed arity: never repeated


def test_the_variadic_path_reads_any_argument_count() -> None:
    for count in (1, 3, MAX_ARGS):
        literals = ",".join(["0.1"] * (count - 1) + ["-0.3"])
        args = parse_formula(SUM, f"=SUM({literals})")
        assert args == [0.1] * (count - 1) + [-0.3]
        assert fit_args(SUM, [0.1] * (count - 1) + [-0.3]) == args
        value = compute(SUM, args).value
        assert value is not None
        assert mpmath.almosteq(mp(value), mpmath.fsum([mp(0.1)] * (count - 1)) - mp(0.3), mp("1e-40"))


def test_a_fixed_arity_definition_still_rejects_another_count() -> None:
    assert fit_args(T_DIST, [1.5, 3]) == "argument count"
    with pytest.raises(ValueError, match="2 arguments, the definition has 3"):
        parse_formula(T_DIST, "=T.DIST(1.5,3)")
    with pytest.raises(ValueError):
        T_DIST.check_domain([mp(1.5), mp(3)])
