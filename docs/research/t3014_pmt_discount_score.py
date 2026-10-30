"""T-3014: Excel's PMT as a discounted payment, scored against Excel's PMT bits.

Scores PMT candidates on the 308 PMT cells of probes-numeric/m9-fin-pmt (fv 0 with type 0, fv -500 with type 1),
the 371 of probes/t3014-pmt-fv-type-alone (fv alone, type 1 alone, and a |nper*L| sweep around 1) and the 49
constant-argument PMT cells of ironcalc-calc/FINANCIAL/PMT.xlsx. L, Excel's EXP and Kahan's expm1 are T-2951's
(docs/research/t2951_pmt_order_score.py, which evaluates the engine's Excel models through the python bindings via
docs/research/t2464/excelmath.py). Result: financial.md "PMT is a discounted payment".

Run from the repository root after `make py-bindings` (plus `uv pip install --python bindings/python/.venv
mpmath`): bindings/python/.venv/bin/python docs/research/t3014_pmt_discount_score.py
"""

from __future__ import annotations

import json
import sys
from collections.abc import Callable
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import t2951_pmt_order_score as t2951

ALONE = Path("eval/suites/probes/cases/t3014-pmt-fv-type-alone")
EXPM1_TO = 1.0  # annuity.rs PAYMENT_EXPM1_TO

Row = t2951.Row


def one_minus_q(rate: float, nper: float, switch: float) -> float:
    """`1-EXP(-nper*L)`: Kahan's expm1 below |x| = switch, the plain subtraction from it on."""
    x = -nper * t2951.growth_log(rate)
    return -t2951.excel_expm1(x) if abs(x) < switch else 1.0 - t2951.excel_fn("EXP", x)


def model(switch: float = EXPM1_TO, type1: str = "1/(1/r+1)") -> Callable[[Row], float]:
    """`-(((pv+fv)/(1-Q) - fv) * d)`, d = rate for type 0 and the named factor for type 1."""

    def f(row: Row) -> float:
        r, n, pv, fv, t = row
        d = r if not t else (1.0 / (1.0 / r + 1.0) if type1 == "1/(1/r+1)" else r / (1.0 + r))
        return -(((pv + fv) / one_minus_q(r, n, switch) - fv) * d)

    return f


def owed_first(row: Row) -> float:
    """The best owed-first form: `-((pv+fv*Q)/(1-Q) * d)`, Q = EXP(-nper*L)."""
    r, n, pv, fv, t = row
    q = t2951.excel_fn("EXP", -n * t2951.growth_log(r))
    d = r if not t else 1.0 / (1.0 / r + 1.0)
    return -((pv + fv * q) / one_minus_q(r, n, EXPM1_TO) * d)


CANDIDATES: dict[str, Callable[[Row], float]] = {
    "model -(((pv+fv)/(1-Q) - fv) * d)": model(),
    "model, Kahan's expm1 at every |x|": model(switch=float("inf")),
    "model, type 1 as r/(1+r)": model(type1="r/(1+r)"),
    "owed first -((pv+fv*Q)/(1-Q) * d)": owed_first,
    "correctly rounded (the exact payment)": t2951.exact,
}


def alone_cells() -> list[tuple[str, Row, float]]:
    case = json.loads(ALONE.with_suffix(".case.json").read_text())
    oracle = json.loads(ALONE.with_suffix(".oracle.json").read_text())["sheets"]["Sheet1"]
    out = []
    for a1, text in case["sheets"][0]["cells"].items():
        args = [t2951.literal(a) for a in text[5:-1].split(",")] + [0.0, 0.0]
        out.append((a1, (args[0], args[1], args[2], args[3], 1.0 if args[4] else 0.0), oracle[a1]["v"]))
    return out


def main() -> None:
    sets = {"m9-fin-pmt": t2951.m9_cells(), "t3014 probe": alone_cells(), "PMT.xlsx": t2951.pmt_xlsx_cells()}
    print("bit-exact of: " + ", ".join(f"{k} {len(v)}" for k, v in sets.items()))
    for name, fn in CANDIDATES.items():
        scores = "  ".join(f"{sum(fn(row) == want for _, row, want in cells):>3}" for cells in sets.values())
        print(f"{scores}  {name}")


if __name__ == "__main__":
    main()
