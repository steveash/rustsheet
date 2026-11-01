"""FISHER, FISHERINV, the EXPON.DIST cdf and HARMEAN over every eval cell they own (T-2437).

Each kernel is one or two Excel operators around a LN or EXP, so the fit is about which operator
rounds twice (numbers.md "Division rounds twice") and in which order the terms are formed:

- FISHER: 0.5 * LN((1 + x) / (1 - x)) with the quotient rounded twice (`FISHER(1E-15/3.14159)` is
  2^-52 less a little, gen-m9/4ab2ba98d26c8a5e A395; the plain quotient gives 1.5 * 2^-52).
- FISHERINV: (EXP(2y) - 1) / (EXP(2y) + 1) with the quotient rounded twice (`FISHERINV(TAN(PI()))`
  is -2^-53, gen-m9/ee2497a5e683c5d9 A30).
- EXPON.DIST's cdf: Excel's Kahan expm1 (numbers.md "Excel's expm1 is Kahan's formula") of -lambda*x
  for lambda*x below a cut between 0.999999999999999 and 1.35, and 1 - EXP(-lambda*x) above it
  (probes/t2437-fit-separators A67:A117).
- HARMEAN: 1 / AVERAGE(1/x), every / and + an Excel operator, not n / sum(1/x).

Results: docs/excel-semantics/statistics.md "FISHER, FISHERINV, EXPON.DIST and HARMEAN are Excel
operators around LN and EXP", and T-2437's Outcome.
"""

from __future__ import annotations

import math
from collections.abc import Callable

from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg, Cell, numeric_arg

Binary = Callable[[float, float], float]
Unary = Callable[[float], float]

_GEN_M9 = [
    f"gen-m9/{h}"
    for h in [
        "05d44f6dd6cfc567",
        "06b4e8f5de409d5e",
        "0a614cfe974b59c3",
        "0a9226016e8f5f33",
        "0c7fc447bc2eebfd",
        "1466c66c61189a83",
        "146b46e74f2dac3b",
        "165db58d1540f014",
        "27601819397dd56e",
        "2801c1c11c21e134",
        "285d87cea1aaa73c",
        "2a18b2f5a4dc43ac",
        "320d51765af9b6b7",
        "35e8168aae2b00f8",
        "3a25a03eb94fcfbd",
        "3e98b26cf6713e77",
        "40425341c5034eed",
        "4250f880ab640c49",
        "4ab2ba98d26c8a5e",
        "4bad303d32cc3f46",
        "4e55500bb8ad3cd8",
        "530c636749a85209",
        "589f42e67f4b4b8d",
        "58e3253bbd3a7a82",
        "60b0b6e69ad35a45",
        "623663cacea53bbc",
        "68d8ffb69ff739d8",
        "7674c89c824aca43",
        "7bd219ec0c0c5e22",
        "7c20351562a2d71d",
        "85f48d5cdfb24555",
        "899f9555e67da2d6",
        "8d3588d8df0812df",
        "8dd2312c96effb1b",
        "8f2ac25945af87da",
        "8fec7b207f4dd753",
        "9cca17438a7e6f8a",
        "a013a45a8fbcab4f",
        "a54c738c697617c3",
        "a5828d3925575713",
        "a612e89321a6df9c",
        "a61e547a1cab994c",
        "a790a4d4a6a82756",
        "a7bf0d8d81d33ba6",
        "abc2939ebd1e2365",
        "b4173fe25109a238",
        "b6593d7dd13f2dac",
        "b8ee55934c5b824c",
        "bbdfc06cae0e0d71",
        "bc40d37cb4d8301d",
        "bcb24385baafb2f0",
        "c18dbf21f11093f3",
        "c2974c061384fdd9",
        "c468f5046aadfbbc",
        "c6cfda6bd10cae93",
        "caa1eed48b1883fe",
        "cde551ea787f713a",
        "d8266bc15533702a",
        "e6cf3b20a014e6a3",
        "eced4bb6a0234c9e",
        "ee2497a5e683c5d9",
        "f2bd5f380ec002bf",
        "f34f66fb7f81e673",
        "fd9fa22d212080d6",
        "fec39efaba9284c3",
    ]
]
_PROBE = "probes/t2437-fit-separators"


