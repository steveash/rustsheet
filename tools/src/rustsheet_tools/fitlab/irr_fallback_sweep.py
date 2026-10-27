"""Generator of probes-numeric/t2927-irr-fallback-sweep and its roots TSV (T-2927).

`write_case()` builds the case: IRR flow families, kept only where the fit-lab model of IRR's
secant (`irr_secant_v`, defaults: guess 0.1 and its retry) fails, so Excel takes its fallback path.
After `rustsheet-oracle fill`, `write_roots()` writes each row's exact root (decimal bisection near
Excel's value, 30 digits) and Excel's signed offset from it in ulps beside the case.

  cd tools && uv run python -m rustsheet_tools.fitlab.irr_fallback_sweep case
  cd tools && uv run python -m rustsheet_tools.fitlab.irr_fallback_sweep roots
"""

from __future__ import annotations

import json
import math
import sys
from decimal import Decimal, localcontext
from pathlib import Path

from rustsheet_tools.fitlab.kernels.solvers import NoResultError, irr_secant_v

ROOT = Path(__file__).resolve().parents[4]
CASE = ROOT / "eval/suites/probes-numeric/cases/t2927-irr-fallback-sweep"
# Every tenth row also gets column B, the same flows with guess 0.5.
GUESS_EVERY = 10
BISECTION_STEPS = 200

Flows = list[float]


def families() -> list[tuple[str, Flows]]:
    """The candidate flows, by family name, before the secant filter."""
    base: list[tuple[str, Flows]] = []
    for k in [2, 3, 4, 5, 6, 7, 8, 10, 12, 15, 20, 25, 30, 40, 42, 50, 60, 70, 80, 90, 100]:
        base.append(("k111", [k, 1, 1, -1]))
    for i in range(20):
        n = round(10 * (200 ** (i / 19)))
        base.append(("N11111", [-n, 1, 1, 1, 1, 1]))
    for s in [1e-3, 1e-2, 0.1, 1, 10, 100, 1e3, 1e4, 1e5, 1e6]:
        base.append(("abc", [0.3 * s, -2.5 * s, 1.005 * s]))
    for b in [2.2, 2.3, 2.4, 2.6, 2.8, 3.0, 3.5, 4.0]:
        base.append(("abc-b", [0.3, -b, 1.005]))
    base.append(("gen", [0, 2, -1]))
    out = list(base)
    for family, flows in base:
        out.append((family + "*1e6", [x * 1e6 for x in flows]))
        out.append((family + "*-1", [-x for x in flows]))
    return out


def secant_fails(flows: Flows) -> bool:
    """Whether the fitted secant and its retry both fail on `flows`."""
    try:
        irr_secant_v()([float(x) for x in flows], 0.1, None)
    except (NoResultError, ZeroDivisionError, OverflowError):
        return True
    return False


def excel_number(x: float) -> str:
    """`x` as a formula literal."""
    if float(x).is_integer():
        return str(int(x))
    return repr(float(x)).replace("e+", "E+").replace("e-", "E-")


def kept_rows() -> list[tuple[int, str, Flows]]:
    """The case's rows: (row, family, flows) for every flow whose secant fails."""
    kept = [(family, flows) for family, flows in families() if secant_fails(flows)]
    return [(i, family, flows) for i, (family, flows) in enumerate(kept, 1)]


def write_case() -> None:
    cells: dict[str, str] = {}
    for i, _, flows in kept_rows():
        array = "{" + ",".join(excel_number(x) for x in flows) + "}"
        cells[f"A{i}"] = f"=IRR({array})"
        if i % GUESS_EVERY == 1:
            cells[f"B{i}"] = f"=IRR({array},0.5)"
    description = (
        "IRR flows where the fitted secant (fitlab irr_secant_v defaults: guess 0.1 and its retry) fails, so Excel"
        " takes its fallback path (T-2927). Column A =IRR(flows) with no guess; column B =IRR(flows,0.5) on every"
        " tenth row. Families: {k,1,1,-1} for k in 2..100; {-N,1,1,1,1,1} for N log-spaced 10..2000;"
        " {0.3,-2.5,1.005} scaled by 1E-3..1E+6 and with b in 2.2..4.0; {0,2,-1}; each base flow also scaled by 1E6"
        " and by -1. Only flows whose secant fails are kept."
    )
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M9", "probe"],
        "sheets": [{"name": "P", "cells": cells}],
        "expected": {},
    }
    Path(f"{CASE}.case.json").write_text(json.dumps(case, indent=1), encoding="utf-8")


def npv(flows: Flows, rate: Decimal) -> Decimal:
    """Sum of c_i / (1 + rate)^i, in the current decimal context, on the flows' exact binary values."""
    v = 1 / (1 + rate)
    total, power = Decimal(0), Decimal(1)
    for c in flows:
        total += Decimal(float(c)) * power
        power *= v
    return total


def exact_root(flows: Flows, near: float) -> Decimal | None:
    """The root of the NPV within 1e-9 relative of `near`, by bisection; None with no sign change."""
    x = Decimal(repr(near))
    width = max(abs(x) * Decimal("1e-9"), Decimal("1e-12"))
    lo, hi = x - width, x + width
    f_lo = npv(flows, lo)
    if f_lo * npv(flows, hi) >= 0:
        return None
    for _ in range(BISECTION_STEPS):
        mid = (lo + hi) / 2
        f_mid = npv(flows, mid)
        if (f_mid < 0) == (f_lo < 0):
            lo, f_lo = mid, f_mid
        else:
            hi = mid
    return (lo + hi) / 2


def write_roots() -> None:
    oracle = json.loads(Path(f"{CASE}.oracle.json").read_text(encoding="utf-8"))["sheets"]["P"]
    lines = ["row\tfamily\tflows\texcel\texcel_guess_0.5\texact_root\toffset_ulps"]
    with localcontext() as ctx:
        ctx.prec = 60
        for i, family, flows in kept_rows():
            a = oracle.get(f"A{i}", {})
            b = oracle.get(f"B{i}", {})
            value = a.get("v", a.get("e"))
            guessed = b.get("v", b.get("e", "")) if b else ""
            exact, offset = "", ""
            if isinstance(value, (int, float)):
                root = exact_root(flows, float(value))
                if root is None:
                    exact = "no sign change"
                else:
                    exact = format(root, ".30g")
                    ulp = Decimal(math.ulp(float(root)))
                    offset = format((Decimal(float(value)) - root) / ulp, ".2f")
            row = (i, family, json.dumps(flows), value, guessed, exact, offset)
            lines.append("\t".join(str(t) for t in row))
    Path(f"{CASE}.roots.tsv").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    {"case": write_case, "roots": write_roots}[sys.argv[1]]()


if __name__ == "__main__":
    main()
