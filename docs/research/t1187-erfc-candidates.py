"""T-1187: score ERFC core-factor candidates against Excel on probes-numeric/t1040-erfc.

Research record, not a tool: re-implements each candidate in IEEE double (Python floats; `math.exp`
is the platform libm, glibc on the agent containers) and counts the grid points with x >= 1 where
it equals Excel's cached value bit for bit. Run from the repo root:

    python3 docs/research/t1187-erfc-candidates.py

The results are recorded in docs/excel-semantics/statistics.md ("ERFC core factor: candidates
that do not reproduce Excel").
"""

import json
import math
import re
import sys
from collections.abc import Callable

CASES = "eval/suites/probes-numeric/cases/"
SQPI = math.sqrt(math.pi)
ISQPI = 1 / SQPI
EPS = sys.float_info.epsilon
FPMIN = sys.float_info.min / EPS
TINY = 1e-300
DEPTHS = [*range(5, 80), 100, 150, 200, 300, 500, 1000]
TOLERANCES = [EPS, 2 * EPS, 3e-16, 5e-16, 1e-16, 1e-15, 1e-14]


def grid() -> list[tuple[float, float]]:
    """(x, Excel value) for every non-zero ERFC(x) with x >= 1."""
    with open(CASES + "t1040-erfc.case.json") as f:
        cells = json.load(f)["sheets"][0]["cells"]
    with open(CASES + "t1040-erfc.oracle.json") as f:
        oracle = json.load(f)["sheets"]["Calc"]
    out = []
    for cell, formula in cells.items():
        m = re.fullmatch(r"=ERFC\((-?[\d.]+)\)", formula)
        if m and cell in oracle and float(m.group(1)) >= 1 and oracle[cell]["v"] != 0:
            out.append((float(m.group(1)), oracle[cell]["v"]))
    return out


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


# The same fractions top-down (modified Lentz) to a tolerance.
def lentz_half(x: float, eps: float) -> float:
    f = c = x
    d = 0.0
    for k in range(1, 5000):
        a = k / 2
        d = x + a * d
        d = 1 / (d or TINY)
        c = x + a / c
        c = c or TINY
        delta = c * d
        f *= delta
        if abs(delta - 1) < eps:
            break
    return 1 / f


def lentz_even(x: float, eps: float) -> float:
    z = 2 * x * x
    f = c = z + 1
    d = 0.0
    for k in range(1, 5000):
        a = -(2 * k - 1) * (2 * k)
        b = z + 1 + 4 * k
        d = b + a * d
        d = 1 / (d or TINY)
        c = b + a / c
        c = c or TINY
        delta = c * d
        f *= delta
        if abs(delta - 1) < eps:
            break
    return 2 * x / f


Combine = Callable[[float, float], float]
# e = exp(-x*x), k = the fraction; orders of applying 1/sqrt(pi).
MUL_FORMS: list[Combine] = [
    lambda e, k: e * k / SQPI,
    lambda e, k: e * k * ISQPI,
    lambda e, k: e * (k * ISQPI),
    lambda e, k: e * (k / SQPI),
    lambda e, k: e / SQPI * k,
    lambda e, k: e * ISQPI * k,
    lambda e, k: k / SQPI * e,
]
# big = exp(x*x), divided out instead.
DIV_FORMS: list[Combine] = [
    lambda big, k: k / big / SQPI,
    lambda big, k: k / (big * SQPI),
    lambda big, k: k * ISQPI / big,
    lambda big, k: k / SQPI / big,
    lambda big, k: (1 / big) * k / SQPI,
    lambda big, k: (1 / big) * (k * ISQPI),
    lambda big, k: k / (SQPI * big),
]


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
    ser = 1.000000000190015
    for c in cof:
        y += 1
        ser += c / y
    return -tmp + math.log(2.5066282746310005 * ser / xx)


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


def main() -> None:
    pts = grid()
    union: set[float] = set()

    def family(name: str, candidates: list[Callable[[float], float]]) -> None:
        best = 0
        for f in candidates:
            hits = {x for x, v in pts if f(x) == v}
            best = max(best, len(hits))
            union.update(hits)
        print(f"{name:28} best {best:3}/{len(pts)}  union so far {len(union)}/{len(pts)}")

    bottom_up = [bu_half, bu_half_mul, bu_half_div, bu_even, bu_laplace]
    family(
        "bottom-up, * exp(-x*x)",
        [
            (lambda x, cf=cf, n=n, g=g: g(math.exp(-x * x), cf(x, n)))
            for cf in bottom_up
            for n in DEPTHS
            for g in MUL_FORMS
        ],
    )
    family(
        "bottom-up, / exp(x*x)",
        [
            (lambda x, cf=cf, n=n, g=g: g(math.exp(x * x), cf(x, n)))
            for cf in bottom_up
            for n in DEPTHS
            for g in DIV_FORMS
        ],
    )
    family(
        "Lentz, * exp(-x*x)",
        [
            (lambda x, cf=cf, t=t, g=g: g(math.exp(-x * x), cf(x, t)))
            for cf in (lentz_half, lentz_even)
            for t in TOLERANCES
            for g in MUL_FORMS
        ],
    )
    glns = [0.5723649429247001, gammln_nr2(0.5), math.lgamma(0.5)]
    family(
        "NR incomplete gamma",
        [(lambda x, gl=gl, t=t: nr_erfc(x, gl, t)) for gl in glns for t in [*TOLERANCES, 3e-7]],
    )


if __name__ == "__main__":
    main()
