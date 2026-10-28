"""T-3014 scratch score: PMT through PPMT's discount, against Excel's m9-fin-pmt bits.

Gap triage 2026-10-28. Rough lower bound only: L is the atanh series of financial.md "PPMT discounts with an
atanh-series log" (64-bit partial sums, then stored and doubled; Excel's LN from 0.38 on is approximated by
libm), EXP and LN are libm (not the engine's x87 models), the operators are plain doubles (not Excel's
twice-rounded ones), and expm1 is Kahan's formula on libm. The real fit must use Excel's EXP/LN models
(docs/research/t2464/excelmath.py, through the python bindings) and Excel's operators.

Run from the repository root: python3 docs/research/t3014_pmt_discount_score.py
"""

from __future__ import annotations

import json
import math
from collections import Counter
from fractions import Fraction
from pathlib import Path

CASES = Path("eval/suites/probes-numeric/cases")


def round_bits(x: Fraction, bits: int) -> Fraction:
    """`x` rounded to nearest-even with a `bits`-bit significand."""
    if x == 0:
        return x
    sign = -1 if x < 0 else 1
    x = abs(x)
    e = x.numerator.bit_length() - x.denominator.bit_length()
    while Fraction(2) ** e > x:
        e -= 1
    while Fraction(2) ** (e + 1) <= x:
        e += 1
    q = x / Fraction(2) ** (e - bits + 1)
    whole = q.numerator // q.denominator
    rem = q - whole
    if rem > Fraction(1, 2) or (rem == Fraction(1, 2) and whole % 2 == 1):
        whole += 1
    return sign * whole * Fraction(2) ** (e - bits + 1)


def excel_l(rate: float) -> float:
    """financial.md's L = ln(1+rate): the atanh series below 0.38 (and above -0.375), LN of 1+rate beyond."""
    if rate >= 0.38 or rate <= -0.375:
        return math.log(1.0 + rate)
    s = Fraction(rate / (2.0 + rate))
    s2 = round_bits(s * s, 64)
    power, total, k = s, s, 1
    while True:
        power = round_bits(power * s2, 64)
        nxt = round_bits(total + round_bits(power / (2 * k + 1), 64), 64)
        if nxt == total:
            break
        total, k = nxt, k + 1
    return float(round_bits(2 * round_bits(total, 53), 53))


def kahan_expm1(x: float) -> float:
    u = math.exp(x)
    if u == 1.0:
        return x
    if u - 1.0 == -1.0:
        return -1.0
    return (u - 1.0) * x / math.log(u)


def forms(rate: float, nper: int, pv: float, fv: float, typ: int) -> dict[str, float]:
    big_l = excel_l(rate)
    k = 1 + rate * typ
    q = math.exp(-nper * big_l)
    one_minus_q = -kahan_expm1(-nper * big_l)
    p = math.exp(nper * big_l)
    p_minus_1 = kahan_expm1(nper * big_l)
    return {
        "-(pv+fv*Q)/(k*((1-Q)/r))": -(pv + fv * q) / (k * (one_minus_q / rate)),
        "-(pv+fv*Q)*r/(k*(1-Q))": -(pv + fv * q) * rate / (k * one_minus_q),
        "-(pv*P+fv)*r/(k*(P-1))": -(pv * p + fv) * rate / (k * p_minus_1),
    }


def main() -> None:
    case = json.loads((CASES / "m9-fin-pmt.case.json").read_text())
    oracle = json.loads((CASES / "m9-fin-pmt.oracle.json").read_text())["sheets"]["Calc"]
    cells = next(s for s in case["sheets"] if s["name"] == "Calc")["cells"]
    score: Counter[str] = Counter()
    total = 0
    for a1, text in cells.items():
        if not str(text).startswith("=PMT("):
            continue
        args = [eval(a, {"__builtins__": {}}) for a in text[5:-1].split(",")]  # numeric literals only
        rate, nper, pv = float(args[0]), int(args[1]), float(args[2])
        fv = float(args[3]) if len(args) > 3 else 0.0
        typ = int(args[4]) if len(args) > 4 else 0
        total += 1
        for name, value in forms(rate, nper, pv, fv, typ).items():
            score[name] += value == oracle[a1]["v"]
    for name, hits in score.most_common():
        print(f"{hits}/{total} bit-exact  {name}")


if __name__ == "__main__":
    main()
