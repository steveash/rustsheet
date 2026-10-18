"""MIRR over the T-1687 sweep (probes-numeric/m9-fin-mirr) and T-1009's MIRR cells (m9-fin-irr-rate).

MIRR(values, fr, rr) is `(FV / -PV)^(1/(n-1)) - 1`, with FV the positive flows compounded at rr to
period n-1 and PV the negative flows discounted at fr to period 0. Every candidate is one shape for
that ratio combined with one root:

- shapes: Microsoft's documented `-NPV(rr, pos) * (1+rr)^n / (NPV(fr, neg) * (1+fr))` with libm
  powers (the engine before T-1687); FV compounded forward and PV discounted directly, libm powers;
  both sums at period 0 with running powers `(1+r)^i = (1+r)^(i-1) * (1+r)`, the positive sum then
  taken forward as `P * (1+rr)^(n-1)` or as `P * (1+rr)^n / (1+rr)`;
- roots: libm `pow`, `exp(ln q / (n-1))`, and Excel's `^` (`power.excel_model`).

The sweep's two-flow 6x6 rate grids separate the shapes: the root is exact there (exponent 1), and
only `P * (1+rr)^n / (1+rr)` reproduces which reinvest rates move the last digit. Results:
docs/excel-semantics/financial.md, "MIRR's evaluation order", and T-1687's Outcome.
"""

from __future__ import annotations

import math
from collections.abc import Callable
from fractions import Fraction

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.kernels.power import excel_model
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg, numeric_arg

Ratio = Callable[[list[float], float, float], float]
Root = Callable[[float, float], float]


def _flows(values: Arg) -> list[float]:
    if not isinstance(values, list):
        msg = f"values {values!r} are not a list"
        raise TypeError(msg)
    return [numeric_arg(v) for v in values]


def _running_power(base: float, k: int) -> float:
    """base^k as k successive products from 1, the running factor of a period loop."""
    p = 1.0
    for _ in range(k):
        p *= base
    return p


def ratio_npv(flows: list[float], fr: float, rr: float) -> float:
    """-NPV(rr, pos) * (1+rr)^n / (NPV(fr, neg) * (1+fr)), libm powers."""
    n = len(flows)
    pos = sum(x / (1 + rr) ** (i + 1) for i, x in enumerate(flows) if x > 0)
    neg = sum(x / (1 + fr) ** (i + 1) for i, x in enumerate(flows) if x < 0)
    return (-pos * (1 + rr) ** n) / (neg * (1 + fr))


def ratio_direct(flows: list[float], fr: float, rr: float) -> float:
    """sum x (1+rr)^(n-1-i) over the positives / -sum x / (1+fr)^i over the negatives, libm powers."""
    n = len(flows)
    fv = sum(x * (1 + rr) ** (n - 1 - i) for i, x in enumerate(flows) if x > 0)
    pv = sum(x / (1 + fr) ** i for i, x in enumerate(flows) if x < 0)
    return fv / -pv


def _period0_sums(flows: list[float], fr: float, rr: float) -> tuple[float, float]:
    pos = neg = 0.0
    for i, x in enumerate(flows):
        if x > 0:
            pos += x / _running_power(1 + rr, i)
        elif x < 0:
            neg += x / _running_power(1 + fr, i)
    return pos, neg


def ratio_period0_n1(flows: list[float], fr: float, rr: float) -> float:
    """Period-0 sums (running powers), FV = P * (1+rr)^(n-1)."""
    pos, neg = _period0_sums(flows, fr, rr)
    return pos * _running_power(1 + rr, len(flows) - 1) / -neg


def ratio_period0_n_over(flows: list[float], fr: float, rr: float) -> float:
    """Period-0 sums (running powers), FV = P * (1+rr)^n / (1+rr): the engine since T-1687."""
    pos, neg = _period0_sums(flows, fr, rr)
    return (pos * _running_power(1 + rr, len(flows)) / (1 + rr)) / -neg


def root_libm(q: float, e: float) -> float:
    return float(q**e)


def root_exp_ln(q: float, e: float) -> float:
    return math.exp(math.log(q) * e)


def root_excel(q: float, e: float) -> float:
    return excel_model(Fraction(q), e)


SHAPES: dict[str, Ratio] = {
    "npv": ratio_npv,
    "direct": ratio_direct,
    "period0 (1+rr)^(n-1)": ratio_period0_n1,
    "period0 (1+rr)^n/(1+rr)": ratio_period0_n_over,
}
ROOTS: dict[str, Root] = {"libm pow": root_libm, "exp ln": root_exp_ln, "Excel ^": root_excel}


def mirr(ratio: Ratio, root: Root) -> Callable[[Arg, Arg, Arg], float]:
    def f(values: Arg, fr: Arg, rr: Arg) -> float:
        flows = _flows(values)
        q = ratio(flows, numeric_arg(fr), numeric_arg(rr))
        # One period has exponent 1, which every root returns exactly.
        e = 1.0 / (len(flows) - 1)
        return (q if len(flows) == 2 else root(q, e)) - 1.0

    return f


def mirr_candidates() -> list[Candidate]:
    return [
        candidate(f"{shape} / {root}", mirr(ratio, fn), family=shape)
        for shape, ratio in SHAPES.items()
        for root, fn in ROOTS.items()
    ]


FITS = [
    Fit(
        name="mirr",
        description="MIRR over m9-fin-mirr and m9-fin-irr-rate (T-1687): ratio shapes x roots",
        cases=["probes-numeric/m9-fin-mirr", "probes-numeric/m9-fin-irr-rate"],
        pattern=r"^=MIRR\((.*)\)$",
        candidates=mirr_candidates,
    ),
]
