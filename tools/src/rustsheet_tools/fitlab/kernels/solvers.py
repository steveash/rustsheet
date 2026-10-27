"""XIRR, IRR, RATE and YIELD: Excel's iterative solvers (T-1525), scored as fit-lab candidates.

Sweeps (excel-graph): probes-numeric/m9-fin-irr-rate (T-1009) and probes/solver-iteration-traces
(T-1525) are the fit set; the literal XIRR cells of five older probes are scored separately as
`xirr-oos`. probes/bond-one-period-bases (T-1525 review) adds YIELD on every basis.

- `xirr`, `xirr-oos`: in kernels/xirr.py.
- `irr`: the fitted secant in v = 1/(1 + r) (`irr_secant_v`, T-1533) and its rejected variants, and the
  value-returning Newton candidates (k steps, or to convergence, in r, x = 1 + r and v).
- `rate`: the T-2005 secant on the FV residual and its residual variants, traced (T-2527; a nested PMT is
  Excel's own value of that call, `Fit.calls`), and the rejected Newton candidates in r and IRR's secant
  applied to r (T-1533).
- `yield-one-period`: the documented closed form with each day-count choice.
- `yield`: basis-0 bonds with more than one coupon left: Newton, secant and regula falsi on PRICE
  (`yield-caret`, kernels/yield_caret.py: the same on PRICE discounted with Excel's `^`, T-2207).

Findings: docs/excel-semantics/financial.md ("XIRR is a bracketing bisection", "IRR returns x - 1",
"RATE is a secant on the FV residual", "YIELD with one coupon period left is closed form"). The lab
scores the `#NUM!` cells of the XIRR fits too (a model with no result is `ExcelError`), and reports
which bisection iterate equals Excel's value (T-1534).
"""

from __future__ import annotations

import calendar
import datetime
import math
from collections.abc import Callable
from contextvars import ContextVar
from fractions import Fraction
from functools import partial

from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.kernels.power import excel_model
from rustsheet_tools.fitlab.score import Candidate, ExcelError, candidate
from rustsheet_tools.fitlab.sweep import (
    Arg,
    Cell,
    UnparsedError,
    numeric_arg,
    serial_date,
)

FIT_CASES = ["probes-numeric/m9-fin-irr-rate", "probes/solver-iteration-traces"]
RATE_CASES = [
    *FIT_CASES,
    "probes/rate-empty-arguments",
    "probes/fin-argument-modes",
    "probes/rate-zero-residual-guess",
]
YIELD_CASES = [
    "probes/solver-iteration-traces",
    "probes/bond-one-period-bases",
    "probes/yield-solver-sweep",
    "probes/yield-start-sweep",
    "probes/yield-start-one-input",
]


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


def caret_pow(base: float, exponent: float) -> float:
    """base ^ exponent with Excel's `^` (`power.excel_model`); a power that is not real is
    NoResultError, as in `rpow`."""
    if base <= 0 and not exponent.is_integer():
        msg = f"{base} ^ {exponent} is not real"
        raise NoResultError(msg)
    return excel_model(Fraction(base), exponent)


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


# RATE's secant (T-2005, financial.md "RATE is a secant on the FV residual"), ported from the scratch
# harness to the lab (T-2527): `compute_rate` (functions/financial/financial_util.rs) and its variants.

Op = Callable[[float, float], float]
Residual = Callable[[float, Annuity], float]
RATE_MAX_STEPS = 200  # Excel's cap (probes/rate-step-cap); the engine's RATE_MAX_STEPS
RATE_CAP_TOLERANCE = 1e-4  # the cap's looser return test (T-2689)


def _plain_add(a: float, b: float) -> float:
    return a + b


def _plain_mul(a: float, b: float) -> float:
    return a * b


def _plain_div(a: float, b: float) -> float:
    return a / b


PLAIN: tuple[Op, Op, Op] = (_plain_add, _plain_mul, _plain_div)
TWICE: tuple[Op, Op, Op] = (X87.add, X87.mul, X87.div)


def fv_residual(ops: tuple[Op, Op, Op] = PLAIN, order: str = "pv,pmt,fv") -> Residual:
    """f(r) = pv·P + pmt·(k·((P-1)/r)) + fv, P = Excel's `^` of Excel's 1+r, k = 1+r for type 1: FV's
    evaluation (the T-2005 model with `PLAIN` and the default order; f(0) = pv + pmt·n + fv). `ops`
    rounds the residual's own `+`, `*` and `/` (`TWICE`: Excel's operators); `order` is the order of
    the three-term sum (`pv,pmt,fv` is ((pv·P + pmt·A) + fv))."""
    add, mul, div = ops

    def f(r: float, annuity: Annuity) -> float:
        n, pmt, pv, fv, t = annuity
        if r == 0:
            return pv + pmt * n + fv
        p = caret_pow(X87.add(1.0, r), n)
        k = add(1.0, r) if t != 0 else 1.0
        terms = {"pv": mul(pv, p), "pmt": mul(pmt, mul(k, div(add(p, -1.0), r))), "fv": fv}
        a, b, c = (terms[name] for name in order.split(","))
        total = add(add(a, b), c)
        if not math.isfinite(total):
            raise NoResultError("residual is not finite")
        return total

    return f


