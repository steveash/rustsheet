"""`^`, POWER and EXP(n*LN(x)) over the power sweeps: the x87 test (T-1524).

One set of pow models is scored on every sweep:

- `Excel model (doubles)`: numbers.md as recorded. Integer exponents are right-to-left
  square-and-multiply in doubles, each product rounded twice like Excel's `*` (the variant
  `products rounded once` is the engine's `integer_power` today); 0.5 is SQRT; other exponents are exp(y*ln x) with libm exp and
  log; a negative exponent is 1 over the positive power, with Excel's twice-rounded `/`.
- `correctly rounded`: x^y rounded once.
- `x87 pow (idealized)`: x^y rounded once to 64 bits, then to a double (`X87.pow`).
- `x87 insn`: fyl2x, frndint, f2xm1, fscale, each rounded once to 64 bits (`X87Insn.pow`); and
  the same with a negative exponent done as 1 over the positive power.
- `ext`: the Excel model's steps (square-and-multiply, or ln, multiply, exp) with every
  intermediate kept at 64 bits, then one rounding to a double; `ext exp(y ln x)` uses the
  logarithm path for integer exponents too.
- `libm pow`.

The `power-1+r` fit passes the rate, not the base, so the base 1+r can also stay at 64 bits (as
it would in an x87 register). The `exp-ln` fit scores EXP(n*LN(1+r)) as a chain of Excel
functions with each function's result stored as a double, or kept at 64 bits throughout.
Results: docs/excel-semantics/numbers.md, "x87 extended precision is not a kernel-wide model".
"""

from __future__ import annotations

import math
from collections.abc import Callable
from fractions import Fraction

from rustsheet_tools.fitlab.exact import (
    EXTENDED,
    X87,
    Rounding,
    X87Insn,
    exp_cr,
    extended_value,
    ln_cr,
    pow_cr,
    round_to_bits,
    sqrt_cr,
    square_and_multiply,
    to_double,
    to_double_value,
    x87_value,
)
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg, Cell

Pow = Callable[[Fraction, float], float]  # (base, possibly at 64 bits; exponent) -> result

_OPERAND = r"\([^()]*\)|-?[0-9.]+(?:E[-+]?\d+)?"  # a number or a parenthesized expression
_BINARY = rf"({_OPERAND})\^({_OPERAND})"
PATTERN_POW = rf"^=(?:POWER\(([^,]+),([^,]+)\)|{_BINARY})$"


def _float(a: Arg) -> float:
    if isinstance(a, bool) or not isinstance(a, float):
        msg = f"non-numeric argument {a!r}"
        raise TypeError(msg)
    return a


def _is_int(y: float) -> bool:
    return y == int(y)


def _recip_double(v: Fraction) -> float:
    """Excel's `1/v` on a double: twice rounded (numbers.md, "Division rounds twice")."""
    return X87.div(1.0, to_double(v))


def excel_model(x: Fraction, y: float, product: Rounding = x87_value) -> float:
    if _is_int(y):
        positive = square_and_multiply(x87_value(x), abs(int(y)), product)
    elif abs(y) == 0.5:
        positive = Fraction(X87.sqrt(to_double(x)))
    else:
        xd = to_double(x)
        positive = Fraction(math.exp(X87.mul(abs(y), math.log(xd))))
    return _recip_double(positive) if y < 0 else to_double(positive)


def excel_model_single(x: Fraction, y: float) -> float:
    """The Excel model with each square-and-multiply product rounded once (the engine today)."""
    return excel_model(x, y, to_double_value)


def correctly_rounded(x: Fraction, y: float) -> float:
    return to_double(pow_cr(x, y))


def x87_ideal(x: Fraction, y: float) -> float:
    return to_double(round_to_bits(pow_cr(x, y, EXTENDED), EXTENDED))


def x87_insn(x: Fraction, y: float) -> float:
    return to_double(X87Insn.pow(x, y))


