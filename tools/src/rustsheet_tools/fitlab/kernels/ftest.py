"""F.TEST / FTEST (T-2361, T-2571): the remainder R of the upper tail exp(a ln x) exp(b ln y) R.

The sweep is every F.TEST/FTEST cell on file whose two arguments are plain ranges or array
literals (`sweep` reads them; a nested call is unparsed). Each candidate forms Excel's structure
(docs/excel-semantics/statistics.md "F.TEST / FTEST: larger variance on top"): f = the larger
two-pass sample variance over the smaller, a = df_small/2, b = df_big/2,
x = df_small/(df_big f + df_small), y = df_big f/(df_big f + df_small), the upper tail
u = I_x(a, b) = exp(a ln x) exp(b ln y) R, and p = 2 min(u, 1 - u). Only R differs:

- `R correctly rounded`: I_x(a,b)/(x^a y^b) from the 70-digit decimal incomplete beta (the engine's
  model since T-2361, which sums it in double-double);
- `R 2F1 forward`, `R 2F1 backward`: 2F1(a+b, 1; a+1; x)/(a B(a,b)) summed in doubles from the
  first term or from the last, with B(a,b) by the engine's recurrence in doubles;
- `R 2F1 forward, Gamma B`: the same forward sum over B(a,b) from correctly rounded Gammas.

The closed forms a = 1 or b = 1 (df_small or df_big 2) are the engine's `beta_reg_complement`
and Excel's exactly (statistics.md), so the candidates use them too: b = 1 is exp(a ln x) and a = 1
is -expm1(b ln1p(-x)). A cell with a + b above 200, or a non-half-integer shape, is out of the
fit (`where`).
"""

from __future__ import annotations

import decimal
import math
import re
from collections.abc import Callable

from rustsheet_tools import repo_root
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.kernels.beta import _CTX, _ibeta_decimal, gamma_cr
from rustsheet_tools.fitlab.score import Candidate, ExcelError, candidate
from rustsheet_tools.fitlab.sweep import Arg, Cell


def _cases() -> list[str]:
    """Every gen-m9, probes and probes-numeric case with an F.TEST or FTEST cell (T-2571); cells
    with nested calls are unparsed."""
    found = []
    for suite in ("gen-m9", "probes", "probes-numeric"):
        root = repo_root() / "eval" / "suites" / suite / "cases"
        for path in sorted(root.rglob("*.case.json")):
            if re.search(r'"=(?:_xlfn\.)?F\.?TEST\(', path.read_text(encoding="utf-8")):
                found.append(f"{suite}/{path.relative_to(root).as_posix().removesuffix('.case.json')}")
    return found


_PATTERN = r"^=_?(?:xlfn\.)?F\.?TEST\(([^()]+)\)$"
_SHAPE_MAX = 200.0
_MAX_TERMS = 100_000  # directive 5: the series converges far sooner for a + b <= 200

Remainder = Callable[[float, float, float], float]  # (x, a, b) -> R


def _numbers(arg: Arg) -> list[float]:
    """The numbers of a range or array argument (text, booleans and blanks skipped)."""
    values = arg if isinstance(arg, list) else [arg]
    return [v for v in values if isinstance(v, float) and not isinstance(v, bool)]


def two_pass_variance(values: list[float]) -> float:
    """Sample variance as two passes in doubles: the mean, then the sum of squared deviations."""
    mean = 0.0
    for v in values:
        mean += v
    mean /= len(values)
    total = 0.0
    for v in values:
        d = v - mean
        total += d * d
    return total / (len(values) - 1)


def _shape(cell: Cell) -> tuple[float, float, float, float] | None:
    """(x, y, a, b) of the cell's upper tail, or None when a variance is not defined."""
    first, second = (_numbers(a) for a in cell.args[:2])
    if len(first) < 2 or len(second) < 2:
        return None
    v1, v2 = two_pass_variance(first), two_pass_variance(second)
    if v1 == 0.0 or v2 == 0.0:
        return None
    (big, n_big), (small, n_small) = sorted([(v1, len(first)), (v2, len(second))], reverse=True)
    f = big / small
    df_big, df_small = n_big - 1.0, n_small - 1.0
    total = df_big * f + df_small
    return df_small / total, df_big * f / total, df_small / 2.0, df_big / 2.0


def in_fit(cell: Cell) -> bool:
    """Half-integer shapes with a + b up to 200 (the engine's series range); errors are out."""
    if cell.error is not None or len(cell.args) != 2:
        return False
    shape = _shape(cell)
    return shape is not None and shape[2] + shape[3] <= _SHAPE_MAX


def beta_recurrence(a: float, b: float) -> float:
    """B(a,b) for half-integer shapes, in doubles, by the engine's recurrence from B(1,1) = 1,
    B(1/2,1) = B(1,1/2) = 2 or B(1/2,1/2) = pi."""
    p = 1.0 if a % 1 == 0 else 0.5
    q = 1.0 if b % 1 == 0 else 0.5
    beta = math.pi if p == q == 0.5 else (2.0 if 0.5 in (p, q) else 1.0)
    while p < a:
        beta = beta * p / (p + q)
        p += 1.0
    while q < b:
        beta = beta * q / (p + q)
        q += 1.0
    return beta


def _series_terms(x: float, a: float, b: float) -> list[float]:
    """The terms t_0 = 1, t_(n+1) = t_n (a+b+n) x/(a+1+n) of 2F1(a+b, 1; a+1; x), in doubles."""
    terms = [1.0]
    n = 0.0
    while len(terms) < _MAX_TERMS:
        t = terms[-1] * (a + b + n) * x / (a + 1.0 + n)
        terms.append(t)
        if t < 2.0**-60 * len(terms):
            break
        n += 1.0
    return terms