def rate_secant_iterates(residual: Residual, *args: Arg) -> list[float]:
    """The iterates r2 of RATE's secant on `residual` (the T-2005 model with `fv_residual()`), ending
    with the returned rate; NoResultError (`#NUM!`) or ExcelError `#VALUE!` (a guess at or below -1)
    where the model has no result."""
    full = [numeric_arg(x) for x in args] + [0.0, 0.0, 0.1][len(args) - 3 :]
    a: Annuity = (full[0], full[1], full[2], full[3], full[4])
    guess = full[5]
    if guess <= -1:
        raise ExcelError("guess at or below -1", "#VALUE!")
    r0, f0 = guess, residual(guess, a)
    r1 = r0 - math.copysign(0.001, f0)
    f1 = residual(r1, a)
    out: list[float] = []
    for step in range(1, RATE_MAX_STEPS + 1):
        if f1 == f0:
            raise NoResultError("flat secant")
        r2 = X87.add(r1, -X87.div(X87.mul(f1, X87.add(r1, -r0)), X87.add(f1, -f0)))
        if not math.isfinite(r2) or r2 <= -1:
            raise NoResultError("iterate is not a rate")
        f2 = residual(r2, a)
        out.append(r2)
        size = abs(r2 - r1)
        if size < 1e-7 and abs(f2) < 1e-7:
            return out
        at_cap = step == RATE_MAX_STEPS and size < RATE_CAP_TOLERANCE
        if at_cap and abs(f2) < RATE_CAP_TOLERANCE and 1 + r2 > RATE_CAP_TOLERANCE:
            return out
        r0, f0, r1, f1 = r1, f1, r2, f2
    raise NoResultError("no convergence in 200 steps")


def rate_secant(residual: Residual) -> Callable[..., float]:
    """RATE's value under `rate_secant_iterates`."""
    return lambda *args: rate_secant_iterates(residual, *args)[-1]


def rate_secant_candidates() -> list[Candidate]:
    """The T-2005 model (the engine's `compute_rate`) and residual variants, traced so the score
    reports which iterate equals Excel's value (T-2527)."""
    out = []
    for ops_name, ops in (("plain", PLAIN), ("Excel operators", TWICE)):
        for order in ("pv,pmt,fv", "pv,fv,pmt", "pmt,fv,pv", "fv,pv,pmt"):
            f = fv_residual(ops, order)
            out.append(
                candidate(
                    f"secant on FV residual, {ops_name}, sum {order}",
                    rate_secant(f),
                    "secant on FV residual",
                    partial(rate_secant_iterates, f),
                )
            )
    return out


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


# The power `price` and `dprice` discount with: libm, or Excel's `^` under yield_caret.caret_priced (T-2207).
PRICE_POWER: ContextVar[Callable[[float, float], float]] = ContextVar("price_power", default=rpow)


def price(y: float, bond: Bond) -> float:
    """PRICE with N > 1 coupons left (documented form, DSC = E - A)."""
    rate, red, freq, a, e, dsc, n = bond
    base, power = 1 + y / freq, PRICE_POWER.get()
    coupons = sum(100 * rate / freq / power(base, k - 1 + dsc / e) for k in range(1, n + 1))
    return red / power(base, n - 1 + dsc / e) + coupons - 100 * rate / freq * a / e


def dprice(y: float, bond: Bond) -> float:
    """d/dy of `price`, analytic."""
    rate, red, freq, _, e, dsc, n = bond
    base, power = 1 + y / freq, PRICE_POWER.get()
    total = 0.0
    for k in range(1, n + 1):
        t = k - 1 + dsc / e
        total -= t * 100 * rate / freq / power(base, t + 1) / freq
    t = n - 1 + dsc / e
    return total - t * red / power(base, t + 1) / freq


def _bond_price(args: tuple[Arg, ...]) -> tuple[Callable[[float], float], Bond, float]:
    """The cell's PRICE(y) function in its day-count basis, its bond tuple and the target price."""
    settle, maturity, rate, pr, red, freq, basis = _bond(args)
    a, e, _, dsc, n = day_counts(settle, maturity, freq, basis, period_e=False)
    bond: Bond = (rate, red, freq, a, e, dsc, n)
    return partial(price, bond=bond), bond, pr


def _newton_yield(start: float, tol: float) -> Callable[..., float]:
    """Newton on PRICE(y) - pr from `start`, returning the iterate after a step below `tol`."""
    return _newton_yield_from(lambda *_args: start, tol)


