"""GAMMA and GAMMALN at integers and half-integers (the T-1042 sweeps): the x87 test (T-1524).

Where the exact value is known in closed form, so a correctly rounded reference exists:

- `gamma-int`: GAMMA(n) = (n-1)!, n = 1..171. Candidates: the factorial correctly rounded, the
  product 2*3*...*(n-1) taken upward or downward with each product rounded to a double, twice
  rounded (Excel's `*`) or kept at 64 bits (the x87-register hypothesis), and libm `tgamma`.
- `gammaln-int`: GAMMALN(n) = ln((n-1)!). The same products, then a logarithm (libm, or ln at
  64 bits), plus libm `lgamma`.
- `gamma-half-int`: GAMMA(m + 1/2) = sqrt(pi) * (1/2)(3/2)...(m - 1/2) for m >= 0, and
  the recurrence divided down for negative m. The same rounding models, with sqrt(pi) a double
  or kept at 64 bits.

GAMMA and GAMMALN at other arguments need a high-precision gamma, which the lab does not have
(`exact` has no gamma), so they are not scored here. Results:
docs/excel-semantics/statistics.md, "GAMMA and GAMMALN sweep (T-1042)", and T-1524's Outcome.
"""

from __future__ import annotations

import math
from collections.abc import Callable
from fractions import Fraction

from rustsheet_tools.fitlab.exact import (
    EXTENDED,
    Rounding,
    extended_value,
    ln_cr,
    pi_cr,
    round_to_bits,
    sqrt_cr,
    to_double,
    to_double_value,
    x87_value,
)
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg

INT_CASES = ["probes-numeric/t1042-gamma-int-1-90", "probes-numeric/t1042-gamma-int-91-171"]
HALF_CASE = "probes-numeric/t1042-gamma-half-int"
_SQRT_PI_BITS = 200
ROUNDINGS: dict[str, Rounding] = {
    "double": to_double_value,
    "x87 op": x87_value,
    "ext": extended_value,
}


def _arg(x: Arg) -> Fraction:
    if isinstance(x, bool) or not isinstance(x, float):
        msg = f"non-numeric argument {x!r}"
        raise TypeError(msg)
    return Fraction(x)


def _positive_int(x: Fraction) -> int:
    if x.denominator != 1 or x < 1:
        msg = f"{float(x)} is not a positive integer"
        raise ValueError(msg)
    return int(x)


def factorial_product(n: int, rnd: Rounding, downward: bool) -> Fraction:
    """(n-1)! as 2*3*...*(n-1) (upward) or (n-1)*(n-2)*...*2 (downward), each product rounded."""
    factors = range(n - 1, 1, -1) if downward else range(2, n)
    p = Fraction(1)
    for k in factors:
        p = rnd(p * k)
    return p


def _gamma_int(rnd: Rounding, downward: bool) -> Callable[..., float]:
    return lambda x: to_double(factorial_product(_positive_int(_arg(x)), rnd, downward))


def _gammaln_int(rnd: Rounding, downward: bool, ln: Callable[[Fraction], float]) -> Callable[..., float]:
    def evaluate(x: Arg) -> float:
        n = _positive_int(_arg(x))
        return 0.0 if n <= 2 else ln(factorial_product(n, rnd, downward))

    return evaluate


def _libm_log(v: Fraction) -> float:
    return math.log(to_double(v))


def _ln64(v: Fraction) -> float:
    return to_double(round_to_bits(ln_cr(v, EXTENDED), EXTENDED))


def _exact_ln(v: Fraction) -> float:
    return to_double(ln_cr(v))


def gamma_int_candidates() -> list[Candidate]:
    out = [
        candidate("correctly rounded", lambda x: to_double(math.factorial(_positive_int(_arg(x)) - 1))),
        candidate("libm tgamma", lambda x: math.gamma(float(_arg(x)))),
    ]
    for name, rnd in ROUNDINGS.items():
        for downward in (False, True):
            label = f"{name}: product {'downward' if downward else 'upward'}"
            out.append(candidate(label, _gamma_int(rnd, downward), name))
    return out


