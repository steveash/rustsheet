"""Pivot sources for GETPIVOTDATA in generated cases (T-2396, decision 0094).

A generated case has no pivot table, so a GETPIVOTDATA call there could only be `#REF!`. When the
generator draws a GETPIVOTDATA call (the catalog marks it `pivot: true`), the case gets a third sheet,
`Pivot`, holding a seeded source block at A1 (two or three text fields and one or two number fields,
4 to 10 records, no blanks) and one compact-form pivot over it at `LOCATION` (a row field, half the
time a column field, a number field aggregated by sum, count or average), written as a spec
`pivot_tables` entry with its `rendered` cells (rustsheet_tools/pivot.py). Every call in the case
queries that pivot.

A call names the pivot's data field (by source name or caption), a cell or range of the pivot, and 0
to 2 field/item pairs drawn from the pivot's axis fields and their items, with names in another case
now and then and an item sometimes a reference to its label cell. `MISS_RATE` of the calls carry one
deliberate miss, each a `#REF!` path Excel showed (docs/excel-semantics/web.md "GETPIVOTDATA against a
pivot table"): an unknown or padded data field, another aggregate's caption, a cell outside the
pivot, an item not in its field, a source field the pivot does not show, the same field twice, or a
field with no item (the 3-argument call only: an odd count of 5 or more makes Excel for the web refuse the
workbook). Arrays of items are left out: they spill, and a focus case's calls sit one per row.
"""

from __future__ import annotations

import random
from collections.abc import Callable
from dataclasses import dataclass
from typing import Any

from xlsxwriter.utility import xl_cell_to_rowcol, xl_rowcol_to_cell

from rustsheet_tools.gen.common import quoted
from rustsheet_tools.pivot import rendered_cells

SHEET = "Pivot"
LOCATION = "H3"  # right of the widest source block (A:E) with a gap column
MISS_RATE = 0.3
TEXT_FIELDS: dict[str, list[str]] = {
    "Region": ["East", "West", "North", "South"],
    "Product": ["A", "B", "C"],
    "Rep": ["Ann", "Bo", "Cy"],
}
NUMBER_FIELDS = ["Sales", "Units"]
NUMBERS: list[float] = [1, 2, 3, 5, 7, 10, 12, 20, 0.5, 2.25, 100, -4, 0]
FUNCTIONS = ["sum", "count", "average"]
CAPTIONS = {"sum": "Sum", "count": "Count", "average": "Average"}
UNKNOWN_FIELDS = ["Profit", "Market", "Total"]
UNKNOWN_ITEMS = ["Nowhere", "D", "Zed"]
OUTSIDE = ["$A$1", "$B$2", "$Z$40", "$G$1"]  # the source block, or cells off every pivot


@dataclass
class PivotSource:
    """The `Pivot` sheet of one case: its source cells and its one pivot table, with the names a call
    may draw from."""

    cells: dict[str, Any]
    table: dict[str, Any]
    axis: dict[str, list[str]]  # axis field → its items as the records spell them
    hidden: list[str]  # source fields the pivot does not show
    labels: dict[str, str]  # item → the A1 address of its row or column label in the rendered pivot
    box: tuple[int, int, int, int]  # the pivot's rendered rectangle: top, left, bottom, right

    def spec_sheet(self) -> dict[str, Any]:
        return {"name": SHEET, "cells": self.cells, "pivot_tables": [self.table]}


def build_source(rng: random.Random) -> PivotSource:
    """A seeded source block and pivot (module docstring)."""
    texts = rng.sample(sorted(TEXT_FIELDS), rng.randint(2, 3))
    numbers = rng.sample(NUMBER_FIELDS, rng.randint(1, 2))
    header = texts + numbers
    rng.shuffle(header)
    records = rng.randint(4, 10)
    cells: dict[str, Any] = {}
    for col, name in enumerate(header):
        cells[xl_rowcol_to_cell(0, col)] = name
        for row in range(1, records + 1):
            pool: list[Any] = TEXT_FIELDS.get(name, NUMBERS)
            cells[xl_rowcol_to_cell(row, col)] = rng.choice(pool)
    row_field, *rest = rng.sample(texts, len(texts))
    column_field = rest[0] if rng.random() < 0.5 else None
    value = rng.choice(numbers)
    function = rng.choice(FUNCTIONS)
    table: dict[str, Any] = {
        "name": "PivotGen",
        "source": f"{SHEET}!A1:{xl_rowcol_to_cell(records, len(header) - 1)}",
        "location": LOCATION,
        "row": row_field,
        "value": value,
        "function": function,
    }
    if column_field is not None:
        table["column"] = column_field
    spec = {"sheets": [{"name": SHEET, "cells": cells}]}
    rendered = rendered_cells(spec, spec["sheets"][0], table)
    table["rendered"] = rendered
    axis_fields = [row_field] if column_field is None else [row_field, column_field]
    column_of = {name: header.index(name) for name in header}
    axis = {
        name: sorted({str(cells[xl_rowcol_to_cell(r, column_of[name])]) for r in range(1, records + 1)})
        for name in axis_fields
    }
    items = {item for values in axis.values() for item in values}
    labels: dict[str, str] = {}
    for a1, cell in sorted(rendered.items(), key=lambda kv: xl_cell_to_rowcol(kv[0])):
        if isinstance(cell, str) and cell in items:
            labels.setdefault(cell, a1)
    corners = [xl_cell_to_rowcol(a1) for a1 in rendered]
    box = (
        min(r for r, _ in corners),
        min(c for _, c in corners),
        max(r for r, _ in corners),
        max(c for _, c in corners),
    )
    hidden = [name for name in header if name not in axis_fields and name != value]
    return PivotSource(cells, table, axis, hidden, labels, box)


