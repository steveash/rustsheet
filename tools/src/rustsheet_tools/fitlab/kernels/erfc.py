"""ERFC: the fit lab's worked example (T-1523), and the home of the T-1187 candidates.

Two fits over the T-1040 sweeps (probes-numeric/t1040-erfc and t1040-erfc-precise, excel-graph):

- `erfc`: every numeric cell of both sweeps (ERFC and ERFC.PRECISE, x in -6..27), scoring the
  engine against libm `math.erfc` and `1 - math.erf`.
- `erfc-tail`: the T-1187 grid, ERFC(x) for x >= 1 where Excel is not 0 (205 cells), scoring the
  continued-fraction, Lentz and Numerical Recipes formulations of the core factor
  K = sqrt(pi) erfc(x) exp(x^2) that docs/research/t1187-erfc-candidates.py tried (moved here;
  results in docs/excel-semantics/statistics.md "ERFC core factor"). Each is plain IEEE double
  (Python floats; `math.exp` is the platform libm, glibc on the agent containers).
- `erfc-x87`: the same grid's best fraction (`bu_even`, the two best combinations) evaluated
  with every operation at 64 bits and one final rounding to a double (the x87 hypothesis,
  T-1524), with exp at 64 bits idealized or as the x87 instruction sequence, against the same
  fraction in doubles and a 128-bit evaluation that stands in for the correctly rounded erfc.
"""

from __future__ import annotations

import math
import sys
from collections.abc import Callable
from fractions import Fraction

from rustsheet_tools.fitlab.exact import (
    EXTENDED,
    Rounding,
    X87Insn,
    exp_cr,
    pi_cr,
    round_to_bits,
    sqrt_cr,
    to_double,
)
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Cell

SQPI = math.sqrt(math.pi)
ISQPI = 1 / SQPI
EPS = sys.float_info.epsilon
FPMIN = sys.float_info.min / EPS
TINY = 1e-300
DEPTHS = [*range(5, 80), 100, 150, 200, 300, 500, 1000]
TOLERANCES = [EPS, 2 * EPS, 3e-16, 5e-16, 1e-16, 1e-15, 1e-14]
CASES = ["probes-numeric/t1040-erfc"]
PATTERN = r"^=ERFC(?:\.PRECISE)?\((.*)\)$"


# Continued fractions for K = sqrt(pi) * erfc(x) * exp(x^2), evaluated bottom-up to depth n.
def bu_half(x: float, n: int) -> float:
    t = x
    for k in range(n, 0, -1):
        t = x + (k / 2) / t
    return 1 / t


def bu_half_mul(x: float, n: int) -> float:
    t = x
    for k in range(n, 0, -1):
        t = x + k * 0.5 / t
    return 1 / t


def bu_half_div(x: float, n: int) -> float:
    t = x
    for k in range(n, 0, -1):
        t = x + k / (2 * t)
    return 1 / t


def bu_even(x: float, n: int) -> float:
    z = 2 * x * x
    t = z + 1 + 4 * n
    for k in range(n, 0, -1):
        t = z + 1 + 4 * (k - 1) - (2 * k - 1) * (2 * k) / t
    return 2 * x / t


def bu_laplace(x: float, n: int) -> float:
    z = x * x
    t = z + 0.5 + 2 * n
    for k in range(n, 0, -1):
        t = z + 0.5 + 2 * (k - 1) - (2 * k - 1) * k / 2 / t
    return x / t


def _lentz(start: float, a: Callable[[int], float], b: Callable[[int], float], eps: float) -> float:
    """Modified Lentz, top-down: start + a(1)/(b(1) + a(2)/(b(2) + ...)) to tolerance `eps`."""
    f = c = start
    d = 0.0
    for k in range(1, 5000):
        d = b(k) + a(k) * d
        d = 1 / (d or TINY)
        c = b(k) + a(k) / c
        c = c or TINY
        delta = c * d
        f *= delta
        if abs(delta - 1) < eps:
            break
    return f


# The same fractions top-down (modified Lentz) to a tolerance.
def lentz_half(x: float, eps: float) -> float:
    return 1 / _lentz(x, lambda k: k / 2, lambda _: x, eps)


def lentz_even(x: float, eps: float) -> float:
    z = 2 * x * x
    return 2 * x / _lentz(z + 1, lambda k: -(2 * k - 1) * (2 * k), lambda k: z + 1 + 4 * k, eps)


