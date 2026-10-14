"""Exact arithmetic for fit-lab candidates: rounding to a p-bit significand and the x87 model.

Every value is a `Fraction`, so a candidate can say precisely where it rounds. `round_to_bits`
rounds to nearest, ties to even, at any precision: p = 53 is IEEE double, p = 64 is the x87
extended significand. `X87` composes operations the way docs/excel-semantics/numbers.md
("Division rounds twice") describes Excel's `/`, `*`, `+`, `-` and SQRT: the exact result rounds
to 64 bits, then to a double. `Ext` keeps 64-bit intermediates across a chain of operations, for
hypotheses where a kernel stays in x87 registers (T-1524).

`exp_cr`, `ln_cr`, `pow_cr` and `sqrt_cr` are correctly rounded to p bits. `sqrt_cr` is exact
integer arithmetic. The others evaluate with `decimal` at increasing precision until the error
bound no longer straddles a rounding boundary (Ziv's strategy). `decimal`'s exp and ln are
themselves correctly rounded at the working precision, which the error bounds rely on.
"""

from __future__ import annotations

import decimal
import math
from collections.abc import Callable
from fractions import Fraction

DOUBLE = 53
EXTENDED = 64
# Least significant bit of the smallest subnormal double: the double grid ends at 2^-1074.
DOUBLE_MIN_EXP = -1074
_START_DIGITS = 40
_MAX_DIGITS = 2400  # about 8,000 bits: far beyond any double, but bounded (directive 5)
_MAX_ARG = 1 << 16  # exp of more than this is outside every binary64 and x87 range anyway
_MAX_EXACT_POW_BITS = 1 << 16

Number = Fraction | float | int


def exact(x: Number) -> Fraction:
    """The exact value of a double, int or Fraction (no rounding)."""
    if isinstance(x, float) and not math.isfinite(x):
        msg = f"{x} has no exact value"
        raise ValueError(msg)
    return x if isinstance(x, Fraction) else Fraction(x)


def round_to_bits(x: Number, p: int, min_exp: int | None = None) -> Fraction:
    """`x` rounded to the nearest number with a `p`-bit significand, ties to even.

    The exponent range is unbounded, unless `min_exp` gives the exponent of the smallest
    representable bit (-1074 for doubles, which makes results below 2^-1022 subnormal).
    """
    if p < 1:
        msg = f"precision must be at least 1 bit, not {p}"
        raise ValueError(msg)
    v = exact(x)
    if v == 0:
        return Fraction(0)
    a = abs(v)
    n, d = a.numerator, a.denominator
    e = n.bit_length() - d.bit_length()  # floor(log2(a)) is e or e - 1
    if (n << max(-e, 0)) < (d << max(e, 0)):
        e -= 1
    q = e - p + 1  # exponent of the last significand bit
    if min_exp is not None:
        q = max(q, min_exp)
    num, den = (n, d << q) if q >= 0 else (n << -q, d)
    m, r = divmod(num, den)
    if 2 * r > den or (2 * r == den and m & 1):
        m += 1
    result = Fraction(m << q) if q >= 0 else Fraction(m, 1 << -q)
    return result if v > 0 else -result


def to_double(x: Number) -> float:
    """`x` correctly rounded to a double, subnormals included (CPython's int/int division is
    correctly rounded). Raises OverflowError beyond the double range."""
    return float(exact(x))


def double_round(x: Number) -> float:
    """The x87 model: round to a 64-bit significand first, then to a double."""
    return to_double(round_to_bits(x, EXTENDED))