def _number(a: Arg) -> float:
    """A value argument as a number: an empty cell is 0 (`FISHER(Data!E3)` of a blank is 0)."""
    return 0.0 if a is None else numeric_arg(a)


def _plain_div(a: float, b: float) -> float:
    return a / b


def _plain_add(a: float, b: float) -> float:
    return a + b


_DIVISIONS: dict[str, Binary] = {"plain /": _plain_div, "/ rounded twice": X87.div}
_LOGS: dict[str, Unary] = {"libm ln": math.log, "LN (64-bit, then double)": X87.ln}
_EXPS: dict[str, Unary] = {"libm exp": math.exp, "EXP (64-bit, then double)": X87.exp}


def fisher_candidates() -> list[Candidate]:
    out = [
        candidate("atanh", lambda a: math.atanh(_number(a))),
        candidate(
            "0.5*(LN(1+x)-LN(1-x))",
            lambda a: 0.5 * (X87.ln(1 + _number(a)) - X87.ln(1 - _number(a))),
        ),
    ]
    for div_name, div in _DIVISIONS.items():
        for ln_name, ln in _LOGS.items():

            def f(a: Arg, div: Binary = div, ln: Unary = ln) -> float:
                x = _number(a)
                return 0.5 * ln(div(1 + x, 1 - x))

            out.append(candidate(f"0.5*{ln_name}((1+x){div_name}(1-x))", f, ln_name))

    def sums_twice(a: Arg) -> float:
        x = _number(a)
        return 0.5 * X87.ln(X87.div(X87.add(1, x), X87.sub(1, x)))

    out.append(
        candidate("0.5*LN((1+x)/(1-x)), sums and quotient rounded twice", sums_twice, "sums rounded twice")
    )
    return out


def fisherinv_candidates() -> list[Candidate]:
    out = [candidate("tanh", lambda a: math.tanh(_number(a)))]
    for div_name, div in _DIVISIONS.items():
        for exp_name, exp in _EXPS.items():

            def f(a: Arg, div: Binary = div, exp: Unary = exp) -> float:
                y = _number(a)
                try:
                    e = exp(2 * y)
                except OverflowError:
                    return 1.0
                return 1.0 if math.isinf(e) else div(e - 1, e + 1)

            out.append(candidate(f"(e-1){div_name}(e+1), e={exp_name}(2y)", f, exp_name))

    def sums_twice(a: Arg) -> float:
        try:
            e = X87.exp(2 * _number(a))
        except OverflowError:
            return 1.0
        return 1.0 if math.isinf(e) else X87.div(X87.sub(e, 1), X87.add(e, 1))

    out.append(
        candidate("(e-1)/(e+1), e=EXP(2y), sums and quotient rounded twice", sums_twice, "sums rounded twice")
    )
    return out


def kahan_expm1(t: float) -> float:
    """Excel's expm1 (`x87_exp.rs` `excel_expm1`): (u - 1) * t / LN(u), u = EXP(t)."""
    u = X87.exp(t)
    if u == 1.0:
        return t
    if u - 1.0 == -1.0:
        return -1.0
    return (u - 1.0) * t / X87.ln(u)


def _expon_t(x: Arg, lam: Arg) -> float:
    return _number(lam) * _number(x)


def _one_minus_exp(t: float) -> float:
    try:
        return 1.0 - X87.exp(-t)
    except OverflowError:
        return 1.0


def expon_candidates() -> list[Candidate]:
    out = [
        candidate("-libm expm1(-t)", lambda x, lam, _c: -math.expm1(-_expon_t(x, lam))),
        candidate("1-EXP(-t)", lambda x, lam, _c: _one_minus_exp(_expon_t(x, lam))),
        candidate("-Kahan expm1(-t)", lambda x, lam, _c: -kahan_expm1(-_expon_t(x, lam))),
    ]
    for cut in (0.5, 1.0, 2.0, 4.0, 8.0):

        def f(x: Arg, lam: Arg, _c: Arg, cut: float = cut) -> float:
            t = _expon_t(x, lam)
            return -kahan_expm1(-t) if t < cut else _one_minus_exp(t)

        out.append(candidate(f"-Kahan expm1(-t) below t={cut:g}, else 1-EXP(-t)", f, "Kahan, then 1-EXP"))
    return out


