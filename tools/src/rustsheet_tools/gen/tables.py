"""Policy `tables`: Excel tables and structured references for M8 (suite `gen-m8b`, T-1607).

Every case is one workbook: sheet `Data` holds four tables side by side, `Sales`, `Stock`, `Tiny`
and `Wide`, each with its own seeded shape (1 to 5 columns, 0 to 20 data rows, a totals row or
not; a table with 0 data rows has one blank data row, as Excel keeps one), and sheet `Calc` holds
most probes. Each case (one per category) differs in its seeded tables and its probes.

Probes sit in column K of `Calc`, rows 1 to `PER_CATEGORY`, except two categories: `this-row`
probes sit on `Data` right of the tables (columns AA onwards, rows 1 to `THIS_ROW_ROWS`), so
`Sales[@Qty]` reads its own row through the table (and is `#VALUE!` on a row outside it), and
`in-table` probes are the cells of four formula columns (`Calc` ...) appended to `Sales` and `Wide`
(20 data rows each). Probes are distinct per row there, elsewhere per case. Every probe yields
one value: a reference to more than one cell is always inside an aggregate or INDEX. Nothing is
volatile. A formula never reads its own table column, so no probe is circular.

The specs use the forms a user types (`Sales[@Qty]`, a bare `Sales`); spec2xlsx stores them in
their file form (`table_refs_to_file_form`). Header text with `[`, `]`, `#`, `'` or `@` is
escaped with `'` in references (`Stock[C'#]`). Known behavior: docs/excel-semantics/references.md
("Structured references") and xlsx-storage.md ("Structured references in the file").
"""

from __future__ import annotations

import random
import re
from dataclasses import dataclass
from functools import partial
from typing import Any

from xlsxwriter.utility import xl_col_to_name

from rustsheet_tools.gen.common import SeededDraws, draw_unseen

PER_CATEGORY = 400
PROBE_COL = "K"
THIS_ROW_FIRST_COL = 26  # AA: right of the four tables (at most 5 + 5 + 2 + 5 columns and 3 gaps)
THIS_ROW_ROWS = 24  # one past the lowest table row (top 2 + header + 20 rows + totals)
TABLE_NAMES = ["Sales", "Stock", "Tiny", "Wide"]
# (name, column counts, data row counts) per table; `Tiny` has 0 or 1 data rows.
SHAPES = {
    "Sales": (range(2, 6), range(1, 21)),
    "Stock": (range(1, 6), range(0, 21)),
    "Tiny": (range(1, 3), range(0, 2)),
    "Wide": (range(5, 6), range(3, 21)),
}
PLAIN_HEADERS = ["Qty", "Price", "Region", "Unit Price", "Amt", "Note", "Q1 Sales", "Key", "Val", "Cost"]
PLAIN_HEADERS += ["2024", "Tax %", "Total$", "a.b", "Name"]
SPECIAL_HEADERS = ["C#", "it's", "[x]", "a]b", "x@y", "#N", "Rate [%]", "O'Neil #2"]
NUMBERS = ["0", "1", "2", "3", "0.5", "10", "-2", "100", "7"]
DATA_POOL: list[Any] = [0, 1, -1, 2, 0.5, 3.25, 10, 100, 7, -4, 12, 5, "a", "b", "x", "1", True, None]
DATA_POOL += [{"t": "e", "v": "#N/A"}]
AGG = ["SUM", "COUNT", "COUNTA", "AVERAGE", "MAX", "MIN", "ROWS", "COLUMNS", "COUNTBLANK", "PRODUCT"]
SPECIALS = ["#All", "#Data", "#Headers", "#Totals"]
SUBTOTAL_CODES = [101, 102, 103, 104, 105, 107, 109, 110]
# The `in-table` case appends these formula columns to `Sales` and `Wide`, with 20 data rows each.
FORMULA_HEADERS = ["Calc", "Calc 2", "Calc 3", "Calc 4"]
IN_TABLE = ("Sales", "Wide")
# Categories whose probes read their own row: the same formula on another row is another probe.
ROW_SCOPED = {"this-row", "in-table"}
_ESCAPED = re.compile(r"([\[\]#'@])")


def escape_column(header: str) -> str:
    """A header as it is written inside a structured reference's brackets: `'` before each of
    `[`, `]`, `#`, `'` and `@` (`C#` → `C'#`)."""
    return _ESCAPED.sub(r"'\1", header)


@dataclass
class Table:
    """One seeded table: its name, top-left (0-based column, 1-based row), headers and shape."""

    name: str
    col: int
    top: int
    headers: list[str]
    rows: int  # data rows written (at least 1: an empty table keeps one blank row)
    empty: bool  # 0 data rows: the one data row is blank
    totals: bool

    @property
    def bottom(self) -> int:
        return self.top + self.rows + int(self.totals)

    @property
    def ref(self) -> str:
        last = xl_col_to_name(self.col + len(self.headers) - 1)
        return f"{xl_col_to_name(self.col)}{self.top}:{last}{self.bottom}"


