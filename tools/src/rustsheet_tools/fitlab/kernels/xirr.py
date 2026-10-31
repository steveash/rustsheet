"""XIRR's bracketing bisection (T-1525, T-1583, T-2328), scored as fit-lab fits `xirr` (the solver fit
set, kernels/solvers.py `FIT_CASES`) and `xirr-oos` (the literal XIRR cells of six older probes): the
fitted model and its rejected variants (stop rule, midpoint order, the bound on the step test, XNPV's
power). Finding: docs/excel-semantics/financial.md ("XIRR is a bracketing bisection", "XNPV discounts
with Excel's `^`"). The lab scores the `#NUM!` cells too (a model with no result is `ExcelError`) and
reports which bisection iterate equals Excel's value (T-1534).
"""

from __future__ import annotations

import contextlib
from collections.abc import Callable
from functools import partial
from typing import NamedTuple

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.kernels.solvers import FIT_CASES, NoResultError, caret_pow, floats, rpow
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg, Cell, numeric_arg

XIRR_OOS_CASES = [
    "probes/xirr-xnpv-whole-column-nogaps",
    "probes/fin-argument-modes",
    "probes/date-system-1900-financial-daycount",
    "probes/xirr-mismatched-empties",
    "probes/xirr-early-dates",
    "probes/xirr-span-zero-first",
    "probes/xirr-back-dated-stop",
]


class Probe(NamedTuple):
    """One bisection step: the midpoint, the half-width (= |m - previous m|), XNPV(m), the values
    and their dates."""

    m: float
    h: float
    f: float
    values: list[float]
    dates: list[float]


Stop = Callable[[Probe], bool]


def xnpv(
    rate: float, values: list[float], dates: list[float], power: Callable[[float, float], float] = rpow
) -> float:
    """Excel's XNPV sum, days counted from the first date, each value divided by `power`(1 + rate,
    days / 365): libm by default; Excel's XNPV discounts with its own `^` (`caret_pow`: 541 of 541
    cells of probes/xirr-exact-root-sign, T-2328, where libm misses the sign at an exact root)."""
    base = 1 + rate
    if base <= 0:
        raise NoResultError(f"1 + rate = {base}")
    total = 0.0
    for value, date in zip(values, dates, strict=True):
        try:
            total += value / power(base, (date - dates[0]) / 365)
        except (OverflowError, ZeroDivisionError):
            raise NoResultError("XNPV overflows") from None
    return total


def xirr_bracket(
    values: list[float], dates: list[float], guess: float, power: Callable[[float, float], float] = rpow
) -> tuple[float, float]:
    """The first point of the trial chain whose XNPV sign differs from XNPV(0)'s, and the point
    before it. With s = guess (1e-5 for guess 0): 0, s, 2s, 4s, ... for a guess >= 0; then, if
    s < 1, 0, -s and p -> (p - 1) / 2 toward -1. A negative guess walks 0, g, (g - 1) / 2, ...
    The cap of 100 points per walk is not probed."""
    f0 = xnpv(0.0, values, dates, power)
    step = guess if guess != 0 else 1e-5

    def walk(first: float, nxt: Callable[[float], float]) -> tuple[float, float] | None:
        prev, point = 0.0, first
        for _ in range(100):
            try:
                fp = xnpv(point, values, dates, power)
            except NoResultError:
                return None
            if (fp > 0) != (f0 > 0):
                return prev, point
            prev, point = point, nxt(point)
        return None

    found = None
    if guess >= 0:
        found = walk(step, lambda p: p * 2)
        if found is None and step < 1:
            found = walk(-step, lambda p: (p - 1) / 2)
    else:
        found = walk(guess, lambda p: (p - 1) / 2)
    if found is None:
        raise NoResultError("no sign change on the chain")
    return found


def xirr_model(
    values: list[float],
    dates: list[float],
    guess: float,
    stop: Stop,
    *,
    from_upper: bool = True,
    half_step: bool = True,
    first_value_sign: bool = False,
    trace: list[float] | None = None,
    power: Callable[[float, float], float] = rpow,
) -> float:
    """Bisection on the bracket: `a` starts at the upper end (lower if not `from_upper`), each
    step replaces the end whose XNPV has XNPV(m)'s sign; an exact 0 does not stop it. With
    `first_value_sign`, the upper end is replaced when XNPV(m) * values[0] >= 0 instead (T-1583):
    the same end whenever XNPV at the upper end has the first value's sign and is not 0, and the
    upper end every step when the first value is 0. The midpoint is a + (b - a) / 2 (Excel's), or
    (a + b) / 2 when not `half_step`. Each midpoint is appended to `trace`, if given. XNPV discounts
    with `power` (see `xnpv`)."""
    lo, hi = sorted(xirr_bracket(values, dates, guess, power))
    a, b = (hi, lo) if from_upper else (lo, hi)
    fa = xnpv(a, values, dates, power)
    for _ in range(200):  # the bisection cap is not probed
        m = a + (b - a) / 2 if half_step else (a + b) / 2
        fm = xnpv(m, values, dates, power)
        if trace is not None:
            trace.append(m)
        if stop(Probe(m, abs(b - a) / 2, fm, values, dates)):
            return m
        replace_a = (fm * values[0] >= 0) == from_upper if first_value_sign else (fm > 0) == (fa > 0)
        if replace_a:
            a, fa = m, fm
        else:
            b = m
    raise NoResultError("bisection did not stop")


