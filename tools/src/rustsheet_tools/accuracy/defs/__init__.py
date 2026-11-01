"""Gate A definitions by family; importing this package registers every definition."""

from rustsheet_tools.accuracy.defs import annuity, fisher, student_t

__all__ = ["annuity", "fisher", "student_t"]