def x87_insn_recip(x: Fraction, y: float) -> float:
    positive = X87Insn.pow(x, abs(y))
    return _recip_double(positive) if y < 0 else to_double(positive)


def _ext_explog(x: Fraction, y: float) -> Fraction:
    log = round_to_bits(ln_cr(x, EXTENDED), EXTENDED)
    return round_to_bits(exp_cr(round_to_bits(log * Fraction(y), EXTENDED), EXTENDED), EXTENDED)


def ext_model(x: Fraction, y: float) -> float:
    base = extended_value(x)
    if _is_int(y):
        positive = square_and_multiply(base, abs(int(y)), extended_value)
    elif abs(y) == 0.5:
        positive = sqrt_cr(base, EXTENDED)
    else:
        positive = _ext_explog(base, abs(y))
    return to_double(extended_value(1 / positive) if y < 0 else positive)


def ext_explog(x: Fraction, y: float) -> float:
    positive = _ext_explog(extended_value(x), abs(y))
    return to_double(extended_value(1 / positive) if y < 0 else positive)


def libm_pow(x: Fraction, y: float) -> float:
    return float(to_double(x) ** y)


MODELS: dict[str, Pow] = {
    "Excel model (doubles)": excel_model,
    "Excel model, products rounded once": excel_model_single,
    "correctly rounded": correctly_rounded,
    "x87 pow (idealized)": x87_ideal,
    "x87 insn": x87_insn,
    "x87 insn, 1/ for y<0": x87_insn_recip,
    "ext": ext_model,
    "ext exp(y ln x)": ext_explog,
    "libm pow": libm_pow,
}


def binary_candidates() -> list[Candidate]:
    """The models on (x, y) arguments, x a double."""

    def wrap(model: Pow) -> Callable[..., float]:
        return lambda x, y: model(Fraction(_float(x)), _float(y))

    return [candidate(name, wrap(model)) for name, model in MODELS.items()]


def rate_candidates() -> list[Candidate]:
    """The models on (r, n): the base 1+r as a double (Excel's `+`) or kept at 64 bits."""
    out: list[Candidate] = []
    for base_name, base in (
        ("1+r double", lambda r: Fraction(X87.add(1.0, r))),
        ("1+r ext", lambda r: extended_value(1 + Fraction(r))),
    ):
        for name, model in MODELS.items():

            def evaluate(
                r: Arg, n: Arg, model: Pow = model, base: Callable[[float], Fraction] = base
            ) -> float:
                return model(base(_float(r)), _float(n))

            out.append(candidate(f"{name}, {base_name}", evaluate, name))
    return out


def sqrt5_candidates() -> list[Candidate]:
    """SQRT(5)^n: the base as Excel's SQRT (twice rounded, which is also the correctly rounded
    double here) or kept at 64 bits (an x87 register)."""
    out: list[Candidate] = []
    for base_name, base in (
        ("SQRT double", Fraction(X87.sqrt(5.0))),
        ("SQRT ext", sqrt_cr(5, EXTENDED)),
    ):
        for name, model in MODELS.items():

            def evaluate(n: Arg, model: Pow = model, base: Fraction = base) -> float:
                return model(base, _float(n))

            out.append(candidate(f"{name}, {base_name}", evaluate, name))
    return out


