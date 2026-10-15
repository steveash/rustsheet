"""XIRR, IRR, RATE and YIELD: Excel's iterative solvers (T-1525), scored as fit-lab candidates.

Sweeps (excel-graph): probes-numeric/m9-fin-irr-rate (T-1009) and probes/solver-iteration-traces
(T-1525) are the fit set; the literal XIRR cells of five older probes are scored separately as
`xirr-oos`. probes/bond-one-period-bases (T-1525 review) adds YIELD on every basis.

- `xirr`, `xirr-oos`: the fitted bracket-and-bisection model and its rejected variants (stop rule,
  midpoint order, the bound on the step test).
- `irr`: the fitted secant in v = 1/(1 + r) (`irr_secant_v`, T-1533) and its rejected variants, and the
  value-returning Newton candidates (k steps, or to convergence, in r, x = 1 + r and v).
- `rate`: Newton candidates in r and IRR's secant applied to r (T-1533); none is Excel's method.
- `yield-one-period`: the documented closed form with each day-count choice.
- `yield`: basis-0 bonds with more than one coupon left: Newton, secant and regula falsi on PRICE.

Findings: docs/excel-semantics/financial.md ("XIRR is a bracketing bisection", "IRR returns x - 1",
"RATE iterates on the rate itself", "YIELD with one coupon period left is closed form"). The lab
scores the `#NUM!` cells of the XIRR fits too (a model with no result is `ExcelError`), and reports
which bisection iterate equals Excel's value (T-1534).
"""

from __future__ import annotations

import calendar
import contextlib
import datetime
import math
from collections.abc import Callable
from functools import partial
from typing import NamedTuple

from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.score import Candidate, ExcelError, candidate
from rustsheet_tools.fitlab.sweep import (
    Arg,
    Cell,
    UnparsedError,
    numeric_arg,
    serial_date,
)

FIT_CASES = ["probes-numeric/m9-fin-irr-rate", "probes/solver-iteration-traces"]
XIRR_OOS_CASES = [
    "probes/xirr-xnpv-whole-column-nogaps",
    "probes/fin-argument-modes",
    "probes/date-system-1900-financial-daycount",
    "probes/xirr-mismatched-empties",
    "probes/xirr-early-dates",
]
YIELD_CASES = ["probes/solver-iteration-traces", "probes/bond-one-period-bases", "probes/yield-solver-sweep"]


class NoResultError(ExcelError):
    """The model's `#NUM!` (no bracket, no convergence, a power that is not real)."""


def rpow(base: float, exponent: float) -> float:
    """base ** exponent as a real double; a negative base with a fractional exponent is NoResultError
    (Python's `**` would return a complex number)."""
    try:
        return math.pow(base, exponent)
    except ValueError:
        raise NoResultError(f"{base} ** {exponent} is not real") from None


def floats(arg: Arg) -> list[float]:
    """An array or range argument whose items are all numbers; TypeError otherwise."""
    if not isinstance(arg, list):
        msg = f"not an array or range: {arg!r}"
        raise TypeError(msg)
    return [numeric_arg(x) for x in arg]


# ----------------------------------------------------------------------------- XIRR


class Probe(NamedTuple):
    """One bisection step: the midpoint, the half-width (= |m - previous m|), XNPV(m), the values
    and their dates."""

    m: float
    h: float
    f: float
    values: list[float]
    dates: list[float]


Stop = Callable[[Probe], bool]


def xnpv(rate: float, values: list[float], dates: list[float]) -> float:
    """Excel's XNPV sum, days counted from the first date."""
    base = 1 + rate
    if base <= 0:
        raise NoResultError(f"1 + rate = {base}")
    total = 0.0
    for value, date in zip(values, dates, strict=True):
        try:
            total += value / rpow(base, (date - dates[0]) / 365)
        except (OverflowError, ZeroDivisionError):
            raise NoResultError("XNPV overflows") from None
    return total