class _Gen(SeededDraws):
    """Seeded draws of structured references over the case's tables."""

    numbers = NUMBERS

    def __init__(self, rng: random.Random, tables: list[Table], data_columns: dict[str, list[str]]) -> None:
        super().__init__(rng)
        self.tables = tables
        # The columns a probe may read (the `in-table` formula column is left out: no cycles).
        self.data_columns = data_columns
        self.home: Table | None = None  # the table an `in-table` probe sits in

    def table(self, names: list[str] | None = None) -> Table:
        return self.rng.choice([t for t in self.tables if names is None or t.name in names])

    def column(self, t: Table) -> str:
        return escape_column(self.pick(self.data_columns[t.name]))

    def col_ref(self, t: Table) -> str:
        """`Sales[Qty]` or the equivalent `Sales[[Qty]]`."""
        c = self.column(t)
        return f"{t.name}[{c}]" if self.rng.random() < 0.7 else f"{t.name}[[{c}]]"

    def col_span(self, t: Table) -> str:
        """`[A]:[C]` over two columns of `t` (in either order, or the same column twice)."""
        cols = self.data_columns[t.name]
        a, b = self.rng.choice(cols), self.rng.choice(cols)
        return f"[{escape_column(a)}]:[{escape_column(b)}]"


def this_row(column: str) -> str:
    """`@Qty`, or `@[Unit Price]` for a column (escaped) that is not one word: Excel only takes a
    one-word column after `@` without its own brackets."""
    return f"@{column}" if re.fullmatch(r"\w+", column) else f"@[{column}]"


def _column(g: _Gen) -> str:
    t = g.table()
    ref = g.col_ref(t)
    k = g.rng.randint(0, 22)
    return g.pick(
        [
            f"{g.pick(AGG)}({ref})",
            f"{g.pick(AGG)}({ref})+{g.num()}",
            f"INDEX({ref},{k})",
            f'IFERROR(INDEX({ref},{k}),"e")',
            f"{g.pick(AGG[:6])}({ref},{g.col_ref(g.table())})",
            f"SUM({ref})-SUM({g.col_ref(t)})",
        ]
    )


# `{t}` the table, `{c}` one of its columns, `{agg}` an aggregate, `{special}` an item specifier.
SPECIAL_FORMS = [
    "{agg}({t}[{special}])",
    "{shape}({t})",
    "ROWS({t}[])",
    "INDEX({t}[{special}],{i},{j})",
    "INDEX({t},{i},{j})",
    "{t}[[#Headers],[{c}]]",
    "{t}[[#Totals],[{c}]]",
    "{agg}({t}[[{special}],[{c}]])",
    "ROWS({t}[[#Headers],[#Data],[{c}]])",
    "ROWS({t}[[#Data],[#Totals]])",
    "ISREF({t}[{special}])",
    'IFERROR(ROWS({t}[{special}]),"e")',
]


def _specials(g: _Gen) -> str:
    t = g.table()
    return g.pick(SPECIAL_FORMS).format(
        t=t.name,
        c=g.column(t),
        agg=g.pick(AGG),
        special=g.pick(SPECIALS),
        shape=g.pick(["ROWS", "COLUMNS", "COUNTA"]),
        i=g.rng.randint(1, 23),
        j=g.rng.randint(1, 5),
    )


def _column_range(g: _Gen) -> str:
    t = g.table(["Sales", "Stock", "Wide"])
    span = g.col_span(t)
    return g.pick(
        [
            f"{g.pick(AGG)}({t.name}[{span}])",
            f"{g.pick(AGG)}({t.name}[[{g.pick(SPECIALS)}],{span}])",
            f"COLUMNS({t.name}[[#Headers],{span}])",
            f"INDEX({t.name}[{span}],{g.rng.randint(1, 20)},{g.rng.randint(1, 3)})",
            f"SUM({t.name}[{span}])-SUM({g.col_ref(t)})",
        ]
    )


def _this_row(g: _Gen) -> str:
    t = g.table()
    c = g.column(t)
    return g.pick(
        [
            f"{t.name}[{this_row(c)}]" if g.rng.random() < 0.5 else f"{t.name}[@[{c}]]",
            f"{t.name}[{this_row(c)}]*{g.num()}",
            f'{t.name}[{this_row(c)}]&"|"',
            f"{t.name}[[#This Row],[{c}]]",
            f"SUM({t.name}[@{g.col_span(t)}])",
            f"COUNTA({t.name}[@])",
            f"ISERROR({t.name}[{this_row(c)}])",
            f'IFERROR({t.name}[{this_row(c)}],"out")',
            f"@{g.col_ref(t)}",
            f"ROW({t.name}[{this_row(c)}])",
        ]
    )


