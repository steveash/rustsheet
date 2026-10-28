"""T-2951: PMT operation orders and Excel's own discount, scored against Excel's PMT bits.

Scores named PMT candidates on three cell sets: the nine PMT inputs T-2951 owns in
ironcalc-calc/FINANCIAL/PMT.xlsx (rates -0.9 and 0.2), the other PMT cells of PMT.xlsx with constant
arguments (both are Excel's cached values, Mac desktop) and the 308 PMT cells of probes-numeric/m9-fin-pmt
(excel-graph). EXP and LN are the engine's bit-exact models of Excel's, evaluated through the python
bindings (docs/research/t2464/excelmath.py); L is the PPMT discount's log (financial.md "PPMT discounts
with an atanh-series log": the x87 atanh series below |rate| 0.375, Excel's LN of 1+rate from it on) and
1-Q is Excel's Kahan expm1 (numbers.md "Excel's expm1 is Kahan's formula"). Results: financial.md
"Annuity evaluation order and `(1+r)^n`" and T-2951's "Blocked because".

Run from the repository root after `make py-bindings` (plus `uv pip install --python bindings/python/.venv
mpmath`): bindings/python/.venv/bin/python docs/research/t2951_pmt_order_score.py
"""

from __future__ import annotations

import json
import math
import subprocess
import sys
from collections.abc import Callable
from functools import cache
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "t2464"))
import mpmath  # noqa: E402
from excelmath import excel, fl53, fl64  # noqa: E402

PMT_XLSX = "crates/rustsheet-xlsx/tests/calc_tests/FINANCIAL/PMT.xlsx"
M9 = Path("eval/suites/probes-numeric/cases/m9-fin-pmt")
OWNED = ("F10", "F11", "F18", "F23", "F27", "G27", "F32", "F33", "G33")
SERIES_TO = 0.375  # growth_log's GROWTH_LOG_SERIES_TO

Row = tuple[float, float, float, float, float]  # rate, nper, pv, fv, type (0 or 1)


@cache
def excel_fn(name: str, x: float) -> float:
    return excel(name, [x])[0]


def growth_log(rate: float) -> float:
    """annuity.rs `growth_log`: the atanh series with s^2, the power, each term and partial sum at 64 bits."""
    if abs(rate) >= SERIES_TO:
        return excel_fn("LN", 1.0 + rate)
    s = fl64(mpmath.mpf(rate / (2.0 + rate)))
    square, power, total = fl64(s * s), s, s
    for k in range(1, 65):
        power = fl64(power * square)
        nxt = fl64(total + fl64(power / (2 * k + 1)))
        if nxt == total:
            break
        total = nxt
    return 2.0 * fl53(total)


def excel_expm1(x: float) -> float:
    """x87_exp.rs `excel_expm1`: Kahan's (u-1)*x/ln(u) on Excel's EXP and LN, each step a double."""
    u = excel_fn("EXP", x)
    if u == 1.0:
        return x
    if u - 1.0 == -1.0:
        return -1.0
    return (u - 1.0) * x / excel_fn("LN", u)


def power_sqm(rate: float, nper: float) -> float:
    """Excel's `^` on Excel's 1+rate: right-to-left square-and-multiply for a positive integer nper."""
    base = 1.0 + rate
    if nper != int(nper) or nper < 0:
        return float(base**nper)
    out, e = 1.0, int(nper)
    while e:
        if e & 1:
            out *= base
        e >>= 1
        if e:
            base *= base
    return out


def textbook(k_after: bool, fv_first: bool) -> Callable[[Row], float]:
    """`-(pv*P+fv)*r/(k*(P-1))` on Excel's `^`, k in the denominator or divided out last; fv-first is
    the PV-fitted annuity factor `-(fv+pv*P)/(k*((P-1)/r))`."""

    def f(row: Row) -> float:
        r, n, pv, fv, t = row
        p = power_sqm(r, n)
        k = 1.0 + r * t
        if fv_first:
            return -(fv + pv * p) / ((p - 1.0) / r) / k if k_after else -(fv + pv * p) / (k * ((p - 1.0) / r))
        return -((pv * p + fv) * r / (p - 1.0)) / k if k_after else -(pv * p + fv) * r / (k * (p - 1.0))

    return f


def split(row: Row) -> float:
    """The fv*r/((1+r)^n-1) split: `-(pv*r + (pv+fv)*r/(P-1))/k`."""
    r, n, pv, fv, t = row
    p = power_sqm(r, n)
    return -(pv * r + (pv + fv) * r / (p - 1.0)) / (1.0 + r * t)