def _newton_yield_from(start: Callable[..., float], tol: float) -> Callable[..., float]:
    """`_newton_yield` from `start(*cell_args)` instead of a fixed start."""

    def call(*args: Arg) -> float:
        price_of, bond, pr = _bond_price(args)
        y = start(*args)
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


def _approximate_yield(bond: Bond, pr: float) -> float:
    """The textbook approximate yield (C + (red - P)/T) / ((red + P)/2), T = coupons left / f."""
    rate, red, freq, _, _, _, n = bond
    years = n / freq
    return (100 * rate + (red - pr) / years) / ((red + pr) / 2)


YIELD_STARTS: dict[str, Callable[[Bond, float], float]] = {
    "0": lambda _bond, _pr: 0.0,
    "the coupon rate": lambda bond, _pr: bond[0],
    "the current yield (rate * 100 / pr)": lambda bond, pr: bond[0] * 100 / pr,
    "the approximate yield (T-1592)": _approximate_yield,
}


def _years_to_maturity(settle: datetime.date, maturity: datetime.date, basis: int) -> float:
    """Settlement to maturity in years: the basis' day count over 360 (bases 0, 2, 4) or 365
    (bases 1, 3). T-1592: the hidden YIELD start moves between bases 0 and 1 as this count does
    (1559/360 against 1581/365 on yield-start-sweep D), not as /366 or /365.25 would."""
    days = days360(settle, maturity, basis == 4) if basis in (0, 4) else (maturity - settle).days
    return days / (365.0 if basis in (1, 3) else 360.0)


def _annuity_model_root(m: int) -> Callable[..., float]:
    """The yield of a frequency-free bond with redemption 100: an annual coupon of 100 * rate paid
    m times a year over the continuous term T (`_years_to_maturity`), m = 0 compounding
    continuously, solved by `regula_falsi`. A T-1592 start candidate: the hidden start does not move
    with redemption or frequency."""

    def call(*args: Arg) -> float:
        settle, maturity, rate, pr, _, _, basis = _bond(args)
        years = _years_to_maturity(settle, maturity, basis)

        def price_of(y: float) -> float:
            v = math.exp(-y * years) if m == 0 else rpow(1 + y / m, -m * years)
            coupons = 100 * rate * years if y == 0 else 100 * rate * (1 - v) / y
            return coupons + 100 * v

        return regula_falsi(price_of, pr)

    return call


def _semiannual_par_root(*args: Arg) -> float:
    """PRICE's root at frequency 2 and redemption 100 whatever the cell's own: a T-1592 start
    candidate."""
    basis = args[6] if len(args) > 6 else 0.0
    price_of, _, pr = _bond_price((*args[:4], 100.0, 2.0, basis))
    return regula_falsi(price_of, pr)


def newton_back_chain(cell_args: tuple[Arg, ...], y: float, steps: int) -> list[float]:
    """Excel's YIELD value `y` and up to `steps` earlier analytic-Newton iterates on PRICE(y) - pr:
    each x < y with x - (PRICE(x) - pr)/PRICE'(x) = y, found by bisection on [y - 0.5, y]
    (T-1592). The bisection assumes the Newton map increases on that bracket, which holds near the
    root because PRICE is convex and decreasing in the yield; a step stops early when the bracket's low
    end already maps above `y`. Precision falls with each step back (Newton's error squares going
    forward)."""
    price_of, bond, pr = _bond_price(cell_args)

    def newton(x: float) -> float:
        return x - (price_of(x) - pr) / dprice(x, bond)

    chain = [y]
    for _ in range(steps):
        lo, hi = y - 0.5, y
        if newton(lo) > y:
            break
        for _ in range(200):
            mid = (lo + hi) / 2
            if mid in (lo, hi):
                break
            if newton(mid) < y:
                lo = mid
            else:
                hi = mid
        y = lo
        chain.append(y)
    return chain


def start_years(settle: datetime.date, maturity: datetime.date, basis: int) -> float:
    """The term the fitted YIELD start takes (T-2777, probes/yield-start-term-basis): whole years from
    `settle` (a 29 February settlement has its anniversaries on 1 March in common years), then the
    basis' days from the last anniversary to `maturity` over 360 (bases 0, 2, 4), 365 (basis 3) or
    the length of the maturity's calendar year (basis 1). The engine's `yield_start_years`."""

    def anniversary(year: int) -> datetime.date:
        if settle.month == 2 and settle.day == 29 and not calendar.isleap(year):
            return datetime.date(year, 3, 1)
        return settle.replace(year=year)

    whole, last = maturity.year - settle.year, anniversary(maturity.year)
    if last > maturity:
        whole, last = whole - 1, anniversary(maturity.year - 1)
    days = days360(last, maturity, basis == 4) if basis in (0, 4) else (maturity - last).days
    year = {1: 366.0 if calendar.isleap(maturity.year) else 365.0, 3: 365.0}.get(basis, 360.0)
    return whole + days / year


