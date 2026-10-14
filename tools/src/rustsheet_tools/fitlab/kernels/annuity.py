"""Annuity functions (PV, FV, PMT, IPMT, PPMT, NPER, NPV) over the T-0942 sweeps: the x87 test (T-1524).

Every candidate is a power source for P = (1+r)^n combined with a formula shape and a rounding
model for the formula's operations:

- power sources: square-and-multiply in doubles (numbers.md, "Integer exponents"), the same in
  64-bit registers on the 64-bit sum 1+r, the exact power, the x87 instruction sequence
  (`X87Insn.pow`, fyl2x/f2xm1/fscale on the 64-bit 1+r), and libm `pow` / `exp(n log1p r)`;
- rounding models: `double` (each operation rounded to a double), `x87 op` (each operation
  rounded to 64 bits, then to a double: Excel's operators), `ext` (every intermediate kept at 64
  bits, P included, one final rounding to a double: the x87-register hypothesis) and `ext, P
  double` (the same with P stored to a double first);
- `correctly rounded`: the exact value of the formula for the cell's double arguments.

The sweeps have integer nper (1..360) and rates 0.2, 0.1, 0.05, 0.07, 0.125, 0.3, 0.01, -0.02 and
0.05/12, 0.035/12, 0.0525/12 (the divisions are evaluated with Excel's `/`). Results:
docs/excel-semantics/financial.md, "Annuity evaluation order", and T-1524's Outcome.
"""

from __future__ import annotations

import math
from collections.abc import Callable
from fractions import Fraction
from functools import cache

from rustsheet_tools.fitlab.exact import (
    Rounding,
    X87Insn,
    extended_value,
    ln_cr,
    round_to_bits,
    square_and_multiply,
    to_double,
    to_double_value,
    x87_value,
)
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg

CASE_PMT = "probes-numeric/m9-fin-pmt"
CASE_PV_FV = "probes-numeric/m9-fin-pv-fv"
CASE_NPER_NPV = "probes-numeric/m9-fin-nper-npv"
_HIGH = 120  # bits for the near-exact logarithms of NPER


def _nums(args: tuple[Arg, ...], count: int) -> list[Fraction]:
    """The cell's numeric arguments, padded with zeros (omitted fv/pv/type) to `count`."""
    out: list[Fraction] = []
    for a in args:
        if isinstance(a, bool) or not isinstance(a, float):
            msg = f"non-numeric argument {a!r}"
            raise TypeError(msg)
        out.append(Fraction(a))
    return out + [Fraction(0)] * (count - len(out))


def _int(n: Fraction) -> int:
    if n.denominator != 1 or n < 0:
        msg = f"nper {float(n)} is not a non-negative integer"
        raise ValueError(msg)
    return int(n)


# Power sources: (r, n) -> P, possibly at 64 bits. Cached: shapes and models share them.
@cache
def p_sqm_double(r: Fraction, n: int) -> Fraction:
    return square_and_multiply(x87_value(1 + r), n, x87_value)


@cache
def p_sqm_ext(r: Fraction, n: int) -> Fraction:
    return square_and_multiply(extended_value(1 + r), n, extended_value)


@cache
def p_exact(r: Fraction, n: int) -> Fraction:
    return (1 + r) ** n


@cache
def p_x87_insn(r: Fraction, n: int) -> Fraction:
    return X87Insn.pow(extended_value(1 + r), n) if n else Fraction(1)


@cache
def p_libm_pow(r: Fraction, n: int) -> Fraction:
    return Fraction((1 + float(r)) ** n)


@cache
def p_exp_log1p(r: Fraction, n: int) -> Fraction:
    return Fraction(math.exp(n * math.log1p(float(r))))


POWERS: dict[str, Callable[[Fraction, int], Fraction]] = {
    "P sqm double": p_sqm_double,
    "P sqm ext": p_sqm_ext,
    "P exact": p_exact,
    "P x87 insn": p_x87_insn,
    "P libm pow": p_libm_pow,
    "P exp(n log1p r)": p_exp_log1p,
}


