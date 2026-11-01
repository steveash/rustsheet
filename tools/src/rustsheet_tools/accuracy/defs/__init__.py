"""Gate A definitions by family; importing this package registers every definition."""

from rustsheet_tools.accuracy.defs import (
    aggregate,
    annuity,
    cashflow,
    discrete,
    fisher,
    rounding,
    student_t,
    trig,
)

__all__ = ["aggregate", "annuity", "cashflow", "discrete", "fisher", "rounding", "student_t", "trig"]