def fitted_yield_start(*args: Arg) -> float:
    """T-1592's fitted start: (100 rate + (100 - pr)/T) / ((100 + pr)/2 - (100 - pr)/(4T)), T the
    `start_years` (T-2777; T-1592 fitted it with `_years_to_maturity`, the same on its basis-0 probes
    but for a 29 February settlement). On yield-start-one-input the two-steps-back starts give a
    denominator pr + w (100 - pr) with (1/2 - w) T = 0.25000 on every row below PRICE's root."""
    settle, maturity, rate, pr, _, _, basis = _bond(args)
    years = start_years(settle, maturity, basis)
    return (100 * rate + (100 - pr) / years) / ((100 + pr) / 2 - (100 - pr) / (4 * years))


def _newton_yield_short(start: Callable[..., float], tol: float) -> Callable[..., float]:
    """Analytic Newton from `start(*cell_args)` that returns the iterate whose next step is below
    `tol`, without taking that step (T-1592: Excel's values stop short of PRICE's root)."""

    def call(*args: Arg) -> float:
        price_of, bond, pr = _bond_price(args)
        y = start(*args)
        for _ in range(100):
            step = (price_of(y) - pr) / dprice(y, bond)
            if abs(step) < tol:
                return y
            y -= step
        raise NoResultError("Newton did not converge")

    return call


def _fd_newton_yield(
    start: float, h: float, relative: bool, central: bool, tol: float
) -> Callable[..., float]:
    """Newton on PRICE(y) - pr from `start` with a finite-difference derivative, its step `h`
    absolute or `h * |y|`, forward or central, returning the iterate after a step below `tol`
    (T-1592: a numerical derivative would make the analytic back-chain's shared start an artefact)."""

    def call(*args: Arg) -> float:
        price_of, _, pr = _bond_price(args)
        y = start
        for _ in range(100):
            d = h * abs(y) if relative and y != 0 else h
            low = price_of(y - d) if central else price_of(y)
            slope = (price_of(y + d) - low) / (2 * d if central else d)
            step = (price_of(y) - pr) / slope
            y -= step
            if abs(step) < tol:
                return y
        raise NoResultError("Newton did not converge")

    return call


FD_NEWTON_YIELDS = [
    # (start, h, relative, central, tol): the best of a T-1592 scratch grid of 560 (starts 0, 0.1, the coupon
    # rate, the current yield; h 1e-8 to 1e-2 and -1e-4, -1e-3; step and residual stops), none above 5/178.
    (0.0, 1e-5, False, False, 1e-10),
    (0.0, 1e-7, False, False, 1e-10),
    (0.0, 1e-6, True, False, 1e-10),
    (0.0, 1e-8, False, True, 1e-10),
    (0.1, -1e-4, True, False, 1e-9),
    (0.1, 1e-4, False, False, 1e-9),
]


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
    # T-1592: Newton from a frequency- and redemption-free start, with T-2187's step cut.
    model_starts = {f"a {m}-coupon-a-year annuity root": _annuity_model_root(m) for m in (1, 2, 3, 4)}
    model_starts["a continuous annuity root"] = _annuity_model_root(0)
    model_starts["PRICE's root at frequency 2, redemption 100"] = _semiannual_par_root
    out += [
        candidate(
            f"Newton from {name}, |step| < 6.9e-6", _newton_yield_from(start, 6.9e-6), "Newton, model start"
        )
        for name, start in model_starts.items()
    ]
    # T-1592 (fourth round): the fitted start, and the step not taken once it is below the cut.
    out += [
        candidate(
            f"Newton from the fitted start, returned before a step below {t:g}",
            _newton_yield_short(fitted_yield_start, t),
            "Newton, fitted start",
        )
        for t in (1e-9, 1e-10, 1e-11)
    ]
    out += [
        candidate(
            f"Newton from {y0:g}, {'central' if c else 'forward'} difference {h:g}{' * |y|' if rel else ''}, "
            f"|step| < {t:g}",
            _fd_newton_yield(y0, h, rel, c, t),
            "Newton, finite difference",
        )
        for y0, h, rel, c, t in FD_NEWTON_YIELDS
    ]
    return out


FITS = [
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
        description="RATE cells of the fit set, nested PMT at Excel's bits: the T-2005 secant, its variants, Newton",
        cases=RATE_CASES,
        pattern=r"^=RATE\((.*)\)$",
        candidates=lambda: rate_secant_candidates() + rate_candidates(),
        where=_literal_rate,
        errors=True,
        calls=["probes-numeric/m9-fin-pmt"],
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
