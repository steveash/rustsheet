"""T-3269 R4.2 fresh sweep after the rework's freeze (decision 0188 R4.2), filled once.

Written after excel_tail.rs was frozen at shapes 3/2 to 11 (FAR_TAIL_MAX_SHAPE) and 30 < x <= 600.
Log-uniform over the whole far-tail domain the old code covered: shape a log-uniform on [1.5, 19.5]
rounded to a multiple of 1/2, x log-uniform on [20, 600]; plus cells around the cuts: x log-uniform
on [27, 34] at any shape (the x = 30 cut) and x on [30, 45] at shapes 10 to 13 (the shape cut).
Integer shapes alternate POISSON(a - 1, x, TRUE) and CHISQ.DIST.RT(2x, 2a); half-integer shapes are
CHISQ.DIST.RT. Each x has 6 significant digits (2x is then exact in 15). Seeded: re-running writes the
same case, eval/suites/probes-numeric/cases/t3269-fresh-sweep-2.case.json.

Run: python3 -I docs/research/t3269/fresh_sweep_2.py
"""

import math
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from fresh_sweep import write_case  # noqa: E402

CASE = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases/t3269-fresh-sweep-2.case.json"
rng = random.Random(32692)


def log_uniform(lo: float, hi: float) -> float:
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def shape(lo: float, hi: float) -> float:
    return min(max(round(2.0 * log_uniform(lo, hi)) / 2.0, lo), hi)


plan = [(1.5, 19.5, 20.0, 600.0)] * 240 + [(1.5, 19.5, 27.0, 34.0)] * 40 + [(10.0, 13.0, 30.0, 45.0)] * 20
cells, hypotheses = {}, {}
for n, (a_lo, a_hi, x_lo, x_hi) in enumerate(plan, 1):
    a = shape(a_lo, a_hi)
    x = float(f"{log_uniform(x_lo, x_hi):.6g}")
    if a.is_integer() and n % 2:
        formula, label = f"=POISSON({int(a) - 1},{x:g},TRUE)", f"fresh2/poisson/a={a:g}/x={x:g}"
    else:
        formula, label = f"=CHISQ.DIST.RT({2 * x:g},{int(2 * a)})", f"fresh2/chisq/a={a:g}/x={x:g}"
    cells[f"A{n}"] = formula
    hypotheses[f"Calc!A{n}"] = label
write_case(
    CASE,
    "T-3269 fresh log-uniform sweep after the reworked far-tail form was frozen (decision 0188 R4.2): "
    "Q(a, x) by POISSON(a-1,x,TRUE) or CHISQ.DIST.RT(2x,2a), a log-uniform on 1.5..19.5 (multiples of 1/2), "
    "x log-uniform on 20..600, plus 40 cells at x 27..34 and 20 at shapes 10..13, x 30..45 "
    "(300 cells; docs/research/t3269/fresh_sweep_2.py).",
    cells,
    hypotheses,
)
print(len(cells))
