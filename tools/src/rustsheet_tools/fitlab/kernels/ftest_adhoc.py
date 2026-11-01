"""F.TEST / FTEST (T-2681): the T-2571 scores that came from ad hoc scripts, as committed fits.

- `f-test-half`: one degree of freedom on each side (a = b = 1/2), Excel's closed form
  2 asin(sqrt x)/pi against (2/pi) asin(sqrt x), 2/pi atan(sqrt(x)/sqrt(y)) and the series model
  (fisher.rs takes the first, T-2571);
- `f-test-nr` (the engine's closed forms at a = 1, b = 1 and a = b = 1/2 in every candidate):
  Numerical Recipes' `betai` in doubles (exp(lnB + a ln x + b ln y) or three
  separate exps, `betacf`, the symmetry switch, ln y or ln(1 - x), libm lgamma), the T-2361
  model with ln y as log1p(-x), and the model at the exact variance ratio (the tail at the
  exact f instead of the two-pass double f).

The engine row of `f-test` scores TOMS 708 bratio's port for both shapes above 1 (T-2681,
fisher.rs `f_test_p`); its Python reference is docs/research/t2681/score_bratio.py.
"""

from __future__ import annotations

import math
from collections.abc import Callable
from fractions import Fraction

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.kernels.ftest import (
    _PATTERN,
    _cases,
    _numbers,
    _shape,
    in_fit,
    r_correct,
    variant,
)
from rustsheet_tools.fitlab.score import Candidate, ExcelError, candidate
from rustsheet_tools.fitlab.sweep import Arg, Cell

_CF_TERMS = 300  # NR's MAXIT is 100; 300 is Cephes' cap and converges every cell on file
_CF_EPS = 3e-16
_FPMIN = 1e-300


def _shape_of(args: tuple[Arg, ...]) -> tuple[float, float, float, float]:
    shape = _shape(Cell("", "", "", "", args, 0.0))
    if shape is None:
        raise ExcelError(code="#DIV/0!")
    return shape


def half_shape(form: str) -> Callable[..., float]:
    """F.TEST at a = b = 1/2 by one closed form of I_x(1/2, 1/2); other shapes are the model."""
    model = variant()

    def call(*args: Arg) -> float:
        x, y, a, b = _shape_of(args)
        if a != 0.5 or b != 0.5:
            return model(*args)
        if form == "asin":
            upper = 2.0 * math.asin(math.sqrt(x)) / math.pi
        elif form == "2/pi asin":
            upper = 2.0 / math.pi * math.asin(math.sqrt(x))
        else:
            upper = 2.0 / math.pi * math.atan(math.sqrt(x) / math.sqrt(y))
        return 2.0 * min(upper, 1.0 - upper)

    return call


def closed_form(x: float, a: float, b: float) -> float | None:
    """The engine's closed forms of the upper tail (fisher.rs): b = 1 is exp(a ln x), a = 1 is
    -expm1(b ln1p(-x)) and a = b = 1/2 is 2 asin(sqrt x)/pi; None for other shapes."""
    if b == 1.0:
        return math.exp(a * math.log(x))
    if a == 1.0:
        return -math.expm1(b * math.log1p(-x))
    if a == b == 0.5:
        return 2.0 * math.asin(math.sqrt(x)) / math.pi
    return None


def _is_half(cell: Cell) -> bool:
    if not in_fit(cell):
        return False
    _, _, a, b = _shape_of(cell.args)
    return a == b == 0.5


def betacf(a: float, b: float, x: float) -> float:
    """Numerical Recipes' modified Lentz continued fraction for the incomplete beta, in doubles."""
    qab, qap, qam = a + b, a + 1.0, a - 1.0
    c, d = 1.0, 1.0 - qab * x / qap
    d = 1.0 / (d if abs(d) >= _FPMIN else _FPMIN)
    h = d
    for m in range(1, _CF_TERMS + 1):
        m2 = 2 * m
        for aa in (
            m * (b - m) * x / ((qam + m2) * (a + m2)),
            -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2)),
        ):
            d = 1.0 + aa * d
            d = 1.0 / (d if abs(d) >= _FPMIN else _FPMIN)
            c = 1.0 + aa / c
            c = c if abs(c) >= _FPMIN else _FPMIN
            h *= d * c
        if abs(d * c - 1.0) < _CF_EPS:
            break
    return h