def h_test(a_term: float = 5e-9, b_term: float = 1e-8) -> Stop:
    """The step part of the stop test: half-width < a_term + b_term * |m|."""
    return lambda p: p.h < a_term + b_term * abs(p.m)


def fitted_stop(a_term: float = 5e-9, b_term: float = 1e-8, c_term: float = 2e-8) -> Stop:
    """The fitted rule: h < 5e-9 + 1e-8 |m| and |XNPV(m)| < 2e-8 max|value|."""
    h_ok = h_test(a_term, b_term)
    return lambda p: h_ok(p) and abs(p.f) < c_term * max(map(abs, p.values))


def span_years(p: Probe) -> float:
    """The years from the first date to the latest one, at least 1."""
    return max(1.0, (max(p.dates) - p.dates[0]) / 365)


def all_dates_span_years(p: Probe) -> float:
    """The years from the earliest date to the latest one, at least 1 (T-2045: a flow dated before
    the first counts)."""
    return max(1.0, (max(p.dates) - min(p.dates)) / 365)


def farthest_date_years(p: Probe) -> float:
    """The years from the first date to the farthest one either side, at least 1 (T-2045)."""
    return max(1.0, max(abs(d - p.dates[0]) for d in p.dates) / 365)


def span_stop(c_term: float = 2e-8, span: Callable[[Probe], float] = span_years) -> Stop:
    """The T-1583 candidate: the fitted rule with the XNPV bound scaled by `span` (max(1, years
    from the first date to the latest)), and no XNPV bound when the first value is 0."""
    h_ok = h_test()
    return lambda p: h_ok(p) and (p.values[0] == 0 or abs(p.f) < c_term * max(map(abs, p.values)) * span(p))


def negative_root_stop(c_term: float = 2e-8) -> Stop:
    """The T-3268 candidate: the step test, and the unscaled XNPV bound |f| < 2e-8 max|value| only
    on a negative bracket (m < 0, the downward chain). Over the 267 numeric cells with a nonzero
    first value of `xirr` and `xirr-oos`, every positive root stops at the first step-test pass,
    and every negative one fits the bound (feasible (1.93e-8, 2.05e-8] times max|value|), with no
    span factor."""
    h_ok = h_test()
    return lambda p: h_ok(p) and (p.m >= 0 or p.values[0] == 0 or abs(p.f) < c_term * max(map(abs, p.values)))


def zero_first_value_stop() -> Stop:
    """The fitted rule, or the step test alone when the first value is 0 (T-1583, rejected: 20/22)."""
    fitted, h_ok = fitted_stop(), h_test()
    return lambda p: fitted(p) or (p.values[0] == 0 and h_ok(p))


STOPS: dict[str, Stop] = {
    "h<5e-9+1e-8|m| & |f|<2e-8*max|v| (fitted)": fitted_stop(),
    "h<5e-9+1e-8|m| & |f|<1.5e-8*sum|v|": lambda p: (
        h_test()(p) and abs(p.f) < 1.5e-8 * sum(map(abs, p.values))
    ),
    "h<5e-9+1e-8|m| only": h_test(),
    "h<6e-9 only": lambda p: p.h < 6e-9,
    "h<1e-8*|m| only": lambda p: p.h < 1e-8 * abs(p.m),
    "|f|<2e-8*max|v| only": lambda p: abs(p.f) < 2e-8 * max(map(abs, p.values)),
}


def _xirr(
    stop: Stop,
    *,
    from_upper: bool = True,
    half_step: bool = True,
    first_value_sign: bool = False,
    power: Callable[[float, float], float] = rpow,
) -> Callable[..., float]:
    def call(values: Arg, dates: Arg, guess: Arg = 0.1, trace: list[float] | None = None) -> float:
        model = partial(
            xirr_model,
            from_upper=from_upper,
            half_step=half_step,
            first_value_sign=first_value_sign,
            trace=trace,
            power=power,
        )
        return model(floats(values), floats(dates), numeric_arg(guess), stop)

    return call


def _xirr_trace(call: Callable[..., float]) -> Callable[..., list[float]]:
    """The midpoints `call` visits; they are kept when it ends without a result."""

    def steps(*args: Arg) -> list[float]:
        mids: list[float] = []
        with contextlib.suppress(NoResultError):
            call(*args, trace=mids)
        return mids

    return steps


