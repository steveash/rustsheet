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
    """B(a,b) for half-integer shapes, in doubles, by the engine's recurrence (`f_test_remainder`,
    fisher.rs, which carries it in double-double) from B(1,1) = 1,
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


def _pairwise(values: list[float]) -> float:
    if len(values) <= 2:
        return sum(values)
    half = len(values) // 2
    return _pairwise(values[:half]) + _pairwise(values[half:])


def pairwise_variance(values: list[float]) -> float:
    """The two-pass variance with both sums pairwise."""
    mean = _pairwise(values) / len(values)
    return _pairwise([(v - mean) * (v - mean) for v in values]) / (len(values) - 1)


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
        lx, ly = math.log(x), math.log(y)
        if power == "log1p":
            lx, ly = (math.log1p(-y), ly) if x > y else (lx, math.log1p(-x))
        elif power == "lndiff":
            lt = math.log(total)
            lx = math.log(df_small * small if ratio_free else df_small) - lt
            ly = math.log(df_big * big if ratio_free else df_big * (big / small)) - lt
        if b == 1.0:
            upper = math.exp(a * math.log(x))
        elif a == 1.0:
            upper = -math.expm1(b * math.log1p(-x))
        elif power in ("log1p", "lndiff"):
            upper = math.exp(a * lx) * math.exp(b * ly) * r_correct(x, a, b)
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
        candidate("ln of the larger by log1p", variant(power="log1p"), "powers"),
        candidate("ln as a difference", variant(power="lndiff"), "powers"),
        candidate("pairwise variance", variant(variance=pairwise_variance), "variance"),
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


def ibeta_finite(x: float, a: float, b: float, *, b_first: bool = True) -> float:
    """I_x(a,b) for half-integer shapes as a finite sum (A&S 26.6.4-26.6.8 in incomplete-beta
    form): the base I_x(a0,b0) (a0, b0 in {1/2, 1}: (2/pi) asin(sqrt x), sqrt x, 1 - sqrt(1-x) or
    x), then I_x(p,q+1) = I_x(p,q) + T/q and I_x(p+1,q) = I_x(p,q) - T/p with
    T = x^p y^q / B(p,q) carried incrementally, in doubles."""
    y = 1.0 - x
    p = 1.0 if a % 1 == 0 else 0.5
    q = 1.0 if b % 1 == 0 else 0.5
    sx, sy = math.sqrt(x), math.sqrt(y)
    if p == q == 0.5:
        value, term = 2.0 / math.pi * math.asin(sx), sx * sy / math.pi  # x^p y^q / B(p,q)
    elif p == 0.5:
        value, term = sx, sx * y / 2.0
    elif q == 0.5:
        value, term = 1.0 - sy, x * sy / 2.0
    else:
        value, term = x, x * y
    steps = [("b", q, b), ("a", p, a)] if b_first else [("a", p, a), ("b", q, b)]
    for which, _, _ in steps:
        while (q < b) if which == "b" else (p < a):
            if which == "b":
                value += term / q
                term *= y * (p + q) / q
                q += 1.0
            else:
                value -= term / p
                term *= x * (p + q) / p
                p += 1.0
    return value


def f_test_finite(*, b_first: bool) -> Callable[..., float]:
    """F.TEST with the upper tail I_x(a,b) as `ibeta_finite`."""

    def call(*args: Arg) -> float:
        shape = _shape(Cell("", "", "", "", tuple(args), 0.0))
        if shape is None:
            raise ExcelError(code="#DIV/0!")
        x, _, a, b = shape
        upper = ibeta_finite(x, a, b, b_first=b_first)
        return 2.0 * min(upper, 1.0 - upper)

    return call


FITS.append(
    Fit(
        name="f-test-finite",
        description="F.TEST/FTEST (T-2571): the upper tail as A&S's finite sums from the asin base",
        cases=_cases(),
        pattern=_PATTERN,
        candidates=lambda: [
            candidate("finite, b first", f_test_finite(b_first=True), "finite"),
            candidate("finite, a first", f_test_finite(b_first=False), "finite"),
            candidate("model", variant(), "model"),
        ],
        where=in_fit,
    )
)


def ibeta_even_b(x: float, a: float, b: float, power: str) -> float:
    """A&S 26.6.4 for an even df_big (b an integer): I_x(a,b) = x^a sum_(k<b) (a)_k/k! y^k, a
    sum of positive terms, with x^a by exp(a ln x), libm pow, or sqrt(x) times x^(a-1/2)."""
    y = 1.0 - x
    if power == "explog":
        lead = math.exp(a * math.log(x))
    elif power == "pow":
        lead = x**a
    else:
        lead = (math.sqrt(x) if a % 1 else 1.0) * x ** int(a)
    total, term, k = 1.0, 1.0, 0.0
    while k + 1.0 < b:
        term *= (a + k) * y / (k + 1.0)
        total += term
        k += 1.0
    return lead * total


def f_test_as(power: str, *, finite_above: float) -> Callable[..., float]:
    """A&S for an even df_big; the finite sums (a first) when the tail is above `finite_above`;
    otherwise the T-2361 model."""
    model = variant()

    def call(*args: Arg) -> float:
        shape = _shape(Cell("", "", "", "", tuple(args), 0.0))
        if shape is None:
            raise ExcelError(code="#DIV/0!")
        x, _, a, b = shape
        if b % 1 == 0:
            upper = ibeta_even_b(x, a, b, power)
        else:
            upper = ibeta_finite(x, a, b, b_first=False)
            if not upper > finite_above:
                return model(*args)
        return 2.0 * min(upper, 1.0 - upper)

    return call


FITS.append(
    Fit(
        name="f-test-as",
        description="F.TEST/FTEST (T-2571): A&S 26.6.4 for an even df_big, finite sums above a tail size",
        cases=_cases(),
        pattern=_PATTERN,
        candidates=lambda: (
            [
                candidate(f"A&S {pw}, finite above {t:g}", f_test_as(pw, finite_above=t), pw)
                for pw in ("explog", "pow", "sqrt")
                for t in (1e-3, 1e-6, 1e-300)
            ]
            + [candidate("model", variant(), "model")]
        ),
        where=in_fit,
    )
)