def _cumulative(cell: Cell) -> bool:
    flag = cell.args[2] if len(cell.args) == 3 else None
    if isinstance(flag, str):
        return flag.upper() == "TRUE"
    return isinstance(flag, bool | float) and bool(flag)


def _harmean_numbers(args: tuple[Arg, ...]) -> list[float]:
    out: list[float] = []
    # A reference (one cell or a range) gives only its numbers; the sweep cannot tell a one-cell
    # reference from a literal, so text, bools and blanks are skipped either way.
    for a in args:
        values = a if isinstance(a, list) else [a]
        out += [v for v in values if isinstance(v, float) and not isinstance(v, bool)]
    if not out or min(out) <= 0:
        msg = "HARMEAN needs positive numbers"
        raise ValueError(msg)
    return out


def harmean_candidates() -> list[Candidate]:
    out = []
    adds: dict[str, Binary] = {"plain +": _plain_add, "+ rounded twice": X87.add}
    for add_name, add in adds.items():
        for div_name, div in _DIVISIONS.items():

            def total(values: list[float], add: Binary = add, div: Binary = div) -> float:
                s = 0.0
                for v in values:
                    s = add(s, div(1.0, v))
                return s

            def n_over_sum(
                *args: Arg, total: Callable[[list[float]], float] = total, div: Binary = div
            ) -> float:
                values = _harmean_numbers(args)
                return div(float(len(values)), total(values))

            def one_over_mean(
                *args: Arg, total: Callable[[list[float]], float] = total, div: Binary = div
            ) -> float:
                values = _harmean_numbers(args)
                return div(1.0, div(total(values), float(len(values))))

            label = f"{add_name}, {div_name}"
            out.append(candidate(f"n/sum(1/x), {label}", n_over_sum, "n/sum(1/x)"))
            out.append(candidate(f"1/AVERAGE(1/x), {label}", one_over_mean, "1/AVERAGE(1/x)"))
    return out


FITS = [
    Fit(
        name="fisher",
        description="FISHER over the gen-m9 sweep and probes (T-2437)",
        cases=[*_GEN_M9, _PROBE, "probes/t2436/4250f880ab640c49", "probes/t2436/4ab2ba98d26c8a5e"],
        pattern=r"^=FISHER\((.*)\)$",
        candidates=fisher_candidates,
    ),
    Fit(
        name="fisherinv",
        description="FISHERINV over the gen-m9 sweep and probes (T-2437)",
        cases=[
            *_GEN_M9,
            _PROBE,
            "probes/stat-argument-modes",
            "probes/t2429/eced4bb6a0234c9e",
            "probes/t2436/7c20351562a2d71d",
        ],
        pattern=r"^=FISHERINV\((.*)\)$",
        candidates=fisherinv_candidates,
    ),
    Fit(
        name="expon-cdf",
        description="EXPON.DIST and EXPONDIST cumulative over the gen-m9 sweep and probes (T-2437)",
        cases=[
            *_GEN_M9,
            _PROBE,
            "probes-numeric/m9-legacy-expondist",
            "probes/stat-argument-modes",
            "probes/t2693-subnormal-expon-dist",
            "probes/t2706-subnormal-density-paths",
        ],
        pattern=r"^=(?:EXPON\.DIST|EXPONDIST)\((.*)\)$",
        candidates=expon_candidates,
        where=_cumulative,
    ),
    Fit(
        name="harmean",
        description="HARMEAN over the gen-m9 sweep and probes (T-2437)",
        cases=[
            *_GEN_M9,
            _PROBE,
            "probes/average-value-args",
            "probes/legacy-value-argument-intersection-b",
            "probes/multiarea-foreach",
            "probes/stat-argument-modes",
            "probes/stats-evidence-floor-t1623",
        ],
        pattern=r"^=HARMEAN\((.*)\)$",
        candidates=harmean_candidates,
    ),
]