def xirr_bracket(values: list[float], dates: list[float], guess: float) -> tuple[float, float]:
    """The first point of the trial chain whose XNPV sign differs from XNPV(0)'s, and the point
    before it. With s = guess (1e-5 for guess 0): 0, s, 2s, 4s, ... for a guess >= 0; then, if
    s < 1, 0, -s and p -> (p - 1) / 2 toward -1. A negative guess walks 0, g, (g - 1) / 2, ...
    The cap of 100 points per walk is not probed."""
    f0 = xnpv(0.0, values, dates)
    step = guess if guess != 0 else 1e-5

    def walk(first: float, nxt: Callable[[float], float]) -> tuple[float, float] | None:
        prev, point = 0.0, first
        for _ in range(100):
            try:
                fp = xnpv(point, values, dates)
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
) -> float:
    """Bisection on the bracket: `a` starts at the upper end (lower if not `from_upper`), each
    step replaces the end whose XNPV has XNPV(m)'s sign; an exact 0 does not stop it. With
    `first_value_sign`, the upper end is replaced when XNPV(m) * values[0] >= 0 instead (T-1583):
    the same end whenever XNPV at the upper end has the first value's sign and is not 0, and the
    upper end every step when the first value is 0. The midpoint is a + (b - a) / 2 (Excel's), or
    (a + b) / 2 when not `half_step`. Each midpoint is appended to `trace`, if given."""
    lo, hi = sorted(xirr_bracket(values, dates, guess))
    a, b = (hi, lo) if from_upper else (lo, hi)
    fa = xnpv(a, values, dates)
    for _ in range(200):  # the bisection cap is not probed
        m = a + (b - a) / 2 if half_step else (a + b) / 2
        fm = xnpv(m, values, dates)
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


