"""The fit lab's exact arithmetic: p-bit rounding, the x87 double-rounding model, and the
correctly rounded sqrt/exp/ln/pow (fitlab/exact.py)."""

from __future__ import annotations

import math
import random
from fractions import Fraction

import pytest

from rustsheet_tools.fitlab.exact import (
    DOUBLE_MIN_EXP,
    ROUNDINGS,
    X87,
    Ext,
    X87Insn,
    double_round,
    exact_value,
    exp_at,
    exp_cr,
    extended_value,
    ln_at,
    ln_cr,
    pi_cr,
    pow_cr,
    round_to_bits,
    rounded_to,
    sqrt_cr,
    square_and_multiply,
    to_double,
    to_double_value,
    x87_value,
)


def test_round_to_bits_is_nearest_ties_to_even() -> None:
    assert round_to_bits(Fraction(5, 4), 2) == Fraction(1)  # 1.01b: tie, down to even 1.0b
    assert round_to_bits(Fraction(7, 4), 2) == Fraction(2)  # 1.11b: tie, up to even 10b
    assert round_to_bits(Fraction(-7, 4), 2) == Fraction(-2)
    assert round_to_bits(Fraction(1, 3), 4) == Fraction(11, 32)  # 0.0101|01..b rounds up
    assert round_to_bits(0, 53) == 0
    assert round_to_bits(1 + 2**-60, 53) == 1  # below half an ulp of 1
    with pytest.raises(ValueError, match="precision"):
        round_to_bits(1, 0)


def test_round_to_bits_53_is_the_double_rounding_of_cpython() -> None:
    rng = random.Random(7)
    for _ in range(500):
        x = Fraction(rng.getrandbits(120), rng.getrandbits(90) | 1)
        assert to_double(round_to_bits(x, 53)) == float(x)


def test_min_exp_gives_subnormals() -> None:
    assert round_to_bits(Fraction(3, 2**1076), 53, DOUBLE_MIN_EXP) == Fraction(1, 2**1074)
    assert round_to_bits(Fraction(1, 2**1076), 53, DOUBLE_MIN_EXP) == 0  # below half the grid step
    # 2.5 grid steps is a tie that rounds down to the even 2 steps.
    assert round_to_bits(Fraction(5, 2**1075), 53, DOUBLE_MIN_EXP) == Fraction(2, 2**1074)
    assert round_to_bits(Fraction(1, 2**1022), 53, DOUBLE_MIN_EXP) == Fraction(1, 2**1022)


def test_x87_division_rounds_twice() -> None:
    # docs/excel-semantics/numbers.md "Division rounds twice": =1/(2-2^-52) is 0.5 in Excel.
    assert 1 / (2 - 2**-52) == 0.5000000000000001  # correctly rounded
    assert X87.div(1, 2 - 2**-52) == 0.5
    assert X87.div(1, 1 + 26955297717044 * 2**-52) == 0.994050331987417
    assert double_round(Fraction(1) / Fraction(2 - 2**-52)) == 0.5


def test_x87_operations_agree_with_doubles_away_from_midpoints() -> None:
    assert X87.add(0.1, 0.2) == 0.1 + 0.2
    assert X87.mul(3.0, 1 / 3) == 3.0 * (1 / 3)
    assert X87.sub(1.0, 2**-53) == 1.0 - 2**-53
    assert X87.sqrt(2.0) == math.sqrt(2.0)


def test_ext_keeps_64_bit_intermediates() -> None:
    # 1 + 2^-60 survives in a 64-bit register and its square keeps the 2^-59 term until the end.
    one = Ext(1.0) + 2.0**-60
    assert one.value == 1 + Fraction(1, 2**60)
    assert (one * one - 1.0).to_double() == 2.0**-59
    assert (1.0 + 2.0**-60) * (1.0 + 2.0**-60) - 1.0 == 0.0  # in doubles it is lost
    assert (1 / Ext(3.0)).value == round_to_bits(Fraction(1, 3), 64)