def nr_betai(*, joint: bool, log_one_minus_x: bool, switch: bool) -> Callable[..., float]:
    """F.TEST with the upper tail NR's `betai`: bt = exp(lgamma(a+b) - lgamma(a) - lgamma(b) +
    a ln x + b ln y) (`joint`) or the three factors exp'd apart; ln y as ln(1 - x) when
    `log_one_minus_x`; with `switch`, NR's reflection at x >= (a+1)/(a+b+2)."""

    def call(*args: Arg) -> float:
        x, y, a, b = _shape_of(args)
        closed = closed_form(x, a, b)
        if closed is not None:
            return 2.0 * min(closed, 1.0 - closed)
        ly = math.log(1.0 - x) if log_one_minus_x else math.log(y)
        ln_g = math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b)
        if joint:
            bt = math.exp(ln_g + a * math.log(x) + b * ly)
        else:
            bt = math.exp(ln_g) * math.exp(a * math.log(x)) * math.exp(b * ly)
        if switch and x >= (a + 1.0) / (a + b + 2.0):
            upper = 1.0 - bt * betacf(b, a, y) / b
        else:
            upper = bt * betacf(a, b, x) / a
        return 2.0 * min(upper, 1.0 - upper)

    return call


def _exact_variance(values: list[float]) -> Fraction:
    exact = [Fraction(v) for v in values]
    mean = sum(exact, Fraction(0)) / len(exact)
    return sum(((v - mean) ** 2 for v in exact), Fraction(0)) / (len(exact) - 1)


def model_variant(change: str) -> Callable[..., float]:
    """The T-2361 model (R correctly rounded) with ln y as log1p(-x) (`log1p`) or at the exact
    variance ratio (`exact f`)."""

    def call(*args: Arg) -> float:
        x, y, a, b = _shape_of(args)
        closed = closed_form(x, a, b)
        if closed is not None:
            return 2.0 * min(closed, 1.0 - closed)
        if change == "exact f":
            first, second = (_numbers(v) for v in args[:2])
            v1, v2 = _exact_variance(first), _exact_variance(second)
            (big, n_big), (small, n_small) = sorted([(v1, len(first)), (v2, len(second))], reverse=True)
            total = (n_big - 1) * (big / small) + (n_small - 1)
            x, y = float((n_small - 1) / total), float((n_big - 1) * (big / small) / total)
            ly = math.log(y)
        else:
            ly = math.log1p(-x)
        upper = math.exp(a * math.log(x)) * math.exp(b * ly) * r_correct(x, a, b)
        return 2.0 * min(upper, 1.0 - upper)

    return call


def nr_candidates() -> list[Candidate]:
    """NR's betai in its variants, and the model's two changes."""
    found = [
        candidate(
            f"NR betai, {'joint exp' if joint else 'three exps'}, {'ln(1-x)' if l1 else 'ln y'}"
            f"{', switch' if switch else ''}",
            nr_betai(joint=joint, log_one_minus_x=l1, switch=switch),
            "NR betai",
        )
        for joint in (True, False)
        for l1 in (False, True)
        for switch in (True, False)
    ]
    return [
        *found,
        candidate("model, ln y as log1p(-x)", model_variant("log1p"), "model"),
        candidate("model at the exact f", model_variant("exact f"), "model"),
        candidate("model", variant(), "model"),
    ]


FITS = [
    Fit(
        name="f-test-half",
        description="F.TEST/FTEST (T-2681): a = b = 1/2, Excel's closed form against its rewritings",
        cases=_cases(),
        pattern=_PATTERN,
        candidates=lambda: [
            candidate("2 asin(sqrt x)/pi", half_shape("asin"), "closed form"),
            candidate("(2/pi) asin(sqrt x)", half_shape("2/pi asin"), "closed form"),
            candidate("2/pi atan(sqrt(x)/sqrt(y))", half_shape("atan"), "closed form"),
            candidate("model", variant(), "model"),
        ],
        where=_is_half,
    ),
    Fit(
        name="f-test-nr",
        description="F.TEST/FTEST (T-2681): NR betai, ln y as log1p(-x), and the model at the exact f",
        cases=_cases(),
        pattern=_PATTERN,
        candidates=nr_candidates,
        where=in_fit,
    ),
]