class Ops:
    """The formula's arithmetic under one rounding model."""

    def __init__(self, rnd: Rounding) -> None:
        self.rnd = rnd

    def add(self, a: Fraction, b: Fraction) -> Fraction:
        return self.rnd(a + b)

    def sub(self, a: Fraction, b: Fraction) -> Fraction:
        return self.rnd(a - b)

    def mul(self, a: Fraction, b: Fraction) -> Fraction:
        return self.rnd(a * b)

    def div(self, a: Fraction, b: Fraction) -> Fraction:
        return self.rnd(a / b)


def _exact_value(x: Fraction) -> Fraction:
    return x


# name -> (rounding of each operation, rounding applied to P before the formula)
MODELS: dict[str, tuple[Rounding, Rounding]] = {
    "double": (to_double_value, to_double_value),
    "x87 op": (x87_value, to_double_value),
    "ext": (extended_value, extended_value),
    "ext, P double": (extended_value, to_double_value),
}

# A shape computes the result from the arguments and P under `Ops`.
Shape = Callable[[Ops, list[Fraction], Fraction], Fraction]


def _k(o: Ops, r: Fraction, t: Fraction) -> Fraction:
    return o.add(Fraction(1), o.mul(r, t))


def _annuity_factor(o: Ops, r: Fraction, t: Fraction, p: Fraction) -> Fraction:
    """k*((P-1)/r), the parenthesization financial.md fits for PV and FV."""
    return o.mul(_k(o, r, t), o.div(o.sub(p, Fraction(1)), r))


# PV(r, n, pmt, fv, type)
PV_SHAPES: dict[str, Shape] = {
    "-(fv+pmt*(k*((P-1)/r)))/P": lambda o, a, p: (
        -o.div(o.add(a[3], o.mul(a[2], _annuity_factor(o, a[0], a[4], p))), p)
    ),
    "engine: (-pmt*k*(P-1)/r-fv)/P": lambda o, a, p: o.div(
        o.sub(o.div(o.mul(o.mul(-a[2], _k(o, a[0], a[4])), o.sub(p, Fraction(1))), a[0]), a[3]), p
    ),
}
# FV(r, n, pmt, pv, type)
FV_SHAPES: dict[str, Shape] = {
    "-(pv*P+pmt*(k*((P-1)/r)))": lambda o, a, p: (
        -o.add(o.mul(a[3], p), o.mul(a[2], _annuity_factor(o, a[0], a[4], p)))
    ),
    "engine: -pv*P-pmt*k*(P-1)/r": lambda o, a, p: o.sub(
        o.mul(-a[3], p), o.div(o.mul(o.mul(a[2], _k(o, a[0], a[4])), o.sub(p, Fraction(1))), a[0])
    ),
}
# PMT(r, n, pv, fv, type)
PMT_SHAPES: dict[str, Shape] = {
    "-(r*(fv+pv*P))/(k*(P-1))": lambda o, a, p: (
        -o.div(o.mul(a[0], o.add(a[3], o.mul(a[2], p))), o.mul(_k(o, a[0], a[4]), o.sub(p, Fraction(1))))
    ),
    "-((fv+pv*P)*r/(P-1))/k": lambda o, a, p: (
        -o.div(o.div(o.mul(o.add(a[3], o.mul(a[2], p)), a[0]), o.sub(p, Fraction(1))), _k(o, a[0], a[4]))
    ),
    "-(fv+pv*P)/(k*((P-1)/r))": lambda o, a, p: (
        -o.div(o.add(a[3], o.mul(a[2], p)), _annuity_factor(o, a[0], a[4], p))
    ),
    "engine: (fv*r+pv*r*P)/(1-P)/k": lambda o, a, p: o.div(
        o.div(o.add(o.mul(a[3], a[0]), o.mul(o.mul(a[2], a[0]), p)), o.sub(Fraction(1), p)),
        _k(o, a[0], a[4]),
    ),
}


