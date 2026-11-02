"""Gate A definitions by family; importing this package registers every definition."""

from rustsheet_tools.accuracy.defs import (
    annuity,
    cashflow,
    ceiling_floor,
    discrete,
    fisher,
    rounding,
    student_t,
    trig,
)

__all__ = ["annuity", "cashflow", "ceiling_floor", "discrete", "fisher", "rounding", "student_t", "trig"]