def test_sqrt_cr_matches_ieee_sqrt() -> None:
    rng = random.Random(3)
    for _ in range(300):
        x = rng.uniform(1e-300, 1e300) * rng.choice([1.0, 1e-8, 1e8])
        assert to_double(sqrt_cr(x)) == math.sqrt(x)
    assert sqrt_cr(Fraction(9, 4)) == Fraction(3, 2)
    assert sqrt_cr(0) == 0


@pytest.mark.parametrize(
    ("value", "expected"),
    [
        (exp_cr(1), math.e),
        (exp_cr(0), 1.0),
        (exp_cr(-1), 0.36787944117144233),
        (ln_cr(2), math.log(2)),
        (ln_cr(1), 0.0),
        (ln_cr(1 + 2**-52), 2.2204460492503128e-16),
        (ln_cr(10), math.log(10)),
        (pow_cr(2, 0.5), math.sqrt(2)),
        (pow_cr(4, 0.5), 2.0),
        (pow_cr(0.25, 1.5), 0.125),
        (pow_cr(10, -0.3), 0.5011872336272722),
        (pow_cr(3, 40), 3.0**40),
    ],
)
def test_transcendentals_are_correctly_rounded(value: Fraction, expected: float) -> None:
    assert to_double(value) == expected


def test_exp_beats_libm_on_a_hard_case() -> None:
    # A near-halfway case: the decimal-backed value must be the nearer double.
    x = 92.2998891171253
    assert to_double(exp_cr(x)) == 1.2171175476232716e40


def test_extended_precision_results_are_64_bit() -> None:
    e64 = exp_cr(1, 64)
    assert e64 == round_to_bits(e64, 64) and e64 != round_to_bits(e64, 53)
    assert to_double(e64) == math.e


def test_out_of_range_inputs_raise() -> None:
    with pytest.raises(OverflowError):
        exp_cr(1e10)
    with pytest.raises(ValueError, match="non-positive"):
        ln_cr(0)
    with pytest.raises(ValueError, match="negative"):
        sqrt_cr(-1)
    with pytest.raises(ValueError, match="no exact value"):
        round_to_bits(math.inf, 53)


def test_square_and_multiply_rounds_each_product() -> None:
    assert square_and_multiply(3, 5, exact_value) == 243
    assert square_and_multiply(1.5, 0, to_double_value) == 1
    b = math.sqrt(5)
    once = to_double(square_and_multiply(b, 10, to_double_value))
    twice = to_double(square_and_multiply(b, 10, x87_value))
    assert (once, twice) == (3125.000000000003, 3125.0000000000023)  # Excel: ...023 (T-1495)
    with pytest.raises(ValueError, match="n >= 0"):
        square_and_multiply(2, -1, x87_value)


def test_x87_instruction_model_and_pi() -> None:
    assert float(pi_cr()) == math.pi
    assert round_to_bits(pi_cr(64), 64) == pi_cr(64)
    assert X87Insn.pow(2, 10) == 1024  # exact powers of two survive fyl2x/f2xm1/fscale
    assert float(X87Insn.exp(1)) == math.e
    assert float(X87Insn.ln(math.e)) == pytest.approx(1.0, rel=1e-18)
    for v in (X87Insn.pow(1.2, 2), X87Insn.exp(0.3), X87Insn.ln(10)):
        assert round_to_bits(v, 64) == v


def test_huge_exponents_stop_at_the_x87_range() -> None:
    with pytest.raises(OverflowError):
        square_and_multiply(1.5, 1 << 53, extended_value)
    with pytest.raises(OverflowError):
        X87Insn.pow(1.0000001, 9007199254740992)
    assert square_and_multiply(0.5, 1 << 53, to_double_value) == 0  # a double underflows to 0


def test_shared_roundings() -> None:
    v = Fraction(1, 3)
    assert exact_value(v) is v
    assert ROUNDINGS["double"](v) == Fraction(1 / 3)
    assert ROUNDINGS["ext"](v) == round_to_bits(v, 64)
    assert rounded_to(128)(v) == round_to_bits(v, 128)
    assert ln_at(53)(Fraction(10)) == Fraction(math.log(10))
    assert exp_at(64)(Fraction(1)) == exp_cr(1, 64)