Combine = Callable[[float, float], float]
# e = exp(-x*x), k = the fraction; orders of applying 1/sqrt(pi).
MUL_FORMS: dict[str, Combine] = {
    "e*k/SQPI": lambda e, k: e * k / SQPI,
    "e*k*ISQPI": lambda e, k: e * k * ISQPI,
    "e*(k*ISQPI)": lambda e, k: e * (k * ISQPI),
    "e*(k/SQPI)": lambda e, k: e * (k / SQPI),
    "e/SQPI*k": lambda e, k: e / SQPI * k,
    "e*ISQPI*k": lambda e, k: e * ISQPI * k,
    "k/SQPI*e": lambda e, k: k / SQPI * e,
}
# big = exp(x*x), divided out instead.
DIV_FORMS: dict[str, Combine] = {
    "k/big/SQPI": lambda big, k: k / big / SQPI,
    "k/(big*SQPI)": lambda big, k: k / (big * SQPI),
    "k*ISQPI/big": lambda big, k: k * ISQPI / big,
    "k/SQPI/big": lambda big, k: k / SQPI / big,
    "(1/big)*k/SQPI": lambda big, k: (1 / big) * k / SQPI,
    "(1/big)*(k*ISQPI)": lambda big, k: (1 / big) * (k * ISQPI),
    "k/(SQPI*big)": lambda big, k: k / (SQPI * big),
}


# Numerical Recipes: erfc(x) = Q(1/2, x^2), continued fraction (gcf) or series (gser).
def gammln_nr2(xx: float) -> float:
    cof = [
        76.18009172947146,
        -86.50532032941677,
        24.01409824083091,
        -1.231739572450155,
        0.1208650973866179e-2,
        -0.5395239384953e-5,
    ]
    y = xx
    tmp = xx + 5.5
    tmp -= (xx + 0.5) * math.log(tmp)
    series = 1.000000000190015
    for c in cof:
        y += 1
        series += c / y
    return -tmp + math.log(2.5066282746310005 * series / xx)


def nr_erfc(x: float, gln: float, eps: float) -> float:
    a, xx = 0.5, x * x
    prefactor = math.exp(-xx + a * math.log(xx) - gln)
    if xx < a + 1:
        ap, total = a, 1 / a
        term = total
        while abs(term) >= abs(total) * eps:
            ap += 1
            term *= xx / ap
            total += term
        return 1 - total * prefactor
    b = xx + 1 - a
    c, d = 1 / FPMIN, 1 / b
    h = d
    for i in range(1, 1000):
        an = -i * (i - a)
        b += 2
        d = an * d + b
        d = d if abs(d) >= FPMIN else FPMIN
        c = b + an / c
        c = c if abs(c) >= FPMIN else FPMIN
        d = 1 / d
        delta = d * c
        h *= delta
        if abs(delta - 1) < eps:
            break
    return prefactor * h


def _x(fn: Callable[[float], float]) -> Callable[..., float]:
    """ERFC's one argument as a float (the sweep's literals parse as floats)."""

    def call(x: float | bool | str | list[float | bool | str | None] | None) -> float:
        if not isinstance(x, float):
            msg = f"ERFC argument {x!r}"
            raise TypeError(msg)
        return fn(x)

    return call


def libm_candidates() -> list[Candidate]:
    return [
        candidate("math.erfc", _x(math.erfc), "libm"),
        candidate("1 - math.erf", _x(lambda x: 1 - math.erf(x)), "libm"),
    ]


def _bottom_up(
    cf: Callable[[float, int], float], n: int, g: Combine, sign: float
) -> Callable[[float], float]:
    return lambda x: g(math.exp(sign * x * x), cf(x, n))


def _top_down(cf: Callable[[float, float], float], eps: float, g: Combine) -> Callable[[float], float]:
    return lambda x: g(math.exp(-x * x), cf(x, eps))


def _nr(gln: float, eps: float) -> Callable[[float], float]:
    return lambda x: nr_erfc(x, gln, eps)


def t1187_candidates() -> list[Candidate]:
    """Every T-1187 configuration (about 5,800), grouped in its four families. `sign` -1 is
    exp(-x*x) (as `-x * x`, exactly), +1 is exp(x*x)."""
    out = libm_candidates()
    bottom_up = [bu_half, bu_half_mul, bu_half_div, bu_even, bu_laplace]
    for family, forms, sign in (
        ("bottom-up, * exp(-x*x)", MUL_FORMS, -1.0),
        ("bottom-up, / exp(x*x)", DIV_FORMS, 1.0),
    ):
        out += [
            candidate(f"{cf.__name__} n={n} {label}", _x(_bottom_up(cf, n, g, sign)), family)
            for cf in bottom_up
            for n in DEPTHS
            for label, g in forms.items()
        ]
    out += [
        candidate(f"{cf.__name__} eps={t:.3g} {label}", _x(_top_down(cf, t, g)), "Lentz, * exp(-x*x)")
        for cf in (lentz_half, lentz_even)
        for t in TOLERANCES
        for label, g in MUL_FORMS.items()
    ]
    glns = {"exact": 0.5723649429247001, "NR2": gammln_nr2(0.5), "lgamma": math.lgamma(0.5)}
    out += [
        candidate(f"nr_erfc gln={name} eps={t:.3g}", _x(_nr(gl, t)), "NR incomplete gamma")
        for name, gl in glns.items()
        for t in [*TOLERANCES, 3e-7]
    ]
    return out


