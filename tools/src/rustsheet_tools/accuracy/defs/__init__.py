"""Gate A definitions by family; importing this package registers every definition."""

from rustsheet_tools.accuracy.defs import (
    annuity,
    cashflow,
    discrete,
    fisher,
    inverse_trig,
    rounding,
    student_t,
    trig,
)

__all__ = ["annuity", "cashflow", "discrete", "fisher", "inverse_trig", "rounding", "student_t", "trig"]
