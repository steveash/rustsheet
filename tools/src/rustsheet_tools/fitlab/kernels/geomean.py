"""GEOMEAN's in-range root models (T-3502, decision 0188 R4).

In range (a normal plain product P = m·2^E, more than one value) the engine took
`r = exp(ln(P)/n)`, snapped to its 15-digit value c when c, multiplied n times left to right, gives
P back (T-1123). T-3502 found that fit does not generalize and replaced it by the split root
`exp(ln(m·2^r)/n)·2^q`, E = q·n + r with q truncated toward zero (`split_root_trunc`). Scored
here: the split roots, the snaps, plain r, and the correctly rounded root of P and of the exact
product, on the in-range probes of T-1113/T-1123 whose arguments are literals (the SEQUENCE cells
are scored in T-3502's Outcome) and on the fresh log-uniform sweep
`probes-numeric/t3502-geomean-fresh` (R4.2 for the snap), and on `t3502-geomean-postfreeze`, generated
with a new seed after the split root was frozen (R4.2 for the split root). Results: numbers.md "GEOMEAN in-range root".
"""

from __future__ import annotations

import math

import mpmath

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg, Cell, numeric_arg

_OLD = [
    "probes/geomean-in-range-many",
    "probes/geomean-root-models",
    "probes/geomean-root-refinement",
    "probes/geomean-same-product",
    "probes/geomean-single-and-copies",
    "probes/geomean-stored-3ulps-low",
]
_FRESH = ["probes-numeric/t3502-geomean-fresh"]
# Generated with a new seed after the split root was frozen (R4.2 for the split root itself).
_POSTFREEZE = ["probes-numeric/t3502-geomean-postfreeze"]


def _values(args: tuple[Arg, ...]) -> list[float]:
    return [numeric_arg(a) for a in args]


def _product(values: list[float]) -> float:
    p = 1.0
    for v in values:
        p *= v
    return p


def plain_root(*args: Arg) -> float:
    """`exp(ln(P)/n)` on the left-to-right double product, as the engine without the snap."""
    values = _values(args)
    p = _product(values)
    if len(values) == 1:
        return p
    return math.exp(math.log(p) / len(values))


def snapped_root(*args: Arg) -> float:
    """T-1123's model: r, snapped to its 15-digit value when that value gives P back."""
    values = _values(args)
    p = _product(values)
    root = plain_root(*args)
    if len(values) == 1:
        return root
    short = float(f"{root:.14e}")
    back = 1.0
    for _ in values:
        back *= short
    return short if short != root and back == p else root


def correctly_rounded(*args: Arg) -> float:
    """The geometric mean of the double values, to 50 digits, rounded once to a double."""
    values = _values(args)
    with mpmath.workdps(50):
        logs = mpmath.fsum(mpmath.log(mpmath.mpf(v)) for v in values)
        return float(mpmath.exp(logs / len(values)))


def rounded_product_root(*args: Arg) -> float:
    """The n-th root of the left-to-right double product P, to 50 digits, rounded once."""
    values = _values(args)
    p = _product(values)
    with mpmath.workdps(50):
        return float(mpmath.root(mpmath.mpf(p), len(values)))


def exact_root_or_plain(*args: Arg) -> float:
    """The correctly rounded root of P when it multiplies back to P (left to right), else r."""
    values = _values(args)
    p = _product(values)
    root = plain_root(*args)
    if len(values) == 1:
        return root
    c = rounded_product_root(*args)
    back = 1.0
    for _ in values:
        back *= c
    return c if back == p else root


def split_root(*args: Arg) -> float:
    """P = m * 2^E (m in [1, 2)), E = q n + r, q floored (0 <= r < n): `exp(ln(m 2^r)/n) * 2^q`."""
    values = _values(args)
    p = _product(values)
    if len(values) == 1:
        return p
    n = len(values)
    mantissa, exponent = math.frexp(p)  # p = mantissa * 2^exponent, mantissa in [0.5, 1)
    m, e = mantissa * 2.0, exponent - 1
    q, r = divmod(e, n)
    return math.exp(math.log(math.ldexp(m, r)) / n) * math.ldexp(1.0, q)


def split_root_trunc(*args: Arg) -> float:
    """[`split_root`] with E = q n + r, q truncated toward zero (r has E's sign)."""
    values = _values(args)
    p = _product(values)
    if len(values) == 1:
        return p
    n = len(values)
    mantissa, exponent = math.frexp(p)
    m, e = mantissa * 2.0, exponent - 1
    q = int(e / n)
    r = e - q * n
    return math.exp(math.log(math.ldexp(m, r)) / n) * math.ldexp(1.0, q)


def split_exact_root(*args: Arg) -> float:
    """[`split_root`], or the correctly rounded root of P when it multiplies back to P."""
    values = _values(args)
    p = _product(values)
    root = split_root(*args)
    if len(values) == 1:
        return root
    c = rounded_product_root(*args)
    back = 1.0
    for _ in values:
        back *= c
    return c if back == p else root


def _candidates() -> list[Candidate]:
    return [
        candidate("correctly rounded", correctly_rounded, "reference"),
        candidate("15-digit snap (T-1123)", snapped_root, "snap"),
        candidate("plain exp(ln(P)/n)", plain_root, "plain"),
        candidate("correctly rounded root of the double P", rounded_product_root, "product root"),
        candidate("r, snapped to the root of P when it multiplies back", exact_root_or_plain, "exact root"),
        candidate("split root, q floored", split_root, "split"),
        candidate("exact root of P else split root", split_exact_root, "split"),
        candidate("split root, q toward zero", split_root_trunc, "split"),
    ]


def _in_range(cell: Cell) -> bool:
    """The snap's domain: more than one value and a normal plain product."""
    try:
        values = _values(cell.args)
    except TypeError:
        return False
    if len(values) < 2 or any(v <= 0 for v in values):
        return False
    p = _product(values)
    return math.isfinite(p) and p >= 2.0**-1022


_PATTERN = r"^=GEOMEAN\((.*)\)$"

FITS = [
    Fit(
        name="geomean-snap",
        description="GEOMEAN in-range root models over the T-1113/T-1123 literal probes (T-3502)",
        cases=_OLD,
        pattern=_PATTERN,
        candidates=_candidates,
        where=_in_range,
    ),
    Fit(
        name="geomean-snap-fresh",
        description="GEOMEAN in-range root on the fresh log-uniform sweep (T-3502, R4.2)",
        cases=_FRESH,
        pattern=_PATTERN,
        candidates=_candidates,
        where=_in_range,
    ),
    Fit(
        name="geomean-postfreeze",
        description="GEOMEAN in-range root on the post-freeze sweep (T-3502, R4.2 for the split root)",
        cases=_POSTFREEZE,
        pattern=_PATTERN,
        candidates=_candidates,
        where=_in_range,
    ),
    Fit(
        name="geomean-all",
        description="GEOMEAN in-range root over the T-1123 probes and both T-3502 sweeps, pooled",
        cases=_OLD + _FRESH + _POSTFREEZE,
        pattern=_PATTERN,
        candidates=_candidates,
        where=_in_range,
    ),
]