def _model_fn(
    shape: Shape, power: Callable[[Fraction, int], Fraction], model: tuple[Rounding, Rounding], count: int
) -> Callable[..., float]:
    rnd, p_rnd = model
    ops = Ops(rnd)

    def evaluate(*args: Arg) -> float:
        a = _nums(args, count)
        return to_double(shape(ops, a, p_rnd(power(a[0], _int(a[1])))))

    return evaluate


def _exact_fn(shape: Shape, count: int) -> Callable[..., float]:
    ops = Ops(_exact_value)

    def evaluate(*args: Arg) -> float:
        a = _nums(args, count)
        return to_double(shape(ops, a, p_exact(a[0], _int(a[1]))))

    return evaluate


def grid(shapes: dict[str, Shape], count: int) -> Callable[[], list[Candidate]]:
    """Every shape x power source x rounding model, plus the correctly rounded value. A family
    is a rounding model, so the family table shows how far each hypothesis gets."""

    def build() -> list[Candidate]:
        first = next(iter(shapes.values()))
        out = [candidate("correctly rounded", _exact_fn(first, count), "correctly rounded")]
        for model_name, model in MODELS.items():
            for power_name, power in POWERS.items():
                for shape_name, shape in shapes.items():
                    out.append(
                        candidate(
                            f"{model_name}: {power_name}, {shape_name}",
                            _model_fn(shape, power, model, count),
                            model_name,
                        )
                    )
        return out

    return build


def _pmt_accurate(shape: Callable[[float, float, float, float, float, float], float]) -> Callable[..., float]:
    """PMT in doubles with P = exp(n log1p r) and P-1 = expm1(n log1p r): no cancellation in
    P-1 (the engine's (1+r)^n - 1 loses up to log2(1/(r n)) bits)."""

    def evaluate(*args: Arg) -> float:
        r, n, pv, fv, t = (float(v) for v in _nums(args, 5))
        y = n * math.log1p(r)
        return shape(r, pv, fv, t, math.exp(y), math.expm1(y))

    return evaluate


PMT_ACCURATE: dict[str, Callable[[float, float, float, float, float, float], float]] = {
    "-(pv*P+fv)/((P-1)/r*k)": lambda r, pv, fv, t, p, m: -(pv * p + fv) / (m / r * (1 + r * t)),
    "-(pv*P/(P-1)+fv/(P-1))*r/k": lambda r, pv, fv, t, p, m: -(pv * p / m + fv / m) * r / (1 + r * t),
    "-(pv*r+(pv+fv)*r/(P-1))/k": lambda r, pv, fv, t, _p, m: -(pv * r + (pv + fv) * r / m) / (1 + r * t),
}


def pmt_candidates() -> list[Candidate]:
    out = grid(PMT_SHAPES, 5)()
    out += [
        candidate(f"double, P-1 = expm1(n log1p r): {name}", _pmt_accurate(shape), "double, accurate P-1")
        for name, shape in PMT_ACCURATE.items()
    ]
    return out


# IPMT(r, per, n, pv, fv, type) and PPMT: payment from a PMT shape, then the FV recurrence
# (ipmt = FV(r, per-1, pmt, pv)*r; type 1: per 1 is 0, else (FV(r, per-2, pmt, pv, 1) - pmt)*r).
def _ipmt_parts(
    o: Ops, a: list[Fraction], power: Callable[[Fraction, int], Fraction], p_rnd: Rounding
) -> tuple[Fraction, Fraction]:
    r, per, n, pv, fv, t = a
    pmt_shape = PMT_SHAPES["-(r*(fv+pv*P))/(k*(P-1))"]
    pmt = pmt_shape(o, [r, n, pv, fv, t], p_rnd(power(r, _int(n))))
    if t != 0 and per == 1:
        return pmt, Fraction(0)
    m = _int(per - 2 if t != 0 else per - 1)
    big = p_rnd(power(r, m))
    fv_shape = FV_SHAPES["-(pv*P+pmt*(k*((P-1)/r)))"]
    fv_at = fv_shape(o, [r, Fraction(m), pmt, pv, t], big)
    ipmt = o.mul(o.sub(fv_at, pmt), r) if t != 0 else o.mul(fv_at, r)
    return pmt, ipmt


