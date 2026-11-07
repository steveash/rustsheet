"""GAMMA.DIST and the chi-squared family at a huge finite x / beta: where Excel is `#NUM!` (T-4635).

At a finite y = x / beta from 1E200 up, Excel's GAMMA.DIST is 1 (cdf) or 0 (density) or `#NUM!`;
CHISQ.DIST, CHISQ.DIST.RT and CHIDIST at shape df / 2 and y = x / 2 share the cut. Each candidate is
a model of where the `#NUM!` starts, scored on the cells of the T-4635 probes and T-4625's ladder
(docs/research/t4635, decision 0188 R4.1's hash hold-out split):

- `correctly rounded`: no `#NUM!` at a finite y (the value the math gives, 1 or 0);
- `4/a` (rule R, gamma.rs `huge_ratio_num_error`): above shape 1, `#NUM!` where y / a * 4
  overflows, and up to shape 1.1 also where y^a does; `4/a no power` drops the second branch;
- the rivals T-4635's second fill ranked against it, each with the same power branch: `4-a`
  (y * (4 - a) overflows), `a/(a-1)` (y * a / (a - 1)) and `exp(k(4-a))` (y * exp(k (4 - a)),
  k = ln 2 / 2 so that it meets 4 / a at shape 2).

A number cell outside y >= 1E200 is not scored: there the value is not 1 or 0 and no candidate
models it. Results: T-4635's Outcome.
"""

from __future__ import annotations

import math
from collections.abc import Callable

from rustsheet_tools.accuracy.defs.gamma_dist import HUGE_RATIO_POWER_SHAPE_MAX, power_overflows
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.holdout import REFERENCE
from rustsheet_tools.fitlab.score import Candidate, ExcelError
from rustsheet_tools.fitlab.sweep import Cell, numeric_arg

_CASES = [
    "probes/t4625-gamma-dist-ratio-overflow",
    "probes/t4635-gamma-dist-huge-ratio",
    "probes/t4635-gamma-dist-huge-ratio-cut",
    "probes/t4635-gamma-dist-huge-ratio-fresh",
    "probes/t4635-chisq-huge-x",
]
_ARG = r"([^,()]+)"
_GAMMA_PATTERN = rf"^=(?:GAMMA\.DIST|GAMMADIST)\({_ARG},{_ARG},{_ARG},{_ARG}\)$"
_CHISQ_PATTERN = rf"^=CHISQ\.DIST\({_ARG},{_ARG},{_ARG}\)$"
_CHISQ_RT_PATTERN = rf"^=(?:CHISQ\.DIST\.RT|CHIDIST)\({_ARG},{_ARG}\)$"
_Y_MIN = 1e200  # the smallest y scored: from here the value is 1 or 0 (probes/t4625 at shape 2)
_K = math.log(2) / 2

# A model: (shape, y) -> whether the cell is `#NUM!`.
Model = Callable[[float, float], bool]


def _quotient(factor: Callable[[float], float], power: bool) -> Model:
    """`#NUM!` above shape 1 where y * factor(a) overflows (or, to shape 1.1, y^a when `power`)."""

    def model(a: float, y: float) -> bool:
        return a > 1.0 and (
            math.isinf(y * factor(a)) or (power and a <= HUGE_RATIO_POWER_SHAPE_MAX and power_overflows(y, a))
        )

    return model


def rule_r(a: float, y: float) -> bool:
    """Rule R in the engine's operation order: y / a * 4, then the power branch."""
    return a > 1.0 and (
        math.isinf(y / a * 4.0) or (a <= HUGE_RATIO_POWER_SHAPE_MAX and power_overflows(y, a))
    )


def _models() -> dict[str, Model]:
    return {
        REFERENCE: lambda _a, _y: False,
        "4/a": rule_r,
        "4/a no power": lambda a, y: a > 1.0 and math.isinf(y / a * 4.0),
        "4-a": _quotient(lambda a: 4.0 - a, power=True),
        "a/(a-1)": _quotient(lambda a: a / (a - 1.0), power=True),
        "exp(k(4-a))": _quotient(lambda a: math.exp(_K * (4.0 - a)), power=True),
    }


def _shape_y_value(name: str, cell: Cell) -> tuple[float, float, float]:
    """(shape, y, Excel's value where no `#NUM!`) of one cell of fit `name`."""
    args = cell.args
    x = numeric_arg(args[0])
    if name == "gamma":
        cumulative = args[3]
        return numeric_arg(args[1]), x / numeric_arg(args[2]), 1.0 if cumulative is True else 0.0
    df = numeric_arg(args[1])
    if name == "chisq":
        return df / 2.0, x / 2.0, 1.0 if args[2] is True else 0.0
    return df / 2.0, x / 2.0, 0.0  # the right tail


def _candidates(name: str) -> Callable[[], list[Candidate]]:
    def build() -> list[Candidate]:
        out = []
        for label, model in _models().items():

            def run(cell: Cell, model: Model = model) -> float:
                a, y, value = _shape_y_value(name, cell)
                if model(a, y):
                    raise ExcelError
                return value

            out.append(Candidate(label, run, label, None))
        return out

    return build


def _where(name: str) -> Callable[[Cell], bool]:
    def keep(cell: Cell) -> bool:
        try:
            _a, y, _v = _shape_y_value(name, cell)
        except TypeError:
            return False
        return math.isfinite(y) and y >= _Y_MIN

    return keep


FITS = [
    Fit(
        name=f"huge-ratio-{name}",
        description=f"{label} at a huge finite y (T-4635): where the #NUM! starts",
        cases=_CASES,
        pattern=pattern,
        candidates=_candidates(name),
        where=_where(name),
        errors=True,
    )
    for name, label, pattern in (
        ("gamma", "GAMMA.DIST/GAMMADIST", _GAMMA_PATTERN),
        ("chisq", "CHISQ.DIST", _CHISQ_PATTERN),
        ("chisq-rt", "CHISQ.DIST.RT/CHIDIST", _CHISQ_RT_PATTERN),
    )
]
