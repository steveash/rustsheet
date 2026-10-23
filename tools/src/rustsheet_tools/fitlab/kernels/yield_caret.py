"""YIELD's solver candidates on PRICE discounted with Excel's `^` (T-2207), as fit `yield-caret`.

The engine's PRICE and ODDFPRICE discount each cash flow with Excel's `^` (`arithmetic::power`;
docs/excel-semantics/financial.md "PRICE and ODDFPRICE discount with Excel's `^`"), while the `yield`
fit in kernels/solvers.py prices with libm. This fit scores every `yield` candidate again with the
price (and its derivative) taken through kernels/power.py's `excel_model`, on the same cells.
"""

from __future__ import annotations

from collections.abc import Callable
from fractions import Fraction

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.kernels import solvers
from rustsheet_tools.fitlab.kernels.power import excel_model
from rustsheet_tools.fitlab.score import Candidate
from rustsheet_tools.fitlab.sweep import Arg, Cell


def caret_pow(base: float, exponent: float) -> float:
    """base ^ exponent with Excel's `^`; a power that is not real is NoResultError, as in `solvers.rpow`."""
    if base <= 0 and not exponent.is_integer():
        msg = f"{base} ^ {exponent} is not real"
        raise solvers.NoResultError(msg)
    return excel_model(Fraction(base), exponent)


def caret_priced(fn: Callable[..., float]) -> Callable[..., float]:
    """`fn` with every PRICE (and PRICE') it takes discounted by Excel's `^` instead of libm."""

    def call(*args: Arg) -> float:
        token = solvers.PRICE_POWER.set(caret_pow)
        try:
            return fn(*args)
        finally:
            solvers.PRICE_POWER.reset(token)

    return call


def _caret_evaluate(c: Candidate) -> Callable[[Cell], float]:
    return lambda cell: caret_priced(lambda: c.evaluate(cell))()


def caret_candidates() -> list[Candidate]:
    """Each `yield` candidate, `^`-priced."""
    return [
        Candidate(f"{c.name}, ^-priced", _caret_evaluate(c), c.family) for c in solvers.yield_candidates()
    ]


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
