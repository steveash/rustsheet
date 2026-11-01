"""Validation of a case's `sheet_ops` (T-1608, T-1933, T-2323): the ops spec2xlsx records and the
oracle and engine backends replay. Their spec keys are described in spec2xlsx's module docstring."""

from __future__ import annotations

import re
from typing import Any

SHEET_OPS = (
    "add",
    "rename",
    "move",
    "insert_row",
    "delete_row",
    "insert_column",
    "delete_column",
    "delete_sheet",
    "convert_table",
    "delete_table",
)
# The spec key holding each op's argument; the ops not listed (add, delete_sheet) take none.
_OP_KEYS = {
    "rename": "to",
    "move": "to",
    "insert_row": "row",
    "delete_row": "row",
    "insert_column": "column",
    "delete_column": "column",
    "convert_table": "table",
    "delete_table": "table",
}
_COLUMN_LETTERS = re.compile(r"[A-Za-z]{1,3}")
_ROW_SPAN = re.compile(r"([1-9][0-9]*):([1-9][0-9]*)")
_LAST_COLUMN = 16384  # XFD


def _op_key(kind: object) -> str | None:
    """The spec key of a sheet op's argument (`to`, `row`, `column` or `table`), None for an op
    that takes none."""
    return _OP_KEYS.get(kind) if isinstance(kind, str) else None


# scripts/exit_owners.py imports this from outside the uv project: keep this module stdlib-only.
def column_number(letters: str) -> int:
    """The 1-based column number of column letters (`B` is 2); letters are A-Z in either case."""
    number = 0
    for ch in letters.upper():
        number = number * 26 + ord(ch) - ord("A") + 1
    return number


def _is_column(value: object) -> bool:
    """Whether `value` is a column letter within Excel's XFD."""
    return (
        isinstance(value, str)
        and _COLUMN_LETTERS.fullmatch(value) is not None
        and column_number(value) <= _LAST_COLUMN
    )


def _is_int_at_least(value: object, minimum: int) -> bool:
    """Whether `value` is an int (not a bool) of at least `minimum`."""
    return isinstance(value, int) and not isinstance(value, bool) and value >= minimum


def _is_name(value: object) -> bool:
    """Whether `value` is a non-empty string."""
    return isinstance(value, str) and bool(value)


def _is_row_span(value: object) -> bool:
    """Whether `value` is the `first:last` rows of a multi-row delete_row, first before last (T-3019)."""
    match = _ROW_SPAN.fullmatch(value) if isinstance(value, str) else None
    return match is not None and int(match[1]) < int(match[2])


def _valid_op_argument(kind: str, to: object) -> bool:
    """Whether `to` is a valid argument for sheet op `kind`."""
    return {
        "rename": _is_name(to),
        "move": _is_int_at_least(to, 0),
        "insert_row": _is_int_at_least(to, 1),
        "delete_row": _is_int_at_least(to, 1) or _is_row_span(to),
        "insert_column": _is_column(to),
        "delete_column": _is_column(to),
        "convert_table": _is_name(to),
        "delete_table": _is_name(to),
    }.get(kind, to is None)


def sheet_op(op: Any) -> tuple[str, str, str | int | None]:
    """A spec's sheet op as `(op, sheet, to)`: `to` is the new name of a rename, the 0-based
    position of a move, the 1-based row of an insert_row or delete_row (spec key `row`; the
    blank row goes above it, T-1933; a delete_row with `count` n > 1 gives the span `row:row+n-1`,
    T-3019), the column letter of an insert_column or delete_column
    (spec key `column`), the table name of a convert_table or delete_table (spec key `table`;
    T-2323) and None for an add or delete_sheet. Raises ValueError for an unknown or malformed op."""
    recorded = isinstance(op, (list, tuple)) and len(op) == 3
    if recorded:
        key = _op_key(op[0])
        op = {"op": op[0], "sheet": op[1], **({} if op[2] is None or key is None else {key: op[2]})}
    if not isinstance(op, dict) or op.get("op") not in SHEET_OPS or not isinstance(op.get("sheet"), str):
        msg = f"unknown sheet op {op!r}: expected op {'/'.join(SHEET_OPS)} with a sheet name"
        raise ValueError(msg)
    kind, sheet = op["op"], op["sheet"]
    key = _op_key(kind)
    to = op.get(key) if key else None
    if set(op) - {"op", "sheet", key, *(["count"] if kind == "delete_row" else [])}:
        msg = f"sheet op {op!r} has unknown keys"
        raise ValueError(msg)
    if "count" in op:
        count = op["count"]
        if not (_is_int_at_least(count, 1) and _is_int_at_least(to, 1)):
            msg = f"sheet op {op!r}: `count` must be a row count >= 1 after a 1-based `row`"
            raise ValueError(msg)
        # One edit of several rows is recorded as their span, `4:7`, which is also Graph's address;
        # a count of 1 keeps the plain row, so single-row workbooks stay byte-identical.
        if count > 1:
            to = f"{to}:{to + count - 1}"
    elif _is_row_span(to) and not recorded:
        msg = f"sheet op {op!r}: a spec deletes several rows with `row` and `count`, not a span"
        raise ValueError(msg)
    if _valid_op_argument(kind, to):
        return kind, sheet, to
    msg = (
        f"sheet op {op!r}: `to` must be absent for add and delete_sheet, a name for rename, a position"
        " >= 0 for move; `row` a 1-based row for insert_row and delete_row (with an optional row `count`"
        " for delete_row); `column` a column letter"
        " (A to XFD) for insert_column and delete_column; `table` a table name for convert_table and"
        " delete_table"
    )
    raise ValueError(msg)