def span_stop(c_term: float = 2e-8) -> Stop:
    """The T-1583 candidate: the fitted rule with the XNPV bound scaled by max(1, span in years),
    and no XNPV bound when the first value is 0."""
    h_ok = h_test()
    return lambda p: (
        h_ok(p) and (p.values[0] == 0 or abs(p.f) < c_term * max(map(abs, p.values)) * span_years(p))
    )


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
    stop: Stop, *, from_upper: bool = True, half_step: bool = True, first_value_sign: bool = False
) -> Callable[..., float]:
    def call(values: Arg, dates: Arg, guess: Arg = 0.1, trace: list[float] | None = None) -> float:
        model = partial(
            xirr_model,
            from_upper=from_upper,
            half_step=half_step,
            first_value_sign=first_value_sign,
            trace=trace,
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


# ----------------------------------------------------------------------------- IRR


def _npv_r(values: list[float], r: float) -> float:
    return sum(c / rpow(1 + r, i) for i, c in enumerate(values))


def _dnpv_r(values: list[float], r: float) -> float:
    return sum(-i * c / rpow(1 + r, i + 1) for i, c in enumerate(values))


def _npv_v(values: list[float], v: float) -> float:
    total = 0.0
    for c in reversed(values):
        total = total * v + c
    return total


def _dnpv_v(values: list[float], v: float) -> float:
    total = 0.0
    for i in range(len(values) - 1, 0, -1):
        total = total * v + i * values[i]
    return total


def iterate(step: Callable[[float], float], start: float, steps: int | None) -> float:
    """`steps` iterations of `step` from `start`, or (`steps` None) until an iterate repeats
    exactly; NoResultError when an iterate is not finite or 60 steps do not converge."""
    u = start
    for _ in range(steps if steps is not None else 60):
        nxt = step(u)
        if not math.isfinite(nxt):
            raise NoResultError("iterate is not finite")
        if steps is None and nxt == u:
            return u
        u = nxt
    if steps is None:
        raise NoResultError("no convergence in 60 steps")
    return u


IrrMethod = Callable[[list[float], float, int | None], float]


def irr_newton_r(values: list[float], g: float, k: int | None) -> float:
    """Newton on NPV(r) with its analytic derivative, from the guess."""
    return iterate(lambda r: r - _npv_r(values, r) / _dnpv_r(values, r), g, k)


def irr_newton_x(values: list[float], g: float, k: int | None) -> float:
    """Newton on NPV as a function of x = 1 + r; returns x - 1 (Excel's results are x - 1)."""
    return iterate(lambda x: x - _npv_r(values, x - 1) / _dnpv_r(values, x - 1), 1 + g, k) - 1


def irr_newton_v(values: list[float], g: float, k: int | None) -> float:
    """Newton on the polynomial NPV(v), v = 1/(1 + r), by Horner; returns 1/v - 1."""
    return 1 / iterate(lambda v: v - _npv_v(values, v) / _dnpv_v(values, v), 1 / (1 + g), k) - 1


def irr_fd_v(h: float) -> IrrMethod:
    """Newton in v with a forward-difference derivative of step h * v."""

    def method(values: list[float], g: float, k: int | None) -> float:
        def step(v: float) -> float:
            f = _npv_v(values, v)
            return v - f * v * h / (_npv_v(values, v * (1 + h)) - f)

        return 1 / iterate(step, 1 / (1 + g), k) - 1

    return method


def _npv_v_over_x(values: list[float], v: float) -> float:
    """NPV(v) as the sum of c_i / x^i with x = 1/v (x87-rounded divisions, numbers.md)."""
    x = X87.div(1.0, v)
    total = 0.0
    for i, c in enumerate(values):
        total += X87.div(c, rpow(x, i))
    return total


NpvV = Callable[[list[float], float], float]


def irr_secant_v(
    npv: NpvV = _npv_v_over_x,
    offset: float = 1e-3,
    tol: float = 1e-7,
    residual: float | None = 1e-7,
    max_steps: int = 50,
    *,
    retry: bool = True,
    signed: bool = True,
    trace: list[float] | None = None,
) -> IrrMethod:
    """Secant on NPV(v), v = 1/(1 + r), from v0 = 1/(1 + guess) and v1 = v0 + offset * sign(NPV(v0)).
    It stops at the first iterate with |v_(n+1) - v_n| < tol and |NPV(v_(n+1))| < residual (None: no
    residual test) and returns 1/v - 1 (`signed` False: always v0 + offset). A run that fails (no stop in `max_steps`, a flat secant, v <= 0)
    is retried once from the default guess 0.1 when `retry`. `k` (the other methods' step count) is
    ignored. Fitted on probes/solver-iteration-traces I (T-1533): the guess sweep follows the secant's
    error law, signs included; the step test separates the returned iterate from the one before it at
    1e-7 (the documented 0.00001 percent); the residual test is what makes IRR scale-dependent
    ({-20,5,...} and {-20000000,5000000,...} differ); far guesses return the guess-0.1 value."""

    def run(values: list[float], g: float) -> float:
        div = X87.div
        u0 = div(1.0, 1 + g)
        f0 = npv(values, u0)
        u1 = u0 + (offset if f0 > 0 or not signed else -offset)
        f1 = npv(values, u1)
        for _ in range(max_steps):
            if f1 == f0:
                raise NoResultError("flat secant")
            u2 = u0 - div(f0 * (u1 - u0), f1 - f0)
            if not math.isfinite(u2) or u2 <= 0:
                raise NoResultError("iterate is not a discount factor")
            f2 = npv(values, u2)
            if trace is not None:
                trace.append(div(1.0, u2) - 1)
            if abs(u2 - u1) < tol and (residual is None or abs(f2) < residual):
                return div(1.0, u2) - 1
            u0, f0, u1, f1 = u1, f1, u2, f2
        raise NoResultError(f"no convergence in {max_steps} steps")

    def method(values: list[float], g: float, k: int | None) -> float:  # noqa: ARG001 - IrrMethod's step count
        try:
            return run(values, g)
        except (NoResultError, ZeroDivisionError, OverflowError):
            if not retry or g == 0.1:
                raise
            return run(values, 0.1)

    return method


def _irr(method: IrrMethod, k: int | None) -> Callable[..., float]:
    def call(values: Arg, guess: Arg = 0.1) -> float:
        return method(floats(values), numeric_arg(guess), k)

    return call


def irr_candidates() -> list[Candidate]:
    methods: dict[str, IrrMethod] = {
        "Newton in r": irr_newton_r,
        "Newton in x=1+r": irr_newton_x,
        "Newton in v=1/(1+r)": irr_newton_v,
        "FD Newton in v, h=1e-4v": irr_fd_v(1e-4),
        "FD Newton in v, h=1e-6v": irr_fd_v(1e-6),
        "FD Newton in v, h=1e-8v": irr_fd_v(1e-8),
    }
    out = []
    for name, method in methods.items():
        out += [candidate(f"{name}, {k} steps", _irr(method, k), name) for k in range(1, 9)]
        out.append(candidate(f"{name}, converged", _irr(method, None), name))
    secants: dict[str, IrrMethod] = {
        "secant in v, fitted": irr_secant_v(),
        "secant in v, fitted, NPV by Horner": irr_secant_v(_npv_v),
        "secant in v, no residual test": irr_secant_v(residual=None),
        "secant in v, residual 1e-6": irr_secant_v(residual=1e-6),
        "secant in v, residual 1e-8": irr_secant_v(residual=1e-8),
        "secant in v, |dv| < 1e-6": irr_secant_v(tol=1e-6),
        "secant in v, |dv| < 1e-8": irr_secant_v(tol=1e-8),
        "secant in v, v1 = v0 +- 1e-4": irr_secant_v(offset=1e-4),
        "secant in v, no retry": irr_secant_v(retry=False),
        "secant in v, 20 steps": irr_secant_v(max_steps=20),
        "secant in v, 35 steps": irr_secant_v(max_steps=35),
        "secant in v, 1000 steps": irr_secant_v(max_steps=1000),
        "secant in v, v1 = v0 + 1e-3 always": irr_secant_v(signed=False),
    }
    out += [candidate(name, _irr(method, None), "secant in v") for name, method in secants.items()]
    return out


def _literal_irr(cell: Cell) -> bool:
    if not cell.args or "{" not in cell.formula:
        return False
    try:
        floats(cell.args[0])
    except TypeError:
        return False
    return all(isinstance(a, float) for a in cell.args[1:])


# ----------------------------------------------------------------------------- RATE


Annuity = tuple[float, float, float, float, float]  # nper, pmt, pv, fv, type


def rate_f(r: float, annuity: Annuity) -> float:
    """The documented RATE equation pv(1+r)^n + pmt(1+r*type)((1+r)^n - 1)/r + fv."""
    n, pmt, pv, fv, t = annuity
    x = rpow(1 + r, n)
    return pv * x + pmt * (1 + r * t) * (x - 1) / r + fv


def rate_df(r: float, annuity: Annuity) -> float:
    """d/dr of `rate_f`, analytic."""
    n, pmt, pv, _, t = annuity
    x, x1 = rpow(1 + r, n), rpow(1 + r, n - 1)
    return pv * n * x1 + pmt * t * (x - 1) / r + pmt * (1 + r * t) * (n * x1 * r - (x - 1)) / (r * r)


def _rate(k: int | None, fd: float | None) -> Callable[..., float]:
    def call(*args: Arg) -> float:
        full = [numeric_arg(x) for x in args] + [0.0, 0.0, 0.1][len(args) - 3 :]
        a: Annuity = (full[0], full[1], full[2], full[3], full[4])

        def step(r: float) -> float:
            f = rate_f(r, a)
            d = (rate_f(r + fd, a) - f) / fd if fd else rate_df(r, a)
            return r - f / d

        return iterate(step, full[5], k)

    return call


def _rate_secant(offset: float, tol: float, residual: float | None) -> Callable[..., float]:
    """IRR's secant (T-1533) on `rate_f` in r: r1 = r0 + offset * sign(f(r0)), stop when |dr| < tol and
    |f| < residual (None: no residual test), up to 100 steps. Rejected: at most 19/136."""

    def call(*args: Arg) -> float:
        full = [numeric_arg(x) for x in args] + [0.0, 0.0, 0.1][len(args) - 3 :]
        a: Annuity = (full[0], full[1], full[2], full[3], full[4])
        u0 = full[5]
        f0 = rate_f(u0, a)
        u1 = u0 + (offset if f0 > 0 else -offset)
        f1 = rate_f(u1, a)
        for _ in range(100):
            if f1 == f0:
                raise NoResultError("flat secant")
            u2 = u0 - X87.div(f0 * (u1 - u0), f1 - f0)
            if not math.isfinite(u2) or u2 <= -1:
                raise NoResultError("iterate is not a rate")
            f2 = rate_f(u2, a)
            if abs(u2 - u1) < tol and (residual is None or abs(f2) < residual):
                return u2
            u0, f0, u1, f1 = u1, f1, u2, f2
        raise NoResultError("no convergence in 100 steps")

    return call


def rate_candidates() -> list[Candidate]:
    out = [
        candidate(
            f"secant in r, r1 = r0 {o:+g}*sign f, |dr| < {t:g}, |f| < {res}",
            _rate_secant(o, t, res),
            "secant",
        )
        for o in (1e-3, -1e-3, 1e-4)
        for t in (1e-7, 1e-10)
        for res in (None, 1e-7)
    ]
    for label, fd in (("Newton in r", None), ("FD Newton in r, h=1e-8", 1e-8)):
        out += [candidate(f"{label}, {k} steps", _rate(k, fd), label) for k in range(1, 9)]
        out.append(candidate(f"{label}, converged", _rate(None, fd), label))
    return out


def _literal_rate(cell: Cell) -> bool:
    return 3 <= len(cell.args) <= 6 and all(isinstance(a, float) for a in cell.args)


# ----------------------------------------------------------------------------- YIELD


def _month_end(d: datetime.date) -> bool:
    return d.day == calendar.monthrange(d.year, d.month)[1]


def days360(start: datetime.date, end: datetime.date, european: bool) -> int:
    """DAYS360: US/NASD (basis 0, with Excel's February rules) or European (basis 4)."""
    d1, d2 = start.day, end.day
    if european:
        d1, d2 = min(d1, 30), min(d2, 30)
    else:
        feb1 = start.month == 2 and _month_end(start)
        if feb1 and end.month == 2 and _month_end(end):
            d2 = 30
        if feb1 or d1 == 31:
            d1 = 30
        if d2 == 31 and d1 >= 30:
            d2 = 30
    return (end.year - start.year) * 360 + (end.month - start.month) * 30 + (d2 - d1)


def _add_months(d: datetime.date, months: int, eom: bool) -> datetime.date:
    y, m = divmod(d.month - 1 + months, 12)
    year, month = d.year + y, m + 1
    last = calendar.monthrange(year, month)[1]
    return datetime.date(year, month, last if eom else min(d.day, last))


def schedule(
    settle: datetime.date, maturity: datetime.date, freq: int
) -> tuple[datetime.date, datetime.date, int]:
    """(previous coupon, next coupon, coupons left) stepping back from maturity by 12/freq
    months; a month-end maturity keeps coupons on month ends. UnparsedError for a frequency
    other than 1, 2 or 4, or more than 4 * 10,000 coupons (Excel's dates end in 9999)."""
    if freq not in (1, 2, 4):
        msg = f"frequency {freq}"
        raise UnparsedError(msg)
    eom = _month_end(maturity)
    ncd = maturity
    for n in range(4 * 10_000):
        try:
            pcd = _add_months(maturity, -12 // freq * (n + 1), eom)
        except ValueError:
            break  # before year 1
        if pcd <= settle:
            return pcd, ncd, n + 1
        ncd = pcd
    msg = f"no coupon schedule from {maturity} back to {settle}"
    raise UnparsedError(msg)


def day_counts(
    settle: datetime.date, maturity: datetime.date, freq: int, basis: int, *, period_e: bool
) -> tuple[float, float, float, float, int]:
    """(A, E, DSR, E - A, N): accrued days, period length, settlement-to-maturity days in the
    basis, PRICE's E - A, and coupons left. `period_e` takes E as the actual period length on
    bases 1, 2 and 3 (YIELD's one-period form) instead of 360/f and 365/f on bases 2 and 3."""
    pcd, ncd, n = schedule(settle, maturity, freq)
    if basis in (0, 4):
        a = float(days360(pcd, settle, basis == 4))
        e = 360.0 / freq
        dsr = float(days360(settle, maturity, basis == 4))
    else:
        a = float((settle - pcd).days)
        dsr = float((maturity - settle).days)
        nominal = {1: float((ncd - pcd).days), 2: 360.0 / freq, 3: 365.0 / freq}[basis]
        e = float((ncd - pcd).days) if period_e or basis == 1 else nominal
    return a, e, dsr, e - a, n


def yield_closed(rate: float, pr: float, red: float, freq: int, counts: tuple[float, float, float]) -> float:
    """The documented one-period YIELD, in the documentation's order of operations;
    `counts` = (A, E, DSR)."""
    a, e, dsr = counts
    return (
        ((red / 100 + rate / freq) - (pr / 100 + (a / e * rate / freq)))
        / (pr / 100 + (a / e) * (rate / freq))
        * (freq * e / dsr)
    )


def _bond(args: tuple[Arg, ...]) -> tuple[datetime.date, datetime.date, float, float, float, int, int]:
    s, m, rate, pr, red, freq = (numeric_arg(x) for x in args[:6])
    basis = int(numeric_arg(args[6])) if len(args) > 6 else 0
    return serial_date(s), serial_date(m), rate, pr, red, int(freq), basis


def _yield_one(period_e: bool, prices_dsr: bool) -> Callable[..., float]:
    """The closed form with E as the period on bases 1-3 (`period_e`) and DSR = E - A
    (`prices_dsr`, PRICE's) instead of the basis' settlement-to-maturity count."""

    def call(*args: Arg) -> float:
        settle, maturity, rate, pr, red, freq, basis = _bond(args)
        a, e, dsr, e_minus_a, _ = day_counts(settle, maturity, freq, basis, period_e=period_e)
        return yield_closed(rate, pr, red, freq, (a, e, e_minus_a if prices_dsr else dsr))

    return call


def yield_one_candidates() -> list[Candidate]:
    return [
        candidate(
            "closed form, DSR, E = period on bases 1-3 (fitted)", _yield_one(True, False), "closed form"
        ),
        candidate(
            "closed form, DSR, E = 360/f, 365/f on bases 2, 3", _yield_one(False, False), "closed form"
        ),
        candidate("closed form, E - A (PRICE's)", _yield_one(False, True), "closed form"),
    ]


def _is_bond(cell: Cell) -> bool:
    return 6 <= len(cell.args) <= 7 and all(isinstance(a, float) for a in cell.args)


def _coupons_left(cell: Cell) -> tuple[int, int] | None:
    """(coupons left, basis) of a bond cell, or None when its arguments are not a valid bond."""
    if not _is_bond(cell):
        return None
    try:
        settle, maturity, _, _, _, freq, basis = _bond(cell.args)
        return schedule(settle, maturity, freq)[2], basis
    except UnparsedError:
        return None


def _one_period(cell: Cell) -> bool:
    found = _coupons_left(cell)
    return found is not None and found[0] == 1


def _many_periods_basis0(cell: Cell) -> bool:
    found = _coupons_left(cell)
    return found is not None and found[0] > 1 and found[1] == 0


Bond = tuple[float, float, int, float, float, float, int]  # rate, redemption, freq, A, E, DSC, N


def price(y: float, bond: Bond) -> float:
    """PRICE with N > 1 coupons left (documented form, DSC = E - A)."""
    rate, red, freq, a, e, dsc, n = bond
    base = 1 + y / freq
    coupons = sum(100 * rate / freq / rpow(base, k - 1 + dsc / e) for k in range(1, n + 1))
    return red / rpow(base, n - 1 + dsc / e) + coupons - 100 * rate / freq * a / e


def dprice(y: float, bond: Bond) -> float:
    """d/dy of `price`, analytic."""
    rate, red, freq, _, e, dsc, n = bond
    base = 1 + y / freq
    total = 0.0
    for k in range(1, n + 1):
        t = k - 1 + dsc / e
        total -= t * 100 * rate / freq / rpow(base, t + 1) / freq
    t = n - 1 + dsc / e
    return total - t * red / rpow(base, t + 1) / freq


def _bond_price(args: tuple[Arg, ...]) -> tuple[Callable[[float], float], Bond, float]:
    """The cell's PRICE(y) function on basis 0, its bond tuple and the target price."""
    settle, maturity, rate, pr, red, freq, _ = _bond(args)
    a, e, _, dsc, n = day_counts(settle, maturity, freq, 0, period_e=False)
    bond: Bond = (rate, red, freq, a, e, dsc, n)
    return partial(price, bond=bond), bond, pr


def _newton_yield(start: float, tol: float) -> Callable[..., float]:
    """Newton on PRICE(y) - pr from `start`, returning the iterate after a step below `tol`."""

    def call(*args: Arg) -> float:
        price_of, bond, pr = _bond_price(args)
        y = start
        for _ in range(100):
            step = (price_of(y) - pr) / dprice(y, bond)
            y -= step
            if abs(step) < tol:
                return y
        raise NoResultError("Newton did not converge")

    return call


def _newton_yield_residual(start: Callable[[Bond, float], float], tol: float) -> Callable[..., float]:
    """Newton on PRICE(y) - pr from `start(bond, pr)`, returning the first iterate whose |PRICE - pr|
    is below `tol`, without the step it would take next (T-1583: Excel's values stop below the root
    with a residual up to 5e-8)."""

    def call(*args: Arg) -> float:
        price_of, bond, pr = _bond_price(args)
        y = start(bond, pr)
        for _ in range(100):
            residual = price_of(y) - pr
            if abs(residual) < tol:
                return y
            y -= residual / dprice(y, bond)
        raise NoResultError("Newton did not converge")

    return call


YIELD_STARTS: dict[str, Callable[[Bond, float], float]] = {
    "0": lambda _bond, _pr: 0.0,
    "the coupon rate": lambda bond, _pr: bond[0],
    "the current yield (rate * 100 / pr)": lambda bond, pr: bond[0] * 100 / pr,
}


def _secant_yield(tol: float) -> Callable[..., float]:
    """The secant on PRICE(y) - pr from yields 0 and 1."""

    def call(*args: Arg) -> float:
        price_of, _, pr = _bond_price(args)
        y0, y1 = 0.0, 1.0
        f0, f1 = price_of(y0) - pr, price_of(y1) - pr
        for _ in range(100):
            y2 = y1 - f1 * (y1 - y0) / (f1 - f0)
            y0, f0, y1 = y1, f1, y2
            f1 = price_of(y1) - pr
            if abs(y1 - y0) < tol:
                return y1
        raise NoResultError("secant did not converge")

    return call


def _falsi_yield(*args: Arg) -> float:
    price_of, _, pr = _bond_price(args)
    return regula_falsi(price_of, pr)


def regula_falsi(price_of: Callable[[float], float], pr: float) -> float:
    """LibreOffice's getYield_ (scaddins analysishelper.cxx): regula falsi from yields 0 and 1,
    doubling the upper yield while the price is not bracketed, to rtl::math::approxEqual."""

    def approx_equal(x: float, y: float) -> bool:
        return x == y or (abs(x - y) < abs(x) * 2**-48 and abs(x - y) < abs(y) * 2**-48)

    y1, y2 = 0.0, 1.0
    p1, p2 = price_of(y1), price_of(y2)
    y_mid, p_mid = (y2 - y1) * 0.5, 0.0
    for _ in range(100):
        if approx_equal(p_mid, pr):
            break
        p_mid = price_of(y_mid)
        if approx_equal(pr, p1):
            return y1
        if approx_equal(pr, p2):
            return y2
        if approx_equal(pr, p_mid):
            return y_mid
        if pr < p2:
            y2 *= 2
            p2 = price_of(y2)
            y_mid = (y2 - y1) * 0.5
        else:
            if pr < p_mid:
                y1, p1 = y_mid, p_mid
            else:
                y2, p2 = y_mid, p_mid
            y_mid = y2 - (y2 - y1) * ((pr - p2) / (p1 - p2))
    return y_mid


def yield_candidates() -> list[Candidate]:
    out = [
        candidate(f"Newton from {s}, |step| < {t:g}", _newton_yield(s, t), "Newton")
        for s in (0.0, 0.05, 0.1)
        for t in (1e-6, 1e-8, 1e-10, 1e-12)
    ]
    out += [candidate(f"secant from 0, 1, |step| < {t:g}", _secant_yield(t), "secant") for t in (1e-8, 1e-10)]
    out.append(candidate("LibreOffice regula falsi", _falsi_yield, "regula falsi"))
    out += [
        candidate(
            f"Newton from {name}, |PRICE - pr| < {t:g}", _newton_yield_residual(start, t), "Newton, residual"
        )
        for name, start in YIELD_STARTS.items()
        for t in (1e-7, 5e-8, 1e-8)
    ]
    return out


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
        description="XIRR out of sample: the literal XIRR cells of five older probes",
        cases=XIRR_OOS_CASES,
        pattern=r"^=XIRR\((.*)\)$",
        candidates=xirr_candidates,
        where=_literal_xirr,
        errors=True,
    ),
    Fit(
        name="irr",
        description="IRR fit set: the fitted secant in v (T-1533) and its variants, Newton candidates in r, x and v",
        cases=FIT_CASES,
        pattern=r"^=IRR\((.*)\)$",
        candidates=irr_candidates,
        where=_literal_irr,
    ),
    Fit(
        name="rate",
        description="RATE literal cells of the fit set: Newton and secant candidates in r (none fits)",
        cases=FIT_CASES,
        pattern=r"^=RATE\((.*)\)$",
        candidates=rate_candidates,
        where=_literal_rate,
    ),
    Fit(
        name="yield-one-period",
        description="YIELD with one coupon period left, every basis: the closed form's day counts",
        cases=YIELD_CASES,
        pattern=r"^=YIELD\((.*)\)$",
        candidates=yield_one_candidates,
        where=_one_period,
    ),
    Fit(
        name="yield",
        description="YIELD, basis 0, more than one coupon left: Newton, secant, regula falsi on PRICE",
        cases=YIELD_CASES,
        pattern=r"^=YIELD\((.*)\)$",
        candidates=yield_candidates,
        where=_many_periods_basis0,
    ),
]