X87_DEPTHS = [30, 60, 100]
_REFERENCE_BITS = 128
_REFERENCE_DEPTH = 1500  # converged to well beyond 128 bits for x >= 1


def bu_even_rounded(x: Fraction, n: int, rnd: Rounding) -> Fraction:
    """`bu_even` with every operation rounded by `rnd` (exact for `lambda v: v`)."""
    z = rnd(2 * rnd(x * x))
    t = rnd(z + 1 + 4 * n)
    for k in range(n, 0, -1):
        t = rnd(rnd(z + (1 + 4 * (k - 1))) - rnd((2 * k - 1) * (2 * k) / t))
    return rnd(2 * x / t)


def _at(bits: int) -> Rounding:
    return lambda v: round_to_bits(v, bits)


def _erfc_rounded(x: float, n: int, bits: int, exp: Callable[[Fraction], Fraction], divide: bool) -> float:
    """sqrt(pi)^-1 * K * exp(-x^2) at `bits`: as (1/exp(x*x)) * (K * ISQPI) (`divide`) or as
    exp(-x*x) * (K * ISQPI), the two best T-1187 combinations."""
    rnd = _at(bits)
    v = Fraction(x)
    isqpi = rnd(1 / sqrt_cr(pi_cr(bits + 8), bits + 8))
    k = bu_even_rounded(v, n, rnd)
    square = rnd(v * v)
    factor = rnd(1 / rnd(exp(square))) if divide else rnd(exp(-square))
    return to_double(rnd(factor * rnd(k * isqpi)))


def _exp64(v: Fraction) -> Fraction:
    return exp_cr(v, EXTENDED)


def _exp_reference(v: Fraction) -> Fraction:
    return exp_cr(v, _REFERENCE_BITS)


def x87_candidates() -> list[Candidate]:
    out = [
        candidate(
            f"128-bit bu_even n={_REFERENCE_DEPTH} (reference)",
            _x(lambda x: _erfc_rounded(x, _REFERENCE_DEPTH, _REFERENCE_BITS, _exp_reference, divide=False)),
            "reference",
        )
    ]
    for n in X87_DEPTHS:
        for divide, label in ((True, "(1/big)*(k*ISQPI)"), (False, "e*(k*ISQPI)")):
            sign = 1.0 if divide else -1.0
            form = DIV_FORMS[label] if divide else MUL_FORMS[label]
            out.append(
                candidate(f"double bu_even n={n} {label}", _x(_bottom_up(bu_even, n, form, sign)), "double")
            )
            for exp_name, exp in (("exp64", _exp64), ("x87 insn exp", X87Insn.exp)):

                def evaluate(
                    x: float, n: int = n, exp: Callable[[Fraction], Fraction] = exp, divide: bool = divide
                ) -> float:
                    return _erfc_rounded(x, n, EXTENDED, exp, divide)

                out.append(candidate(f"ext bu_even n={n} {label}, {exp_name}", _x(evaluate), "ext"))
    return out


def _tail(cell: Cell) -> bool:
    """The T-1187 grid: x >= 1 and Excel's value is not 0."""
    x = cell.args[0] if cell.args else None
    return isinstance(x, float) and x >= 1 and cell.expected != 0


FITS = [
    Fit(
        name="erfc",
        description="ERFC and ERFC.PRECISE over x in -6..27 (T-1040 sweeps): engine vs libm",
        cases=[*CASES, "probes-numeric/t1040-erfc-precise"],
        pattern=PATTERN,
        candidates=libm_candidates,
    ),
    Fit(
        name="erfc-tail",
        description="ERFC(x), x >= 1, Excel non-zero (the T-1187 grid): the T-1187 core-factor candidates",
        cases=CASES,
        pattern=PATTERN,
        candidates=t1187_candidates,
        where=_tail,
    ),
    Fit(
        name="erfc-x87",
        description="ERFC(x), x >= 1, Excel non-zero: the best T-1187 fraction in doubles, at 64 bits, and at 128 bits",
        cases=CASES,
        pattern=PATTERN,
        candidates=x87_candidates,
        where=_tail,
    ),
]