def _in_table(g: _Gen) -> str:
    home = g.home or g.table(list(IN_TABLE))
    n, c = home.name, g.column(home)
    other = g.table([t.name for t in g.tables if t.name != n])
    return g.pick(
        [
            f"{n}[{this_row(c)}]*{g.num()}",
            f"{n}[@[{c}]]",
            f"SUM({n}[@{g.col_span(home)}])",
            f"{n}[[#This Row],[{c}]]+{g.num()}",
            f"ROW({n}[{this_row(c)}])",
            f"SUM({n}[{c}])",
            f'{n}[{this_row(c)}]&"|"',
            f"{g.pick(AGG[:6])}({n}[{c}])-{n}[{this_row(c)}]",
            f"COUNTA({other.name}[{this_row(g.column(other))}])",
            f'IFERROR({other.name}[{this_row(g.column(other))}],"out")',
        ]
    )


def _escaped(g: _Gen) -> str:
    t = g.table(["Stock", "Wide"])
    return g.pick(
        [
            f"{g.pick(AGG)}({g.col_ref(t)})",
            f"{t.name}[[#Headers],[{g.column(t)}]]",
            f"{g.pick(AGG)}({t.name}[{g.col_span(t)}])",
            f"INDEX({g.col_ref(t)},{g.rng.randint(1, 20)})",
            f"{g.pick(AGG)}({t.name}[[#All],[{g.column(t)}]])",
        ]
    )


TOTALS_FORMS = [
    "{t}[[#Totals],[{c}]]",
    "SUM({t}[#Totals])",
    "COUNTA({t}[#Totals])",
    "ROWS({t}[[#Data],[#Totals]])",
    "ROWS({t}[#All])",
    "ISERROR({t}[[#Totals],[{c}]])",
    "SUBTOTAL({code},{t}[{c}])",
    "SUBTOTAL({code},{t}[#All])",
    "{agg}({t}[[#Totals],{span}])",
]


def _totals(g: _Gen) -> str:
    t = g.table()
    return g.pick(TOTALS_FORMS).format(
        t=t.name, c=g.column(t), code=g.rng.choice(SUBTOTAL_CODES), agg=g.pick(AGG[:6]), span=g.col_span(t)
    )


def _lookups(g: _Gen) -> str:
    t = g.table(["Sales", "Stock", "Wide"])
    a, b = g.col_ref(t), g.col_ref(t)
    v = g.pick([*NUMBERS, '"a"', '"x"', '"1"'])
    width = len(g.data_columns[t.name])
    return g.pick(
        [
            f'IFERROR(VLOOKUP({v},{t.name},{g.rng.randint(1, width)},FALSE),"nf")',
            f"MATCH({v},{a},0)",
            f'XLOOKUP({v},{a},{b},"nf")',
            f'SUMIF({a},">"&{g.num()},{b})',
            f"COUNTIF({a},{v})",
            f"SUMPRODUCT(--ISNUMBER({a}))",
            f'AVERAGEIF({a},"<>"&{v})',
            f"INDEX({t.name},MATCH({v},{a},0),1)",
            f"SUM({a})-SUM(Data!A1:E25)",
            f'COUNTIFS({a},{v},{b},"<>")',
        ]
    )


def _names_case(g: _Gen) -> str:
    """Table and column names in other letter cases, and the table name as a whole."""
    t = g.table()
    c = g.column(t)
    name = g.pick([t.name.lower(), t.name.upper(), t.name])
    col = g.pick([c.lower(), c.upper(), c])
    return g.pick(
        [
            f"SUM({name}[{col}])",
            f"ROWS({name}[#{g.pick(['all', 'DATA', 'Headers', 'totals'])}])",
            f"COLUMNS({name})",
            f"{name}[[#HEADERS],[{col}]]",
            f"COUNTA({name}[[{col}]:[{col}]])",
        ]
    )


CATEGORIES = {
    "column": _column,
    "specials": _specials,
    "column-range": _column_range,
    "this-row": _this_row,
    "in-table": _in_table,
    "escaped": _escaped,
    "totals": _totals,
    "lookups": _lookups,
    "names-case": _names_case,
}


