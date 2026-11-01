"""Spec table options spec2xlsx reads: the header-row flag (T-3260) and a filtered column's values (T-1036)."""

from __future__ import annotations

from typing import Any


def has_header_row(table: dict[str, Any]) -> bool:
    """Whether a spec table has a header row: unless it says `"header_row": false` (T-3260)."""
    return table.get("header_row", True) is not False


def filter_value(raw: Any, a1: str) -> str:
    """The text a values filter lists for a first-column cell (General format: 3 → "3")."""
    if isinstance(raw, str) and raw and not raw.startswith("="):
        return raw
    if isinstance(raw, int | float) and not isinstance(raw, bool) and float(raw).is_integer():
        return str(int(raw))
    msg = f"filtered table column cell {a1} must be literal text or a whole number, not {raw!r}"
    raise ValueError(msg)