def discount(k_after: bool, exact_quotient: bool = False) -> Callable[[Row], float]:
    """T-2730/T-3014's Excel-L/EXP form `-(pv+fv*Q)/(k*((1-Q)/r))`, Q = EXP(-n*L), 1-Q = -expm1(-n*L);
    `exact_quotient` rounds `(pv+fv*Q)*r/((1-Q)*k)` once from the double Q and 1-Q."""

    def f(row: Row) -> float:
        r, n, pv, fv, t = row
        big_l = growth_log(r)
        q = excel_fn("EXP", -n * big_l)
        one_minus_q = -excel_expm1(-n * big_l)
        k = 1.0 + r * t
        if exact_quotient:
            mp = mpmath.mpf
            return -fl53((mp(pv) + mp(fv) * mp(q)) * mp(r) / (mp(one_minus_q) * mp(k)))
        owed = pv + fv * q
        return -owed / (one_minus_q / r) / k if k_after else -owed / (k * (one_minus_q / r))

    return f


def exact(row: Row) -> float:
    """The correctly rounded payment of the double arguments (the engine's `accurate_payment`)."""
    r, n, pv, fv, t = (mpmath.mpf(v) for v in row)
    p = (1 + r) ** n
    return fl53(-(fv + pv * p) * r / ((1 + r * t) * (p - 1)))


CANDIDATES: dict[str, Callable[[Row], float]] = {
    "correctly rounded (engine)": exact,
    "textbook -(pv*P+fv)*r/(k*(P-1))": textbook(k_after=False, fv_first=False),
    "textbook, k after: -((pv*P+fv)*r/(P-1))/k": textbook(k_after=True, fv_first=False),
    "fv-first -(fv+pv*P)/(k*((P-1)/r))": textbook(k_after=False, fv_first=True),
    "fv-first, k after: -(fv+pv*P)/((P-1)/r)/k": textbook(k_after=True, fv_first=True),
    "split -(pv*r+(pv+fv)*r/(P-1))/k": split,
    "Excel L/EXP -(pv+fv*Q)/(k*((1-Q)/r))": discount(k_after=False),
    "Excel L/EXP, k after: -(pv+fv*Q)/((1-Q)/r)/k": discount(k_after=True),
    "Excel L/EXP, one rounding: -(pv+fv*Q)*r/((1-Q)*k)": discount(k_after=False, exact_quotient=True),
}


def pmt_xlsx_cells() -> list[tuple[str, Row, float]]:
    """PMT.xlsx's PMT cells over constant cells (rate 0 and the overflow forms excluded), via the harness's
    snapshot (tools never parse xlsx values themselves)."""
    shown = subprocess.run(
        ["target/release/rustsheet-eval", "snapshot", PMT_XLSX], check=True, capture_output=True, text=True
    ).stdout
    cells = json.loads(shown)["sheets"][0]["cells"]
    out = []
    for a1, cell in cells.items():
        formula, value = cell["formula"], cell["value"]
        if not formula or not formula.startswith("PMT(") or value["t"] != "n":
            continue
        refs = formula[4:-1].split(",")
        if any(cells.get(ref, {}).get("formula") for ref in refs):
            continue
        args = [float(cells[ref]["value"]["v"]) if ref in cells else 0.0 for ref in refs]
        args += [0.0] * (5 - len(args))
        args[4] = 1.0 if args[4] else 0.0
        if args[0] != 0.0:
            out.append((a1, (args[0], args[1], args[2], args[3], args[4]), float(value["v"])))
    return out


def literal(arg: str) -> float:
    num, _, den = arg.partition("/")
    return float(num) / float(den) if den else float(num)


def m9_cells() -> list[tuple[str, Row, float]]:
    case = json.loads(M9.with_suffix(".case.json").read_text())
    oracle = json.loads(M9.with_suffix(".oracle.json").read_text())["sheets"]["Calc"]
    cells = next(s for s in case["sheets"] if s["name"] == "Calc")["cells"]
    out = []
    for a1, text in cells.items():
        if str(text).startswith("=PMT("):
            args = [literal(a) for a in text[5:-1].split(",")] + [0.0, 0.0]
            out.append((a1, (args[0], args[1], args[2], args[3], 1.0 if args[4] else 0.0), oracle[a1]["v"]))
    return out


def ulps(got: float, want: float) -> float:
    return (got - want) / math.ulp(want)


def main() -> None:
    xlsx = pmt_xlsx_cells()
    sets = {
        "owned": [c for c in xlsx if c[0] in OWNED],
        "PMT.xlsx other": [c for c in xlsx if c[0] not in OWNED],
        "m9-fin-pmt": m9_cells(),
    }
    print("bit-exact (within 4 ulp) of: " + ", ".join(f"{k} {len(v)}" for k, v in sets.items()))
    for name, fn in CANDIDATES.items():
        scores = []
        for cells in sets.values():
            off = [abs(ulps(fn(row), want)) for _, row, want in cells]
            scores.append(f"{sum(d == 0 for d in off):>3} ({sum(d <= 4 for d in off):>3})")
        owned = " ".join(f"{a1}{ulps(fn(row), want):+.0f}" for a1, row, want in sets["owned"])
        print(f"{'  '.join(scores)}  {name}  [{owned}]")


if __name__ == "__main__":
    main()