def _tables(rng: random.Random, category: str) -> list[Table]:
    """Seeded table shapes, placed left to right on `Data` with a blank column between them."""
    tables: list[Table] = []
    col = 0
    for name in TABLE_NAMES:
        widths, rows = SHAPES[name]
        width = rng.choice(widths)
        pool = list(PLAIN_HEADERS)
        if category == "escaped" and name in ("Stock", "Wide"):
            pool = list(SPECIAL_HEADERS) + rng.sample(PLAIN_HEADERS, 1)
        elif rng.random() < 0.3:
            pool += SPECIAL_HEADERS
        headers = rng.sample(pool, width)
        count = rng.choice(rows)
        top = rng.choice([1, 2])
        if category == "in-table" and name in IN_TABLE:
            headers, count = headers + FORMULA_HEADERS, 20
        table = Table(name, col, top, headers, max(count, 1), count == 0, rng.random() < 0.5)
        tables.append(table)
        col += len(table.headers) + 1
    return tables


def _table_cells(rng: random.Random, t: Table, data_columns: list[str]) -> dict[str, Any]:
    cells: dict[str, Any] = {}
    for i, header in enumerate(t.headers):
        letter = xl_col_to_name(t.col + i)
        cells[f"{letter}{t.top}"] = header
        if header not in data_columns:
            continue  # an `in-table` formula column, filled with probes
        for r in range(t.top + 1, t.top + 1 + t.rows):
            v = None if t.empty else rng.choice(DATA_POOL)
            if v is not None:
                cells[f"{letter}{r}"] = v
        if t.totals:
            code = rng.choice(SUBTOTAL_CODES)
            ref = f"{t.name}[{escape_column(header)}]"
            total = "Total" if i == 0 and rng.random() < 0.5 else f"=SUBTOTAL({code},{ref})"
            cells[f"{letter}{t.bottom}"] = rng.choice([total, total, None])
    return {a1: v for a1, v in cells.items() if v is not None}


def _probe_cells(category: str, tables: list[Table], count: int) -> list[tuple[str, str, Table | None]]:
    """Where a category's probes go, in order: (sheet, A1, the table the cell is in or None)."""
    if category == "this-row":
        width = -(-count // THIS_ROW_ROWS)
        cells = [(c, r) for c in range(width) for r in range(1, THIS_ROW_ROWS + 1)][:count]
        return [("Data", f"{xl_col_to_name(THIS_ROW_FIRST_COL + c)}{r}", None) for c, r in cells]
    if category == "in-table":
        return [
            ("Data", f"{xl_col_to_name(t.col + i)}{r}", t)
            for t in tables
            if t.name in IN_TABLE
            for i, header in enumerate(t.headers)
            if header in FORMULA_HEADERS
            for r in range(t.top + 1, t.top + 1 + t.rows)
        ]
    return [("Calc", f"{PROBE_COL}{r}", None) for r in range(1, count + 1)]


def generate_tables(seed: int, per_category: int = PER_CATEGORY) -> list[dict[str, Any]]:
    """Return one `rustsheet-case/v1` spec per category, up to `per_category` probes each."""
    cases: list[dict[str, Any]] = []
    for category, draw in CATEGORIES.items():
        rng = random.Random(f"{seed}/{category}")
        tables = _tables(rng, category)
        data_columns = {t.name: [h for h in t.headers if h not in FORMULA_HEADERS] for t in tables}
        sheets: dict[str, dict[str, Any]] = {"Data": {}, "Calc": {"A1": "calc"}}
        for t in tables:
            sheets["Data"].update(_table_cells(rng, t, data_columns[t.name]))
        g = _Gen(rng, tables, data_columns)
        hypotheses: dict[str, str] = {}
        seen: dict[str, set[str]] = {}  # formulas drawn, per row for ROW_SCOPED categories
        for sheet, a1, home in _probe_cells(category, tables, per_category):
            g.home = home
            scope = a1.lstrip("ABCDEFGHIJKLMNOPQRSTUVWXYZ") if category in ROW_SCOPED else ""
            if (formula := draw_unseen(partial(draw, g), seen.setdefault(scope, set()))) is not None:
                sheets[sheet][a1] = formula
                hypotheses[f"{sheet}!{a1}"] = category
        if category == "totals":
            # The totals-row formulas are probes too.
            for t in (t for t in tables if t.totals):
                for c in range(t.col, t.col + len(t.headers)):
                    a1 = f"{xl_col_to_name(c)}{t.bottom}"
                    if str(sheets["Data"].get(a1, "")).startswith("="):
                        hypotheses[f"Data!{a1}"] = category
        tables_spec = [
            {"name": t.name, "ref": t.ref} | ({"totals_row": True} if t.totals else {}) for t in tables
        ]
        cases.append(
            {
                "format": "rustsheet-case/v1",
                "description": f"tables probes ({category}) seed={seed}; M8 Excel tables and structured references",
                "tags": ["generated", "M8", "tables", category],
                "sheets": [
                    {"name": "Data", "cells": sheets["Data"], "tables": tables_spec},
                    {"name": "Calc", "cells": sheets["Calc"]},
                ],
                "expected": {},
                "check": list(hypotheses),
                "hypotheses": hypotheses,
            }
        )
    return cases
