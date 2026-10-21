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
_LAST_COLUMN = 16384  # XFD


def _op_key(kind: object) -> str | None:
    """The spec key of a sheet op's argument (`to`, `row`, `column` or `table`), None for an op
    that takes none."""
    return _OP_KEYS.get(kind) if isinstance(kind, str) else None


def _is_column(value: object) -> bool:
    """Whether `value` is a column letter within Excel's XFD."""
    if not isinstance(value, str) or not _COLUMN_LETTERS.fullmatch(value):
        return False
    index = 0
    for ch in value.upper():
        index = index * 26 + ord(ch) - ord("A") + 1
    return index <= _LAST_COLUMN


def _valid_op_argument(kind: str, to: object) -> bool:
    """Whether `to` is a valid argument for sheet op `kind`."""
    is_int = isinstance(to, int) and not isinstance(to, bool)
    return {
        "rename": isinstance(to, str) and bool(to),
        "move": is_int and to >= 0,  # type: ignore[operator]
        "insert_row": is_int and to >= 1,  # type: ignore[operator]
        "delete_row": is_int and to >= 1,  # type: ignore[operator]
        "insert_column": _is_column(to),
        "delete_column": _is_column(to),
        "convert_table": isinstance(to, str) and bool(to),
        "delete_table": isinstance(to, str) and bool(to),
    }.get(kind, to is None)


def sheet_op(op: Any) -> tuple[str, str, str | int | None]:
    """A spec's sheet op as `(op, sheet, to)`: `to` is the new name of a rename, the 0-based
    position of a move, the 1-based row of an insert_row or delete_row (spec key `row`; the
    blank row goes above it, T-1933), the column letter of an insert_column or delete_column
    (spec key `column`), the table name of a convert_table or delete_table (spec key `table`;
    T-2323) and None for an add or delete_sheet. Raises ValueError for an unknown or malformed op."""
    if isinstance(op, (list, tuple)) and len(op) == 3:
        key = _op_key(op[0])
        op = {"op": op[0], "sheet": op[1], **({} if op[2] is None or key is None else {key: op[2]})}
    if not isinstance(op, dict) or op.get("op") not in SHEET_OPS or not isinstance(op.get("sheet"), str):
        msg = f"unknown sheet op {op!r}: expected op {'/'.join(SHEET_OPS)} with a sheet name"
        raise ValueError(msg)
    kind, sheet = op["op"], op["sheet"]
    key = _op_key(kind)
    to = op.get(key) if key else None
    if set(op) - {"op", "sheet", key}:
        msg = f"sheet op {op!r} has unknown keys"
        raise ValueError(msg)
    if _valid_op_argument(kind, to):
        return kind, sheet, to
    msg = (
        f"sheet op {op!r}: `to` must be absent for add and delete_sheet, a name for rename, a position"
        " >= 0 for move; `row` a 1-based row for insert_row and delete_row; `column` a column letter"
        " (A to XFD) for insert_column and delete_column; `table` a table name for convert_table and"
        " delete_table"
    )
    raise ValueError(msg)
