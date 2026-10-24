"""The database sheet for the D* functions in generated cases (T-2249, decision 0094).

DSUM, DCOUNT, DGET and the other database functions need a `database` whose first row holds headers and
a `criteria` range whose first row names some of them (docs/excel-semantics/statistics.md "Database
function argument modes": both must be references; an array literal database makes Excel for the web
refuse the workbook, and an array criteria is `#VALUE!` per element). The random `Data` sheet holds
neither, so the `db_table` and `db_criteria` pools in pools.py point into a fixed sheet, `Db`, which a case
gets when one of its formulas refers to it.

`Db!A1:D9` is the table: Name, Qty, Price and Date over eight records, with a blank Name, a Price that
is text and one that is TRUE, a blank Price, a Name that holds a literal `*` and a numeric-text Name.
Each criteria block sits in its own column from F on (two adjacent blocks are an AND of both).
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

SHEET = "Db"
REF = f"{SHEET}!"

# The table, A1:D9. Row 10 onward is blank, so `Db!A1:D12` has trailing blank records.
TABLE: list[list[Any]] = [
    ["Name", "Qty", "Price", "Date"],
    ["apple", 4, 2.5, 45351],
    ["Apricot", 10, "x", 45000],
    ["banana", 0, 4, None],
    ["a*b", 2, True, 45657],
    [None, 1, 10, 45322],
    ["cherry", -4, 0.5, 45412],
    ["apple pie", 7, 100, 45535],
    ["123", 5, -1, 45291],
]


@dataclass(frozen=True)
class Formula:
    """A criteria cell that is a formula; any other text starting with "=" is a typed string."""

    text: str


# Criteria blocks, one per column from F; each is a header and its criteria rows. The quirk each one
# reaches (docs/excel-semantics/criteria.md unless said otherwise):
CRITERIA: dict[str, list[Any]] = {
    "F": ["Qty", ">2"],  # a comparison operator
    "G": ["Name", "a*"],  # a wildcard
    "H": ["Name", "apple"],  # plain text matches a prefix ("apple pie" too)
    "I": ["Name", "=apple"],  # = makes it exact
    "J": ["Price", "<>"],  # any non-blank cell
    "K": ["Qty", None],  # a blank criterion matches every record
    "L": ["Qty", ">=4", "<1"],  # two criteria rows are an OR
    "M": ["Name", "a~**"],  # an escaped wildcard: names that start with "a*"
    "N": ["Nope", 1],  # a header the table does not have
    "O": ["Calc", Formula("=B2>3")],  # a computed criterion: a formula under a label that is not a header
    "P": ["Date", ">45351"],  # a date column compared as a serial
    "Q": ["Qty", "<>4"],  # not equal
    "R": ["qty", 4],  # headers match case-insensitively; a number criterion
    "S": ["Price", "x"],  # a text criterion on a mixed column
}


def cells() -> dict[str, Any]:
    """The sheet's cells as a spec writes them (a formula is text starting with `=`)."""
    out: dict[str, Any] = {}
    for r, row in enumerate(TABLE, 1):
        for c, v in enumerate(row):
            if v is not None:
                out[f"{chr(ord('A') + c)}{r}"] = v
    for col, block in CRITERIA.items():
        for r, v in enumerate(block, 1):
            if v is None:
                continue
            # A spec reads text starting with "=" as a formula, so "=apple" is written as a typed string.
            if isinstance(v, Formula):
                out[f"{col}{r}"] = v.text
            elif isinstance(v, str) and v.startswith("="):
                out[f"{col}{r}"] = {"t": "s", "v": v}
            else:
                out[f"{col}{r}"] = v
    return out


def spec_sheet() -> dict[str, Any]:
    """The `Db` sheet of a case spec."""
    return {"name": SHEET, "cells": cells()}


def uses_db(formulas: list[str]) -> bool:
    """Whether any formula refers to the `Db` sheet."""
    return any(REF in f for f in formulas)