def ipmt_grid(principal: bool) -> Callable[[], list[Candidate]]:
    def build() -> list[Candidate]:
        out: list[Candidate] = []
        models = {"exact": (_exact_value, _exact_value), **MODELS}
        powers = {"P exact": p_exact} | {k: v for k, v in POWERS.items() if k != "P exact"}
        for model_name, (rnd, p_rnd) in models.items():
            for power_name, power in powers.items():
                if model_name == "exact" and power_name != "P exact":
                    continue
                ops = Ops(rnd)

                def evaluate(
                    *args: Arg,
                    ops: Ops = ops,
                    power: Callable[[Fraction, int], Fraction] = power,
                    p_rnd: Rounding = p_rnd,
                ) -> float:
                    pmt, ipmt = _ipmt_parts(ops, _nums(args, 6), power, p_rnd)
                    return to_double(ops.sub(pmt, ipmt) if principal else ipmt)

                name = "correctly rounded" if model_name == "exact" else f"{model_name}: {power_name}"
                out.append(candidate(name, evaluate, model_name))
        return out

    return build


# NPER(r, pmt, pv, fv, type) = ln((pmt*k - fv*r)/(pmt*k + pv*r)) / ln(1+r)
def nper_candidates() -> list[Candidate]:
    def ratio(o: Ops, a: list[Fraction]) -> Fraction:
        r, pmt, pv, fv, t = a
        pk = o.mul(pmt, _k(o, r, t))
        return o.div(o.sub(pk, o.mul(fv, r)), o.add(pk, o.mul(pv, r)))

    exact_ops = Ops(_exact_value)

    def correctly_rounded(*args: Arg) -> float:
        a = _nums(args, 5)
        num = ln_cr(ratio(exact_ops, a), _HIGH)
        return to_double(num / ln_cr(1 + a[0], _HIGH))

    def libm(log_rate: Callable[[float], float], rnd: Rounding) -> Callable[..., float]:
        ops = Ops(rnd)

        def evaluate(*args: Arg) -> float:
            a = _nums(args, 5)
            return float(
                x87_value(Fraction(math.log(float(ratio(ops, a)))) / Fraction(log_rate(float(a[0]))))
            )

        return evaluate

    def x87(insn: bool, keep: bool) -> Callable[..., float]:
        rnd = extended_value if keep else x87_value
        ops = Ops(rnd)

        def ln(x: Fraction) -> Fraction:
            value = X87Insn.ln(x) if insn else round_to_bits(ln_cr(x, _HIGH), 64)
            return rnd(value)

        def evaluate(*args: Arg) -> float:
            a = _nums(args, 5)
            base = extended_value(1 + a[0]) if keep else x87_value(1 + a[0])
            return to_double(rnd(ln(ratio(ops, a)) / ln(base)))

        return evaluate

    return [
        candidate("correctly rounded", correctly_rounded, "correctly rounded"),
        candidate("double: log(ratio)/log(1+r)", libm(lambda r: math.log(1 + r), to_double_value), "double"),
        candidate("double: log(ratio)/log1p(r)", libm(math.log1p, to_double_value), "double"),
        candidate("x87 op: log(ratio)/log(1+r)", libm(lambda r: math.log(1 + r), x87_value), "x87 op"),
        candidate("x87 op: ln64 (idealized)", x87(insn=False, keep=False), "x87 op"),
        candidate("ext: ln64 (idealized)", x87(insn=False, keep=True), "ext"),
        candidate("ext: fldln2/fyl2x", x87(insn=True, keep=True), "ext"),
        candidate("x87 op: fldln2/fyl2x", x87(insn=True, keep=False), "x87 op"),
    ]


