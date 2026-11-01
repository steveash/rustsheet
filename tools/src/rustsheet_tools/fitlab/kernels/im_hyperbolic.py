"""IMSINH, IMCOSH, IMSIN and IMCOS: the complex hyperbolic selectors of complex/trig.rs (T-3525).

The engine picks its sinh and cosh by |x| (`INVERSE_EXP_LOW` < |x| <= `INVERSE_EXP_HIGH`: the
`1/u` form, libm or the EXP pair outside) and takes IMSINH and IMCOSH in log space past
`HYPERBOLIC_LOG_CUT`. Decision 0188 R4 asks whether those tuned cut-points generalize: the
candidates here are engine builds (`rustsheet-fitlab im-hyperbolic --build NAME=PATH`, one build
with the selector, one with the accurate form only) and the correctly rounded reference, scored part
by part on the IM* text (`Fit.complex_text`).

- `im-hyperbolic`: the old cells the selectors were fitted on (T-2231, T-2535, T-2629, T-2778).
- `im-hyperbolic-fresh`: the fresh log-uniform sweep no fit saw
  (probes-numeric/t3525-im-hyperbolic-fresh-sweep, docs/research/t3525/gen_sweep.py).
- `im-small-imag`: IMSIN and IMCOS with |y| <= 0.0105, drawn after T-3545 froze their small-y kernel
  (probes-numeric/t3545-imsin-imcos-small-imag-sweep, docs/research/t3545/gen_sweep.py).
- `im-hyperbolic-t3589-fresh`: IMSINH and IMCOSH at |x| <= 0.01 and IMSIN and IMCOS at |y| in [20, 709.7], drawn
  after T-3589 froze the series sinh and the reciprocal form past 20 (docs/research/t3589/gen_sweep.py; its
  IMSECH and IMCSCH cells fall outside the pattern and are scored by the eval).
- `im-hyperbolic-log`: the log form's cells (|x| near 708-710.5) T-3546 chose Excel's EXP on, and
  `im-hyperbolic-log-fresh`, its sweep drawn after that choice (docs/research/t3546/libm-exp.patch
  makes the libm build).
- `im-near-half-pi`: IMSIN and IMCOS with x near k*pi/2, drawn after T-3726 froze sin x and cos x as SIN and COS's
  x87 reduction (docs/research/t3726/gen_sweep.py), with the T-3616 probe whose K1, L1, K22 and L22 found it.
"""

from __future__ import annotations

import re
import sys

import mpmath

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.holdout import REFERENCE
from rustsheet_tools.fitlab.score import Candidate, ExcelError
from rustsheet_tools.fitlab.sweep import Cell, parse_complex_text

_FUNCTION = re.compile(r"^=(IMSINH|IMCOSH|IMSIN|IMCOS)\(")
_KERNELS = {"IMSINH": mpmath.sinh, "IMCOSH": mpmath.cosh, "IMSIN": mpmath.sin, "IMCOS": mpmath.cos}
_DBL_MAX = sys.float_info.max
_DBL_MIN_NORMAL = sys.float_info.min
PATTERN = r"^=IM(?:SINH|COSH|SIN|COS)\((.*)\)$"


def _part(v: mpmath.mpf) -> float:
    """One exact part as the IM* text writes it: the nearest double at 15 significant digits, a
    subnormal part 0 (complex/parse.rs `Complex` Display); `#NUM!` past the double range."""
    d = float(v)
    if not abs(d) <= _DBL_MAX:  # beyond DBL_MAX, or NaN
        raise ExcelError(code="#NUM!")
    if abs(d) < _DBL_MIN_NORMAL:
        return 0.0
    rounded = float(f"{d:.15g}")
    return d if abs(rounded) > _DBL_MAX else rounded


