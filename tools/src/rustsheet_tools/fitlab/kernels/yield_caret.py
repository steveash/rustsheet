"""YIELD's solver candidates on PRICE discounted with Excel's `^` (T-2207), as fit `yield-caret`.

The engine's PRICE and ODDFPRICE discount each cash flow with Excel's `^` (`arithmetic::power`;
docs/excel-semantics/financial.md "PRICE and ODDFPRICE discount with Excel's `^`"), while the `yield`
fit in kernels/solvers.py prices with libm. This fit scores every `yield` candidate again with the
price (and its derivative) taken through kernels/power.py's `excel_model`, on the same cells.
"""

from __future__ import annotations

from collections.abc import Callable
from fractions import Fraction

from rustsheet_tools.fitlab.exact import DOUBLE, round_to_bits
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.kernels import solvers
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg, Cell


def caret_priced(fn: Callable[..., float]) -> Callable[..., float]:
    """`fn` with every PRICE (and PRICE') it takes discounted by Excel's `^` instead of libm."""

    def call(*args: Arg) -> float:
        token = solvers.PRICE_POWER.set(solvers.caret_pow)
        try:
            return fn(*args)
        finally:
            solvers.PRICE_POWER.reset(token)

    return call


def _caret_evaluate(c: Candidate) -> Callable[[Cell], float]:
    return lambda cell: caret_priced(lambda: c.evaluate(cell))()


def _start_term(args: tuple[Arg, ...]) -> tuple[float, float, int, int, int]:
    """(rate, pr, whole years, days, year days) of a bond cell; `NoResultError` for a zero term,
    where the engine's start is NaN (`#NUM!`)."""
    settle, maturity, rate, pr, _, _, basis = solvers._bond(args)
    whole, days, year = solvers.start_term(settle, maturity, basis)
    if whole == 0 and days == 0:
        raise solvers.NoResultError("a zero term")
    return rate, pr, whole, days, year


def rounded_once_yield_start(*args: Arg) -> float:
    """`solvers.fitted_yield_start` computed exactly over the exact term `whole + days/year` and
    rounded once to a double (T-2974 attempt 2, the engine's start under decision 0162)."""
    rate, pr, whole, days, year = _start_term(args)
    t, q = whole + Fraction(days, year), 100 - Fraction(pr)
    exact_start = (100 * Fraction(rate) + q / t) / ((100 + Fraction(pr)) / 2 - q / (4 * t))
    return float(round_to_bits(exact_start, DOUBLE))


def unrounded_term_yield_start(*args: Arg) -> float:
    """The start multiplied through by T, `((r T) 100 + q) / (D/2 - q/4)` with `q = 100 - pr`, in
    doubles but for `D = pr T + 100 T`, rounded once with `100 T` rounded and `T` in `pr T` the exact
    sum `whole + fl(days/year)`; every other `T` that sum rounded (T-2974 attempt 4, decision 0166;
    the engine's `yield_start` until T-3215)."""
    rate, pr, whole, days, year = _start_term(args)
    fraction = days / year
    years, q = whole + fraction, 100 - pr
    d = float(round_to_bits(Fraction(pr) * (whole + Fraction(fraction)) + Fraction(100 * years), DOUBLE))
    return ((rate * years) * 100 + q) / (d / 2 - q / 4)


def hundred_t_minus_qt_yield_start(*args: Arg) -> float:
    """The term-scaled start in doubles with the denominator written `100 T - q T/2 - q/4`:
    `((r T) 100 + q) / ((100 T - q T/2) - q/4)`, `q = 100 - pr`, `T` the double `whole + days/year`
    (T-3215: the denominators Excel's cells allow are `pr T + 2 fl(100 T) - 100 T`, which this
    leaves; the engine's `yield_start`)."""
    rate, pr, whole, days, year = _start_term(args)
    years, q = whole + days / year, 100 - pr
    return ((rate * years) * 100 + q) / ((100 * years - q * years / 2) - q / 4)


def caret_candidates() -> list[Candidate]:
    """Each `yield` candidate, `^`-priced, then Newton from the starts the engine took under
    decisions 0162 (rounded once) and 0166 (`pr T` on the unrounded term) and in T-3215
    (`100 T - q T/2`), with the short stop at 1e-10 (T-3152)."""
    out = [Candidate(f"{c.name}, ^-priced", _caret_evaluate(c), c.family) for c in solvers.yield_candidates()]
    out += [
        candidate(
            f"Newton from the {name} start, returned before a step below 1e-10, ^-priced",
            caret_priced(solvers._newton_yield_short(start, 1e-10)),
            "Newton, fitted start",
        )
        for name, start in (
            ("rounded-once (exact term)", rounded_once_yield_start),
            ("unrounded-term", unrounded_term_yield_start),
            ("100T - qT/2", hundred_t_minus_qt_yield_start),
        )
    ]
    return out


_YIELD = next(fit for fit in solvers.FITS if fit.name == "yield")

FITS = [
    Fit(
        name="yield-caret",
        description="YIELD, basis 0, more than one coupon left: the `yield` candidates on a `^`-priced PRICE",
        cases=_YIELD.cases,
        pattern=_YIELD.pattern,
        candidates=caret_candidates,
        where=_YIELD.where,
    ),
]