def _xirr_candidate(name: str, call: Callable[..., float], family: str) -> Candidate:
    return candidate(name, call, family, _xirr_trace(call))


def xirr_candidates() -> list[Candidate]:
    out = [_xirr_candidate(f"bisect, stop {name}", _xirr(stop), "stop rule") for name, stop in STOPS.items()]
    out += [
        _xirr_candidate("bisect fitted, m=(a+b)/2", _xirr(fitted_stop(), half_step=False), "midpoint"),
        _xirr_candidate(
            "bisect fitted, a from the lower end", _xirr(fitted_stop(), from_upper=False), "midpoint"
        ),
    ]
    # T-1583: the end replaced by the first value's sign, and the XNPV bound scaled by the span.
    out += [
        _xirr_candidate(
            "bisect, first-value sign, span-scaled |f| bound (T-1583)",
            _xirr(span_stop(), first_value_sign=True),
            "T-1583",
        ),
        _xirr_candidate(
            "bisect, first-value sign, fitted stop", _xirr(fitted_stop(), first_value_sign=True), "T-1583"
        ),
        _xirr_candidate("bisect, span-scaled |f| bound", _xirr(span_stop()), "T-1583"),
        _xirr_candidate(
            "bisect, first-value sign, |f| bound skipped when values[0] = 0",
            _xirr(zero_first_value_stop(), first_value_sign=True),
            "T-1583",
        ),
    ]
    # T-2328: XNPV discounted with Excel's `^` (probes/xirr-exact-root-sign) under the T-1583 rules.
    out += [
        _xirr_candidate(
            "bisect, first-value sign, span-scaled |f| bound, Excel ^ (T-2328)",
            _xirr(span_stop(), first_value_sign=True, power=caret_pow),
            "T-2328",
        ),
        _xirr_candidate(
            "bisect, first-value sign, fitted stop, Excel ^",
            _xirr(fitted_stop(), first_value_sign=True, power=caret_pow),
            "T-2328",
        ),
        _xirr_candidate(
            "bisect, first-value sign, step test only, Excel ^",
            _xirr(h_test(), first_value_sign=True, power=caret_pow),
            "T-2328",
        ),
    ]
    # T-2045: the span counts a flow dated before the first (probes/whole-column-empty-walkers E7, F3).
    out += [
        _xirr_candidate(
            f"bisect, first-value sign, |f| bound scaled by {name}, Excel ^ (T-2045)",
            _xirr(span_stop(span=span), first_value_sign=True, power=caret_pow),
            "T-2045",
        )
        for name, span in (
            ("earliest-to-latest years", all_dates_span_years),
            ("first-to-farthest years", farthest_date_years),
        )
    ]
    # T-3268: the XNPV bound binds only on a negative root (the A4 family against E7, F3).
    out += [
        _xirr_candidate(
            "bisect, first-value sign, |f| bound on a negative root only, Excel ^ (T-3268)",
            _xirr(negative_root_stop(), first_value_sign=True, power=caret_pow),
            "T-3268",
        )
    ]
    # The step test's bound: A in A + 1e-8 |m| (fitted 4.93e-9 <= A <= 5.09e-9), then B at A = 5e-9.
    out += [
        _xirr_candidate(f"bisect fitted, A={a:.3g}", _xirr(fitted_stop(a_term=a)), "bound A")
        for a in (4.85e-9, 4.95e-9, 5.05e-9, 5.15e-9)
    ]
    out += [
        _xirr_candidate(f"bisect fitted, B={b:.3g}", _xirr(fitted_stop(b_term=b)), "bound B")
        for b in (0.99e-8, 1.02e-8, 1.03e-8)
    ]
    return out


def _literal_xirr(cell: Cell) -> bool:
    """Literal-array XIRR cells whose values and dates are all numbers."""
    if len(cell.args) not in (2, 3) or "{" not in cell.formula:
        return False
    try:
        floats(cell.args[0])
        floats(cell.args[1])
        return len(cell.args) == 2 or isinstance(cell.args[2], float)
    except TypeError:
        return False


FITS = [
    Fit(
        name="xirr",
        description="XIRR fit set (m9-fin-irr-rate + solver-iteration-traces): bisection model and variants",
        cases=FIT_CASES,
        pattern=r"^=XIRR\((.*)\)$",
        candidates=xirr_candidates,
        where=_literal_xirr,
        errors=True,
    ),
    Fit(
        name="xirr-oos",
        description="XIRR out of sample: the literal XIRR cells of seven probes",
        cases=XIRR_OOS_CASES,
        pattern=r"^=XIRR\((.*)\)$",
        candidates=xirr_candidates,
        where=_literal_xirr,
        errors=True,
    ),
]