def correctly_rounded(cell: Cell) -> complex:
    """The 50-digit value of the cell's function, each part rounded as the IM* text rounds it."""
    m = _FUNCTION.match(cell.formula)
    arg = cell.args[0] if cell.args else None
    if m is None or not isinstance(arg, str):
        msg = f"not a one-argument IM* call: {cell.formula}"
        raise TypeError(msg)
    x, y = parse_complex_text(arg)
    with mpmath.workdps(50):
        z = _KERNELS[m[1]](mpmath.mpc(x, y))
        return complex(_part(z.real), _part(z.imag))


def candidates() -> list[Candidate]:
    return [Candidate(REFERENCE, correctly_rounded)]


_NOTES = [
    "Engine builds are the selector's candidates: --build NAME=PATH; docs/research/t3525/*.patch make them.",
]


def _fit(name: str, description: str, cases: list[str]) -> Fit:
    """One IM* fit: the shared pattern, reference, error cells and complex-text scoring."""
    return Fit(
        name=name,
        description=description,
        cases=cases,
        pattern=PATTERN,
        candidates=candidates,
        errors=True,
        complex_text=True,
        notes=_NOTES,
    )


FITS = [
    _fit(
        "im-hyperbolic",
        "IMSINH/IMCOSH/IMSIN/IMCOS on the cells the T-2231/T-2629 selectors were fitted on",
        [
            "probes/eng-im-overflow-edges",
            "probes/eng-im-overflow-edges-2",
            "probes/t2535-imsinh-imcosh-kernel-sweep",
            "probes/t2629-imcosh-imsinh-y-sweep",
            "probes/t2778-imsinh-sign-sweep",
        ],
    ),
    _fit(
        "im-hyperbolic-fresh",
        "IMSINH/IMCOSH/IMSIN/IMCOS on T-3525's fresh log-uniform sweep (decision 0188 R4.2)",
        [
            "probes-numeric/t3525-im-hyperbolic-fresh-sweep",
            "probes-numeric/t3525-im-hyperbolic-log-band-sweep",
        ],
    ),
    _fit(
        "im-small-imag",
        "IMSIN/IMCOS with a small imaginary part, T-3545's sweep drawn after the freeze",
        [
            "probes-numeric/t3545-imsin-imcos-small-imag-sweep",
            "probes-numeric/t3545-imsin-imcos-tiny-imag-sweep",
        ],
    ),
    _fit(
        "im-hyperbolic-t3589-fresh",
        "IMSINH/IMCOSH at |x| <= 0.01, IMSIN/IMCOS at |y| >= 20: T-3589's sweeps drawn after the freeze",
        [
            "probes-numeric/t3589-im-hyperbolic-small-x-sweep",
            "probes-numeric/t3589-imsin-imcos-large-imag-sweep",
        ],
    ),
    _fit(
        "im-hyperbolic-log",
        "IMSINH/IMCOSH in log space: the cells T-3546 chose Excel's EXP on (cut and band probes)",
        [
            "probes/eng-im-overflow-edges",
            "probes/eng-im-overflow-edges-2",
            "probes-numeric/t3525-im-hyperbolic-log-band-sweep",
            "probes-numeric/t3546-imsinh-neg-cut-coarse",
            "probes-numeric/t3546-imsinh-neg-cut-fine",
        ],
    ),
    _fit(
        "im-hyperbolic-log-fresh",
        "IMSINH/IMCOSH in log space on T-3546's sweep drawn after choosing Excel's EXP (decision 0188 R4.2)",
        ["probes-numeric/t3546-im-hyperbolic-log-exp-fresh-sweep"],
    ),
    _fit(
        "im-near-half-pi",
        "IMSIN/IMCOS with x near k*pi/2: T-3726's sweep drawn after the x87 reduction was frozen, and the T-3616 probe",
        [
            "probes-numeric/t3726-imsin-imcos-near-half-pi-sweep",
            "probes-numeric/t3616-imsin-imcos-positive-cut-probe",
        ],
    ),
]