def sqrt_cr(x: Number, p: int = DOUBLE) -> Fraction:
    """sqrt(x) correctly rounded to `p` bits, by exact integer square root."""
    v = exact(x)
    if v < 0:
        msg = "sqrt of a negative number"
        raise ValueError(msg)
    if v == 0:
        return Fraction(0)
    n, d = v.numerator, v.denominator
    # Scale by 4^k so the integer root has at least p + 2 bits.
    k = max(0, (p + 2) - (n.bit_length() - d.bit_length()) // 2 + 1)
    scaled_num = n << (2 * k)
    s = math.isqrt(scaled_num // d)
    # An inexact root lies strictly between s and s + 1. s has more than p + 1 bits, so every
    # p-bit midpoint is an integer and s + 1/2 rounds like the true root.
    root = Fraction(s) if s * s * d == scaled_num else Fraction(2 * s + 1, 2)
    return round_to_bits(root / (1 << k), p)


def _context(digits: int) -> decimal.Context:
    return decimal.Context(
        prec=digits, rounding=decimal.ROUND_HALF_EVEN, Emax=decimal.MAX_EMAX, Emin=decimal.MIN_EMIN
    )


def _to_decimal(v: Fraction, ctx: decimal.Context) -> decimal.Decimal:
    """`v` at the context precision (relative error at most half a decimal ulp)."""
    return ctx.divide(decimal.Decimal(v.numerator), decimal.Decimal(v.denominator))


def _ziv(
    approx: Callable[[decimal.Context], tuple[decimal.Decimal, Fraction]], p: int, name: str
) -> Fraction:
    """Rounds the value that `approx` brackets to `p` bits, raising the working precision until
    the bracket [v - err, v + err] rounds to one number. `approx(ctx)` returns (v, err)."""
    digits = _START_DIGITS
    while digits <= _MAX_DIGITS:
        value, err = approx(_context(digits))
        v = Fraction(value)
        lo, hi = round_to_bits(v - err, p), round_to_bits(v + err, p)
        if lo == hi:
            return lo
        digits *= 2
    msg = f"{name}: no correctly rounded result within {_MAX_DIGITS} digits (exact midpoint?)"
    raise ArithmeticError(msg)


def _ulps(digits: int, extra: int = 2) -> Fraction:
    """10^(extra - digits): a few decimal ulps at `digits` precision, the error budget."""
    return Fraction(1, 10 ** (digits - extra))


def exp_cr(x: Number, p: int = DOUBLE) -> Fraction:
    """exp(x) correctly rounded to `p` bits. exp of a non-zero rational is transcendental, so the
    precision loop always ends."""
    v = exact(x)
    if v == 0:
        return Fraction(1)
    if abs(v) > _MAX_ARG:
        msg = f"exp argument {float(v)} out of range"
        raise OverflowError(msg)

    def approx(ctx: decimal.Context) -> tuple[decimal.Decimal, Fraction]:
        result = ctx.exp(_to_decimal(v, ctx))
        # The argument's rounding (relative 10^(1-P), so absolute |x| 10^(1-P)) becomes a relative
        # error of the result; exp's own rounding adds 10^(1-P).
        return result, abs(Fraction(result)) * (abs(v) + 1) * _ulps(ctx.prec)

    return _ziv(approx, p, "exp")


def ln_cr(x: Number, p: int = DOUBLE) -> Fraction:
    """ln(x) correctly rounded to `p` bits (x > 0). ln of a rational other than 1 is
    transcendental, so the precision loop always ends."""
    v = exact(x)
    if v <= 0:
        msg = "ln of a non-positive number"
        raise ValueError(msg)
    if v == 1:
        return Fraction(0)

    def approx(ctx: decimal.Context) -> tuple[decimal.Decimal, Fraction]:
        result = ctx.ln(_to_decimal(v, ctx))
        # Rounding the argument costs an absolute 10^(1-P) after ln; ln's own rounding is relative.
        return result, (1 + abs(Fraction(result))) * _ulps(ctx.prec)

    return _ziv(approx, p, "ln")


def _exact_power(base: Fraction, y: Fraction, p: int) -> Fraction | None:
    """base^y when it is rational and cheap to verify: always for an integer y, else when a
    (p+8)-bit candidate's denominator-th power reproduces it exactly (e.g. 4^0.5, 0.25^1.5)."""
    size = max(base.numerator.bit_length(), base.denominator.bit_length())
    if abs(y.numerator) * size > _MAX_EXACT_POW_BITS or y.denominator > 64:
        return None
    if y.denominator == 1:
        return base**y.numerator
    guess = round_to_bits(exp_cr(ln_cr(base, p + 16) * y, p + 16), p + 8)
    return guess if guess**y.denominator == base**y.numerator else None


def pow_cr(x: Number, y: Number, p: int = DOUBLE) -> Fraction:
    """x^y correctly rounded to `p` bits, for x > 0 (or x = 0 with y > 0)."""
    base, power = exact(x), exact(y)
    if base < 0:
        msg = "pow_cr needs a non-negative base"
        raise ValueError(msg)
    if base == 0:
        if power <= 0:
            msg = "0 to a non-positive power"
            raise ZeroDivisionError(msg)
        return Fraction(0)
    if power == 0 or base == 1:
        return Fraction(1)
    if (exact_value := _exact_power(base, power, p)) is not None:
        return round_to_bits(exact_value, p)
    if abs(power * Fraction(base.numerator.bit_length() - base.denominator.bit_length())) > _MAX_ARG:
        msg = "pow result out of range"
        raise OverflowError(msg)

    def approx(ctx: decimal.Context) -> tuple[decimal.Decimal, Fraction]:
        log = ctx.ln(_to_decimal(base, ctx))
        arg = ctx.multiply(log, _to_decimal(power, ctx))
        result = ctx.exp(arg)
        # ln's error (absolute (1 + |ln x|) 10^(2-P)) scaled by |y|, the product's and exp's
        # roundings: all relative errors of the result.
        rel = (abs(power) * (2 + abs(Fraction(log))) + 2) * _ulps(ctx.prec, 3)
        return result, abs(Fraction(result)) * rel

    return _ziv(approx, p, "pow")


class X87:
    """Single operations under the x87 model: exact result → 64-bit significand → double.

    Each method takes and returns doubles, as one Excel operator does (numbers.md: in a chained
    expression each operator's result is rounded twice before the next one uses it)."""

    @staticmethod
    def add(a: float, b: float) -> float:
        return double_round(exact(a) + exact(b))

    @staticmethod
    def sub(a: float, b: float) -> float:
        return double_round(exact(a) - exact(b))

    @staticmethod
    def mul(a: float, b: float) -> float:
        return double_round(exact(a) * exact(b))

    @staticmethod
    def div(a: float, b: float) -> float:
        return double_round(exact(a) / exact(b))

    @staticmethod
    def sqrt(a: float) -> float:
        """Correctly rounded sqrt at 64 bits, then to double."""
        return to_double(sqrt_cr(a, EXTENDED))

    @staticmethod
    def exp(a: float) -> float:
        """Correctly rounded exp at 64 bits, then to double (real x87 f2xm1 is not correctly
        rounded, so this is the idealized hypothesis)."""
        return to_double(exp_cr(a, EXTENDED))

    @staticmethod
    def ln(a: float) -> float:
        """Correctly rounded ln at 64 bits, then to double (idealized: real x87 fyl2x is not
        correctly rounded)."""
        return to_double(ln_cr(a, EXTENDED))

    @staticmethod
    def pow(a: float, b: float) -> float:
        """Correctly rounded x^y at 64 bits, then to double (idealized, not an x87 instruction
        sequence)."""
        return to_double(pow_cr(a, b, EXTENDED))


class Ext:
    """A value held in an x87 register: every operation rounds its exact result to 64 bits, and
    only `to_double` rounds to a double. Mixes freely with floats and ints."""

    __slots__ = ("value",)
    value: Fraction

    def __init__(self, value: Number | Ext) -> None:
        self.value = value.value if isinstance(value, Ext) else round_to_bits(value, EXTENDED)

    @staticmethod
    def _of(other: Number | Ext) -> Fraction:
        return other.value if isinstance(other, Ext) else exact(other)

    def __add__(self, other: Number | Ext) -> Ext:
        return Ext(self.value + self._of(other))

    __radd__ = __add__

    def __sub__(self, other: Number | Ext) -> Ext:
        return Ext(self.value - self._of(other))

    def __rsub__(self, other: Number | Ext) -> Ext:
        return Ext(self._of(other) - self.value)

    def __mul__(self, other: Number | Ext) -> Ext:
        return Ext(self.value * self._of(other))

    __rmul__ = __mul__

    def __truediv__(self, other: Number | Ext) -> Ext:
        return Ext(self.value / self._of(other))

    def __rtruediv__(self, other: Number | Ext) -> Ext:
        return Ext(self._of(other) / self.value)

    def __neg__(self) -> Ext:
        return Ext(-self.value)

    def sqrt(self) -> Ext:
        return Ext(sqrt_cr(self.value, EXTENDED))

    def exp(self) -> Ext:
        return Ext(exp_cr(self.value, EXTENDED))

    def ln(self) -> Ext:
        return Ext(ln_cr(self.value, EXTENDED))

    def to_double(self) -> float:
        return to_double(self.value)

    def __repr__(self) -> str:
        return f"Ext({float(self.value)!r})"