def gammaln_int_candidates() -> list[Candidate]:
    out = [
        candidate("correctly rounded", _gammaln_int(lambda v: v, True, _exact_ln)),
        candidate("libm lgamma", lambda x: math.lgamma(float(_arg(x)))),
    ]
    for name, rnd in ROUNDINGS.items():
        for downward in (False, True):
            for ln_name, ln in (("libm log", _libm_log), ("ln64", _ln64)):
                label = f"{name}: {ln_name} of product {'downward' if downward else 'upward'}"
                out.append(candidate(label, _gammaln_int(rnd, downward, ln), name))
    return out


def half_int_gamma(x: Fraction, sqrt_pi: Fraction, rnd: Rounding, downward: bool) -> Fraction:
    """GAMMA(x) for x = m + 1/2 by the recurrence from GAMMA(1/2) = sqrt(pi). For m >= 0 the
    factors (1/2)...(x-1) multiply sqrt(pi) one by one (upward), or form their own product from
    the top before one multiplication by sqrt(pi) (downward); for m < 0 sqrt(pi) is divided by
    -1/2, -3/2, ..., x."""
    if (2 * x).denominator != 1 or x.denominator != 2:
        msg = f"{float(x)} is not a half-integer"
        raise ValueError(msg)
    if x < 0:
        g, y = sqrt_pi, Fraction(-1, 2)
        while y >= x:
            g = rnd(g / y)
            y -= 1
        return g
    if not downward:
        g, y = sqrt_pi, Fraction(1, 2)
        while y < x:
            g = rnd(g * y)
            y += 1
        return g
    p, y = Fraction(1), x - 1
    while y > 0:
        p = rnd(p * y)
        y -= 1
    return rnd(p * sqrt_pi)


def gamma_half_candidates() -> list[Candidate]:
    exact_sqrt_pi = sqrt_cr(pi_cr(_SQRT_PI_BITS), _SQRT_PI_BITS)
    double_sqrt_pi = Fraction(math.sqrt(math.pi))

    def exact(x: Arg) -> float:
        return to_double(half_int_gamma(_arg(x), exact_sqrt_pi, lambda v: v, downward=False))

    out = [
        candidate("correctly rounded", exact),
        candidate("libm tgamma", lambda x: math.gamma(float(_arg(x)))),
    ]
    for name, rnd in ROUNDINGS.items():
        for root_name, root in (
            ("sqrt(pi) double", double_sqrt_pi),
            ("sqrt(pi) 64-bit", round_to_bits(exact_sqrt_pi, EXTENDED)),
        ):
            for downward in (False, True):
                label = f"{name}: {root_name}, {'downward' if downward else 'upward'}"

                def evaluate(
                    x: Arg, root: Fraction = root, rnd: Rounding = rnd, downward: bool = downward
                ) -> float:
                    return to_double(half_int_gamma(_arg(x), root, rnd, downward))

                out.append(candidate(label, evaluate, name))
    return out


FITS = [
    Fit(
        name="gamma-int",
        description="GAMMA(n), n = 1..171 (T-1042): factorial products per rounding model",
        cases=INT_CASES,
        pattern=r"^=GAMMA\((.*)\)$",
        candidates=gamma_int_candidates,
    ),
    Fit(
        name="gammaln-int",
        description="GAMMALN(n), n = 1..171 (T-1042): ln of the factorial products per rounding model",
        cases=INT_CASES,
        pattern=r"^=GAMMALN\((.*)\)$",
        candidates=gammaln_int_candidates,
    ),
    Fit(
        name="gamma-half-int",
        description="GAMMA(m+1/2), m = -11..50 (T-1042): the sqrt(pi) recurrence per rounding model",
        cases=[HALF_CASE],
        pattern=r"^=GAMMA\((.*)\)$",
        candidates=gamma_half_candidates,
    ),
]
