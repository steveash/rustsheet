"""Kernel fits shipped with the lab. Add a module here and list its `FITS` in `MODULES`."""

from __future__ import annotations

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.kernels import annuity, beta, beta_inv, erfc, gamma, power, solvers

MODULES = [erfc, annuity, power, gamma, solvers, beta, beta_inv]


def registry() -> dict[str, Fit]:
    """Every shipped fit by name."""
    return {fit.name: fit for module in MODULES for fit in module.FITS}