def _cased(rng: random.Random, name: str) -> str:
    """`name`, now and then in another case (GETPIVOTDATA ignores case in names)."""
    roll = rng.random()
    if roll < 0.1:
        return name.lower()
    if roll < 0.2:
        return name.upper()
    return name


def _ref(a1: str) -> str:
    row, col = xl_cell_to_rowcol(a1.replace("$", ""))
    return f"{SHEET}!{xl_rowcol_to_cell(row, col, row_abs=True, col_abs=True)}"


def pivot_call(rng: random.Random, source: PivotSource, spelled: str) -> str:
    """One GETPIVOTDATA call against `source` (module docstring), `spelled` as the catalog says."""
    table = source.table
    data_field = _cased(
        rng, table["value"] if rng.random() < 0.7 else f"{CAPTIONS[table['function']]} of {table['value']}"
    )
    top, left, bottom, right = source.box
    if rng.random() < 0.6:
        pivot_ref = _ref(xl_rowcol_to_cell(top, left))
    elif rng.random() < 0.5:
        pivot_ref = _ref(xl_rowcol_to_cell(rng.randint(top, bottom), rng.randint(left, right)))
    else:
        pivot_ref = (
            f"{_ref(xl_rowcol_to_cell(top, left))}:{_ref(xl_rowcol_to_cell(bottom, right)).split('!')[1]}"
        )
    # Mostly with pairs: without one every call is the same grand total.
    count = 0 if rng.random() < 0.15 else rng.randint(1, len(source.axis))
    fields = rng.sample(sorted(source.axis), count)
    pairs: list[list[str]] = []
    for name in fields:
        item = rng.choice(source.axis[name])
        if item in source.labels and rng.random() < 0.15:
            pairs.append([quoted(_cased(rng, name)), _ref(source.labels[item])])
        else:
            pairs.append([quoted(_cased(rng, name)), quoted(_cased(rng, item))])

    def unknown_data_field() -> None:
        nonlocal data_field
        other = [c for c in CAPTIONS.values() if c != CAPTIONS[table["function"]]]
        data_field = rng.choice(
            [
                *UNKNOWN_FIELDS,
                f" {table['value']} ",
                f"{rng.choice(other)} of {table['value']}",
                *source.hidden,
            ]
        )

    def outside() -> None:
        nonlocal pivot_ref
        pivot_ref = _ref(rng.choice(OUTSIDE))

    def drop(name: str) -> None:
        # A miss names its field once more only where the miss is the repetition (`twice`).
        pairs[:] = [pair for pair in pairs if pair[0].strip('"').casefold() != name.casefold()]

    def unknown_item() -> None:
        name = rng.choice(sorted(source.axis))
        drop(name)
        item = rng.choice([*UNKNOWN_ITEMS, " " + rng.choice(source.axis[name])])
        pairs.append([quoted(name), quoted(item)])

    def hidden_field() -> None:
        names = [*source.hidden, table["value"], *UNKNOWN_FIELDS]
        pairs.append([quoted(rng.choice(names)), quoted(rng.choice(sorted(source.labels) or ["East"]))])

    def twice() -> None:
        # Two different items: the same item twice is unprobed (SPEC-0015 unknown 5).
        name = rng.choice([n for n in sorted(source.axis) if len(source.axis[n]) > 1] or [""])
        if not name:
            odd()
            return
        drop(name)
        items = source.axis[name]
        pairs.extend([[quoted(name), quoted(items[0])], [quoted(name), quoted(items[-1])]])

    def odd() -> None:
        # Only `(data_field, pivot_table, field)`: Excel for the web refuses the whole workbook when a
        # GETPIVOTDATA has an odd count of 5 or more arguments (web.md, T-2396).
        pairs[:] = [[quoted(rng.choice(sorted(source.axis)))]]

    misses: list[Callable[[], None]] = [unknown_data_field, outside, unknown_item, hidden_field, twice, odd]
    if rng.random() < MISS_RATE:
        rng.choice(misses)()
    args = [quoted(data_field), pivot_ref, *(text for pair in pairs for text in pair)]
    return f"{spelled}({','.join(args)})"
