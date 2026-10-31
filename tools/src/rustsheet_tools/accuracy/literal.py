"""Formula literals as a Gate A table writes them (SPEC-0029, "Excel behavior to match").

Every argument is written with at most 15 significant digits (numbers.md, "Formula number literals
are read at 15 significant digits"), below 1E+308 ("Formula literals of 1E+308 or more are
invalid") and never as a subnormal ("Subnormal results become 0"): a magnitude is 0 or within
[`MIN_LITERAL`, `MAX_LITERAL`]. The math is evaluated at the double nearest the written decimal,
which is the double Excel and the engine read.
"""

from __future__ import annotations

from decimal import ROUND_HALF_EVEN, Decimal

# The largest 15-digit literal below 1E+308, and the smallest 15-digit literal at or above the
# smallest normal double (2.2250738585072014E-308).
MAX_LITERAL = 9.99999999999999e307
MIN_LITERAL = 2.22507385850721e-308

Arg = float | int | bool
"""One drawn argument: a real (float), an integer parameter (int) or a logical (bool)."""


def round15(x: float) -> float:
    """`x` rounded half-even to 15 significant digits and clamped into the writable range."""
    if x == 0:
        return 0.0
    magnitude = abs(x)
    if magnitude >= MAX_LITERAL:
        return MAX_LITERAL if x > 0 else -MAX_LITERAL
    if magnitude < MIN_LITERAL:
        return MIN_LITERAL if x > 0 else -MIN_LITERAL
    d = Decimal(x)
    d = d.quantize(Decimal(1).scaleb(d.adjusted() - 14), rounding=ROUND_HALF_EVEN)
    value = float(d)
    if abs(value) >= MAX_LITERAL:  # 9.999999999999999E+307 rounds up to 1E+308
        return MAX_LITERAL if x > 0 else -MAX_LITERAL
    return value


def neighbour(b: float, up: bool) -> float:
    """The nearest 15-digit literal above (`up`) or below `b`, itself a 15-digit literal: the
    literal next to a domain bound (0 → ±`MIN_LITERAL`, 1 → 0.999999999999999 or
    1.00000000000001)."""
    if b == 0:
        return MIN_LITERAL if up else -MIN_LITERAL
    d = Decimal(repr(b))
    step = Decimal(1).scaleb(d.adjusted() - 14)
    # Moving toward zero across a power of ten lands in the smaller decade (1 → 0.999999999999999).
    toward_zero = (d > 0) != up
    if toward_zero and abs(d).normalize() == Decimal(1).scaleb(d.adjusted()):
        step = step / 10
    return round15(float(d + step if up else d - step))


def text(a: Arg) -> str:
    """`a` as formula text: TRUE/FALSE, an integer's digits (15-digit exponent form above 1E15),
    or a real's 15 significant digits in Excel's exponent style (`1.87E-297`)."""
    if isinstance(a, bool):
        return "TRUE" if a else "FALSE"
    if isinstance(a, int) and abs(a) < 10**15:
        return str(a)
    s = f"{float(a):.15g}".upper()
    mantissa, sep, exponent = s.partition("E")
    if sep:
        sign = "-" if exponent.startswith("-") else "+"
        s = f"{mantissa}E{sign}{int(exponent.lstrip('+-')):02d}"
    return s
