"""Gate A definitions by family; importing this package registers every definition."""

from rustsheet_tools.accuracy.defs import annuity, cashflow, fisher, student_t

__all__ = ["annuity", "cashflow", "fisher", "student_t"]