def exp_ln_candidates() -> list[Candidate]:
    """EXP(n*LN(1+r)): each function's result a double, or the whole chain at 64 bits."""

    def chain(
        ln: Callable[[Fraction], Fraction], exp: Callable[[Fraction], Fraction], keep: bool
    ) -> Callable[..., float]:
        store = extended_value if keep else x87_value

        def evaluate(n: Arg, r: Arg) -> float:
            base = extended_value(1 + Fraction(_float(r))) if keep else Fraction(X87.add(1.0, _float(r)))
            log = store(ln(base))
            return to_double(store(exp(store(log * Fraction(_float(n))))))

        return evaluate

    def libm(n: Arg, r: Arg) -> float:
        return math.exp(X87.mul(_float(n), math.log(X87.add(1.0, _float(r)))))

    def ln_cr53(x: Fraction) -> Fraction:
        return Fraction(to_double(ln_cr(x)))

    def exp_cr53(x: Fraction) -> Fraction:
        return Fraction(to_double(exp_cr(x)))

    def ln_cr64(x: Fraction) -> Fraction:
        return ln_cr(x, EXTENDED)

    def exp_cr64(x: Fraction) -> Fraction:
        return exp_cr(x, EXTENDED)

    return [
        candidate("libm exp/log, x87 op", libm, "double"),
        candidate("correctly rounded LN and EXP, x87 op", chain(ln_cr53, exp_cr53, keep=False), "double"),
        candidate("x87 LN/EXP (idealized), stored", chain(ln_cr64, exp_cr64, keep=False), "x87 stored"),
        candidate("x87 insn LN/EXP, stored", chain(X87Insn.ln, X87Insn.exp, keep=False), "x87 stored"),
        candidate("ext chain (idealized)", chain(ln_cr64, exp_cr64, keep=True), "ext"),
        candidate("ext chain (x87 insn)", chain(X87Insn.ln, X87Insn.exp, keep=True), "ext"),
    ]


def _positive_base(cell: Cell) -> bool:
    x = cell.args[0] if cell.args else None
    return isinstance(x, float) and x > 0


FITS = [
    Fit(
        name="power-1+r",
        description="(1+r)^n, POWER(1+r,n) and their reciprocals (m9-fin-power): pow models x base",
        cases=["probes-numeric/m9-fin-power"],
        pattern=r"^=(?:POWER\(1\+([^,]+),(-?\d+)\)|\(1\+([^()]+)\)\^(-?\d+))$",
        candidates=rate_candidates,
    ),
    Fit(
        name="exp-ln",
        description="EXP(n*LN(1+r)) (m9-fin-power): function results stored as doubles or kept at 64 bits",
        cases=["probes-numeric/m9-fin-power"],
        pattern=r"^=EXP\((-?\d+)\*LN\(1\+([^()]+)\)\)$",
        candidates=exp_ln_candidates,
    ),
    Fit(
        name="power-nonint",
        description="x^y, x > 0, non-integer and reciprocal exponents (power-non-integer-sweep)",
        cases=["probes-numeric/power-non-integer-sweep"],
        pattern=PATTERN_POW,
        candidates=binary_candidates,
        where=_positive_base,
    ),
    Fit(
        name="power-recip",
        description="x^-1, x^-3, x^-0.5, x^-0.3, POWER(x,-1) (power-reciprocal-rounding)",
        cases=["probes-numeric/power-reciprocal-rounding"],
        pattern=PATTERN_POW,
        candidates=binary_candidates,
        where=_positive_base,
    ),
    Fit(
        name="power-mantissa",
        description="literal full-significand bases ^ 3..255 (power-integer-full-mantissa; "
        "the function-call bases are unparsed)",
        cases=["probes-numeric/power-integer-full-mantissa"],
        pattern=PATTERN_POW,
        candidates=binary_candidates,
        where=_positive_base,
    ),
    Fit(
        name="power-large-int",
        description="integer exponents up to 2^53, positive bases (power-large-integer-exponents{,-2,-3,-4}, T-1311); "
        "base 10 and out-of-range results follow their own rules",
        cases=[
            "probes-numeric/power-large-integer-exponents",
            "probes-numeric/power-large-integer-exponents-2",
            "probes-numeric/power-large-integer-exponents-3",
            "probes-numeric/power-large-integer-exponents-4",
        ],
        pattern=PATTERN_POW,
        candidates=binary_candidates,
        where=_positive_base,
    ),
    Fit(
        name="power-sqrt5",
        description="SQRT(5)^n (power-integer-full-mantissa, T-1495): base double or 64-bit",
        cases=["probes-numeric/power-integer-full-mantissa"],
        pattern=r"^=SQRT\(5\)\^(-?\d+)$",
        candidates=sqrt5_candidates,
    ),
]