def r_forward(x: float, a: float, b: float) -> float:
    """R as the forward double sum of the 2F1 series over a B(a,b) (recurrence)."""
    return sum(_series_terms(x, a, b)) / (a * beta_recurrence(a, b))


def r_backward(x: float, a: float, b: float) -> float:
    """R as the series summed from its smallest term up."""
    total = 0.0
    for t in reversed(_series_terms(x, a, b)):
        total += t
    return total / (a * beta_recurrence(a, b))


def r_forward_gamma(x: float, a: float, b: float) -> float:
    """The forward sum over B(a,b) = Gamma(a)Gamma(b)/Gamma(a+b) from correctly rounded Gammas."""
    return sum(_series_terms(x, a, b)) / (a * (gamma_cr(a) * gamma_cr(b) / gamma_cr(a + b)))


def r_correct(x: float, a: float, b: float) -> float:
    """I_x(a,b)/(x^a y^b) correctly rounded (70-digit decimal)."""
    dx, da, db = decimal.Decimal(x), decimal.Decimal(a), decimal.Decimal(b)
    ibeta = _ibeta_decimal(dx, da, db)
    powers = _CTX.multiply(_CTX.power(dx, da), _CTX.power(_CTX.subtract(1, dx), db))
    return float(_CTX.divide(ibeta, powers))


def f_test(remainder: Remainder) -> Callable[..., float]:
    """A candidate F.TEST with `remainder` as R (closed forms at a = 1 or b = 1)."""

    def call(*args: Arg) -> float:
        cell_args = Cell("", "", "", "", tuple(args), 0.0)
        shape = _shape(cell_args)
        if shape is None:
            raise ExcelError(code="#DIV/0!")
        x, y, a, b = shape
        if b == 1.0:
            upper = math.exp(a * math.log(x))
        elif a == 1.0:
            upper = -math.expm1(b * math.log1p(-x))
        else:
            upper = math.exp(a * math.log(x)) * math.exp(b * math.log(y)) * remainder(x, a, b)
        return 2.0 * min(upper, 1.0 - upper)

    return call


def f_test_candidates() -> list[Candidate]:
    """The T-2571 remainders."""
    return [
        candidate("R correctly rounded", f_test(r_correct), "model"),
        candidate("R 2F1 forward", f_test(r_forward), "double series"),
        candidate("R 2F1 backward", f_test(r_backward), "double series"),
        candidate("R 2F1 forward, Gamma B", f_test(r_forward_gamma), "double series"),
    ]


FITS = [
    Fit(
        name="f-test",
        description="F.TEST/FTEST (T-2361, T-2571): exp(a ln x) exp(b ln y) R, candidate remainders R",
        cases=_cases(),
        pattern=_PATTERN,
        candidates=f_test_candidates,
        where=in_fit,
    ),
]


def one_pass_variance(values: list[float]) -> float:
    """Sample variance as (n sum x^2 - (sum x)^2)/(n(n-1)) in doubles (VAR.S's one-pass form)."""
    n = float(len(values))
    s = sq = 0.0
    for v in values:
        s += v
        sq += v * v
    return (n * sq - s * s) / (n * (n - 1.0))


def variant(
    *,
    variance: Callable[[list[float]], float] = two_pass_variance,
    ratio_free: bool = False,
    power: str = "explog",
) -> Callable[..., float]:
    """F.TEST with R correctly rounded and one structural choice changed: the variance, x and y
    formed from the variances without f (`ratio_free`), or the powers (`pow`: libm x**a;
    `joint`: exp(a ln x + b ln y))."""

    def call(*args: Arg) -> float:
        first, second = (_numbers(a) for a in args[:2])
        v1, v2 = variance(first), variance(second)
        (big, n_big), (small, n_small) = sorted([(v1, len(first)), (v2, len(second))], reverse=True)
        df_big, df_small = n_big - 1.0, n_small - 1.0
        if ratio_free:
            total = df_big * big + df_small * small
            x, y = df_small * small / total, df_big * big / total
        else:
            f = big / small
            total = df_big * f + df_small
            x, y = df_small / total, df_big * f / total
        a, b = df_small / 2.0, df_big / 2.0
        if b == 1.0:
            upper = math.exp(a * math.log(x))
        elif a == 1.0:
            upper = -math.expm1(b * math.log1p(-x))
        elif power == "pow":
            upper = x**a * y**b * r_correct(x, a, b)
        elif power == "joint":
            upper = math.exp(a * math.log(x) + b * math.log(y)) * r_correct(x, a, b)
        else:
            upper = math.exp(a * math.log(x)) * math.exp(b * math.log(y)) * r_correct(x, a, b)
        return 2.0 * min(upper, 1.0 - upper)

    return call


def structure_candidates() -> list[Candidate]:
    """The structural alternatives around the T-2361 model."""
    return [
        candidate("model", variant(), "model"),
        candidate("one-pass variance", variant(variance=one_pass_variance), "variance"),
        candidate("x, y without f", variant(ratio_free=True), "x"),
        candidate("pow powers", variant(power="pow"), "powers"),
        candidate("joint exp", variant(power="joint"), "powers"),
    ]


FITS.append(
    Fit(
        name="f-test-structure",
        description="F.TEST/FTEST (T-2571): the T-2361 model with the variance, x/y or powers changed",
        cases=_cases(),
        pattern=_PATTERN,
        candidates=structure_candidates,
        where=in_fit,
    )
)
