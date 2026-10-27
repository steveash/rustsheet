"""Kernel fits shipped with the lab. Add a module here and list its `FITS` in `MODULES`."""

from __future__ import annotations

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.kernels import (
    annuity,
    asin,
    beta,
    beta_inv,
    erfc,
    ftest,
    ftest_adhoc,
    gamma,
    mirr,
    norm_inv,
    power,
    solvers,
    xirr,
    yield_caret,
)

MODULES = [
    erfc,
    annuity,
    power,
    gamma,
    solvers,
    beta,
    norm_inv,
    beta_inv,
    mirr,
    asin,
    yield_caret,
    xirr,
    ftest,
    ftest_adhoc,
]


def registry() -> dict[str, Fit]:
    """Every shipped fit by name."""
    return {fit.name: fit for module in MODULES for fit in module.FITS}