# NPV(r, v1, ..., vn) = sum v_i / (1+r)^i
def npv_candidates() -> list[Candidate]:
    def values(args: tuple[Arg, ...]) -> tuple[Fraction, list[Fraction]]:
        a = _nums(args, len(args))
        return a[0], a[1:]

    def by_power(
        power: Callable[[Fraction, int], Fraction], rnd: Rounding, p_rnd: Rounding
    ) -> Callable[..., float]:
        def evaluate(*args: Arg) -> float:
            r, vs = values(args)
            total = Fraction(0)
            for i, v in enumerate(vs, 1):
                total = rnd(total + rnd(v / p_rnd(power(r, i))))
            return to_double(total)

        return evaluate

    def horner(rnd: Rounding) -> Callable[..., float]:
        def evaluate(*args: Arg) -> float:
            r, vs = values(args)
            base = rnd(1 + r)
            total = Fraction(0)
            for v in reversed(vs):
                total = rnd(rnd(total + v) / base)
            return to_double(total)

        return evaluate

    def running(rnd: Rounding) -> Callable[..., float]:
        """A running discount factor d = d * (1+r)."""

        def evaluate(*args: Arg) -> float:
            r, vs = values(args)
            base, d, total = rnd(1 + r), Fraction(1), Fraction(0)
            for v in vs:
                d = rnd(d * base)
                total = rnd(total + rnd(v / d))
            return to_double(total)

        return evaluate

    out = [candidate("correctly rounded", by_power(p_exact, _exact_value, _exact_value), "correctly rounded")]
    for model_name, (rnd, p_rnd) in MODELS.items():
        for power_name, power in POWERS.items():
            out.append(
                candidate(f"{model_name}: sum v/{power_name}", by_power(power, rnd, p_rnd), model_name)
            )
        out.append(candidate(f"{model_name}: Horner (v_n/(1+r)+v_n-1)/(1+r)...", horner(rnd), model_name))
        out.append(candidate(f"{model_name}: running factor d*=(1+r)", running(rnd), model_name))
    return out


FITS = [
    Fit(
        name="annuity-pv",
        description="PV over the T-0942 sweep: power sources x shapes x rounding models",
        cases=[CASE_PV_FV],
        pattern=r"^=PV\((.*)\)$",
        candidates=grid(PV_SHAPES, 5),
    ),
    Fit(
        name="annuity-fv",
        description="FV over the T-0942 sweep: power sources x shapes x rounding models",
        cases=[CASE_PV_FV],
        pattern=r"^=FV\((.*)\)$",
        candidates=grid(FV_SHAPES, 5),
    ),
    Fit(
        name="annuity-pmt",
        description="PMT over the T-0942 sweep: power sources x shapes x rounding models",
        cases=[CASE_PMT],
        pattern=r"^=PMT\((.*)\)$",
        candidates=pmt_candidates,
    ),
    Fit(
        name="annuity-ipmt",
        description="IPMT over the T-0942 sweep: PMT, then the FV recurrence, per rounding model",
        cases=[CASE_PMT],
        pattern=r"^=IPMT\((.*)\)$",
        candidates=ipmt_grid(principal=False),
    ),
    Fit(
        name="annuity-ppmt",
        description="PPMT over the T-0942 sweep: PMT - IPMT, per rounding model",
        cases=[CASE_PMT],
        pattern=r"^=PPMT\((.*)\)$",
        candidates=ipmt_grid(principal=True),
    ),
    Fit(
        name="annuity-nper",
        description="NPER over the T-0942 sweep: logarithm sources x rounding models",
        cases=[CASE_NPER_NPV],
        pattern=r"^=NPER\((.*)\)$",
        candidates=nper_candidates,
    ),
    Fit(
        name="annuity-npv",
        description="NPV over the T-0942 sweep: discounting orders x power sources x rounding models",
        cases=[CASE_NPER_NPV],
        pattern=r"^=NPV\((.*)\)$",
        candidates=npv_candidates,
    ),
]
