"""Gate A definitions by family; importing this package registers every definition."""

from rustsheet_tools.accuracy.defs import (
    annuity,
    cashflow,
    discrete,
    fisher,
    inverse_hyperbolic,
    rounding,
    student_t,
)

__all__ = ["annuity", "cashflow", "discrete", "fisher", "inverse_hyperbolic", "rounding", "student_t"]
