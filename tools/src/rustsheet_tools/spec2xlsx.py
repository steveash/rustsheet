"""Builds an .xlsx from a `rustsheet-case/v1` spec so a real Excel can compute it.

Conventions (normative; mirrored by the Rust engine adapter in crates/rustsheet-eval/src/engine.rs):
  * JSON number → number cell, JSON bool → boolean cell, "=..." → formula, other string → text.
  * Tagged values: {"t":"s"} text, {"t":"n"} number, {"t":"b"} bool, {"t":"e"} error.
  * Every formula is written as a *dynamic-array* formula, i.e. with the semantics of a
    formula typed into current Microsoft 365 Excel (no implicit intersection).
    A sheet may list cells in `"legacy_formulas": ["H1", ...]`; those are written as plain `<f>`
    formulas (no `cm` dynamic-array metadata), as a pre-dynamic-array file holds them (T-1077).
    The engine adapter ignores the list. `@` in such a cell is still written as raw `@`, not the
    stored `_xlfn.SINGLE`, so it shows Excel's parse of the text, not of a genuine legacy file.
  * Error literals are written as the formula `=#N/A` etc. (xlsx writers cannot emit
    error constants); they behave identically for dependants. A typed error with `"stored": true`
    (`{"t":"e","v":"#BLOCKED!","stored":true}`) is instead patched into the file as a stored
    constant `<c t="e"><v>#BLOCKED!</v></c>`, for names Excel rejects as formula grammar (T-0999).
  * The empty-string text constant "" is written as the formula `=""` (XlsxWriter turns an
    empty string into a blank cell, which has different semantics).
  * Spill references (`A1#`) are written in their file form `_xlfn.ANCHORARRAY(A1)`; Excel for the
    web rejects a literal `A1#` in `<f>` (T-0042). The engine reads the spec text, so cases keep `A1#`.
  * Typed entry `{"entered":"(1)"}` (T-0280): the cell is left blank and the docProps custom
    properties `rustsheet.entered.<n>` (one per cell, n from 1 in spec order; Excel caps a property
    value at 255 characters) hold it as JSON `[sheet, a1, text]`; oracle backends write
    the text into the cell as a user types it (Graph `values`), so Excel picks value and number
    format. The engine adapter enters the same text through its user-input parser.
  * `"sheet_ops": [{"op": "rename", "sheet": "Data", "to": "Inputs"}, {"op": "add", "sheet": "New"},
    {"op": "move", "sheet": "Inputs", "to": 0}]` at the top level are sheet edits made after the
    workbook loads and before it is recalculated (T-1608): `add` appends a sheet, `rename` renames
    one (Excel rewrites the formulas that refer to it), `move` puts one at a 0-based position. The
    docProps custom properties `rustsheet.sheetop.<n>` hold them as JSON, n from 1 in order, and the
    oracle backend replays them in Excel; the engine adapter applies them through its edit API.
  * A sheet may list `"hidden_rows": [3, 5]` (1-based rows written with `hidden="1"`) and
    `"tables": [{"name": "T1", "ref": "A1:B6", "filter": true}]` (T-1036). A table's header row is the
    first row of `ref` and its cells must be literal text. With `"filter": true` the table's autoFilter
    gets a values filter on its first column that keeps exactly the visible data rows, so the hidden rows
    inside the table are the rows the filter hides; that column's data cells must be literal numbers or
    text, and no value may be both kept and hidden. The engine adapter loads both. With
    `"totals_row": true` the last row of `ref` is the totals row: its cells (labels or formulas) are
    written with the table's totals metadata (T-1607). Structured references are stored in their file
    form: a bare table name as `Sales[]`, `@` as `#This Row` (`table_refs_to_file_form`).
  * `"date_system": 1904` at the top level writes `<workbookPr date1904="1"/>` (1900, the default,
    writes nothing), and a sheet's `"number_formats": {"A1": "[h]:mm"}` gives those cells that number
    format, blank cells included; an `entered` cell cannot have one (T-1268). The engine adapter
    applies both.
  * `"calculation": {"iterate": true, "iterate_count": 100, "iterate_delta": 0.001}` at the top level
    writes those attributes on `<calcPr>` (iterative calculation; counts and deltas default to
    Excel's 100 and 0.001). A sheet's `"data_tables": [{"ref": "B4:B6", "column_input": "A1"}]`
    writes a what-if data table: the cells of `ref` are placeholders (0) and its top-left cell gets
    `<f t="dataTable">`; `row_input` alone is a one-variable row table, both inputs a two-variable
    table. `ref` must not overlap the sheet's `cells`. The engine adapter ignores both (Phase 2,
    T-0428).
  * A sheet's `"pivot_tables": [{"name": "Pv", "source": "Data!A1:C7", "location": "E3", "row": "Region",
    "column": "Product", "value": "Sales", "function": "sum"}]` writes pivot tables, rendered cells
    included (rustsheet_tools/pivot.py, T-1705). The engine adapter ignores them.
  * Future functions get their `_xlfn.` prefixes from the builder's own table (the engine exporter's
    list plus `_BUILDER_ONLY_PREFIXES`), never from XlsxWriter's `use_future_functions`, which skips a
    whole formula once any call in it is prefixed (T-0052); LET/LAMBDA names are written as `_xlpm.x`.
    A sheet may list formula cells in `"bare_function_names": ["A1", ...]`; those keep their function
    names as typed (no `_xlfn.` added), to probe how a bare newer name loads (T-0416). The engine
    adapter ignores the list.
"""

from __future__ import annotations

import argparse
import functools
import io
import json
import math
import re
from collections.abc import Callable, Iterator
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Any
from xml.sax.saxutils import escape, unescape

import xlsxwriter

from rustsheet_tools.oracle.package import patch_parts, read_parts
from rustsheet_tools.pivot import PivotParts, pivot_tables, relationships_xml

# XlsxWriter stamps docProps/core.xml with the wall clock; a fixed instant keeps identical specs
# byte-identical so `input_sha256` (sidecars, eval/.cache/oracle) is stable across runs (T-0015).
_FIXED_CREATED = datetime(2000, 1, 1)

ENTERED_PROPERTY = "rustsheet.entered"
SHEET_OP_PROPERTY = "rustsheet.sheetop"
SHEET_OPS = ("add", "rename", "move")

_A1 = re.compile(r"^\$?([A-Za-z]{1,3})\$?(\d+)$")


# `A1#`, `$A$1#`, `Sheet!A1#`, `'My Sheet'!A1#`; the lookbehind keeps names like `X1A1#` whole.
_SPILL_REF = re.compile(r"(?<![\w.$!'])((?:(?:'(?:[^']|'')+'|[A-Za-z_][\w.]*)!)?\$?[A-Za-z]{1,3}\$?\d+)#")
_STRING_LITERAL = re.compile(r'"(?:[^"]|"")*"')


# A name starts with a letter of any script or `_` (`LET(é,1,é+1)` is `_xlpm.é`, T-1387).
_IDENT = re.compile(r"[^\W\d][\w.]*")
_ENGINE_FUNCTIONS = (
    Path(__file__).resolve().parents[3] / "crates/rustsheet-engine/src/functions/xlsx_names.rs"
)
_XLSX_NAME_ARM = re.compile(r'Function::\w+ => "(_xlfn\.[\w.]+)"\.to_string\(\)')


@functools.cache
def future_function_prefixes() -> dict[str, str]:
    """Maps bare upper-case function names to their file form (`TRIMRANGE` → `_xlfn.TRIMRANGE`).

    Read from the engine exporter's `Function::to_xlsx_string` so there is one list (T-0166).
    """
    found = _XLSX_NAME_ARM.findall(_ENGINE_FUNCTIONS.read_text(encoding="utf-8"))
    return {f.removeprefix("_xlfn.").removeprefix("_xlws.").upper(): f for f in found}


def _skip_literal(formula: str, i: int) -> int:
    """Index just past the string literal or quoted sheet name starting at `i`."""
    quote = formula[i]
    i += 1
    while i < len(formula):
        if formula[i] == quote:
            if formula[i + 1 : i + 2] == quote:
                i += 2
                continue
            return i + 1
        i += 1
    return i


def _opens_table_ref(formula: str, i: int) -> bool:
    """Whether the `[` at `i` opens a table's bracket group (`Sales[`): it follows a name directly.
    A LAMBDA's optional parameter `[y]` never does."""
    return formula[i] == "[" and i > 0 and (formula[i - 1].isalnum() or formula[i - 1] in "_.")


def _skip_brackets(formula: str, i: int) -> int:
    """Index just past the structured-reference bracket group whose `[` is at `i`. Inside it `'`
    escapes the next character (`Sales[C'#]`, `Sales[it''s]`), so it is no quote."""
    depth = 0
    while i < len(formula):
        ch = formula[i]
        if ch == "'":
            i += 2
            continue
        if ch == "[":
            depth += 1
        elif ch == "]":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return i


def _call_args(formula: str, open_idx: int) -> tuple[list[str], int]:
    """Top-level argument texts of the call whose `(` is at `open_idx`, and the index of its `)`."""
    args: list[str] = []
    depth, start, i = 0, open_idx + 1, open_idx
    while i < len(formula):
        ch = formula[i]
        if ch in "\"'":
            i = _skip_literal(formula, i)
            continue
        if _opens_table_ref(formula, i):
            i = _skip_brackets(formula, i)
            continue
        if ch in "({":
            depth += 1
        elif ch in ")}":
            depth -= 1
            if depth == 0:
                args.append(formula[start:i])
                return args, i
        elif ch == "," and depth == 1:
            args.append(formula[start:i])
            start = i + 1
        i += 1
    return args, len(formula)


def _words(formula: str) -> Iterator[tuple[bool, str, int]]:
    """Splits a formula into `(is_name, text, start)` pieces; literals, a table's bracket groups
    (`[[#This Row],[A]]` after `Sales`) and punctuation are not names."""
    i = 0
    while i < len(formula):
        ch = formula[i]
        if ch in "\"'" or _opens_table_ref(formula, i):
            end = _skip_literal(formula, i) if ch in "\"'" else _skip_brackets(formula, i)
            yield False, formula[i:end], i
            i = end
            continue
        m = _IDENT.match(formula, i)
        if not m or (i > 0 and (formula[i - 1].isalnum() or formula[i - 1] in "_.!$")):
            yield False, ch, i
            i += 1
            continue
        yield True, m.group(0), i
        i = m.end()


_OPTIONAL_PARAM = re.compile(r"\s*(\[)\s*([^\W\d][\w.]*)\s*(\])\s*")


def prefix_bound_names(formula: str) -> str:
    """Writes LET variables and LAMBDA parameters as `_xlpm.x`, as Excel stores them (T-0166).

    An optional LAMBDA parameter `[y]` is declared as `_xlop.y`, brackets dropped, and used as
    `_xlpm.y`: Excel for the web stores `=LAMBDA(x,[y],ISOMITTED(y))(1)` as
    `_xlfn.LAMBDA(_xlpm.x,_xlop.y,_xlfn.ISOMITTED(_xlpm.y))(1)`, and refuses `[_xlpm.y]` as a
    corrupt file (T-1274; docs/excel-semantics/xlsx-storage.md).
    """
    out: list[str] = []
    scopes: list[tuple[int, set[str]]] = []  # (end index of the call, lower-cased names)
    # `_words` yields `[`, the name and `]` as separate pieces, so the declaration is marked by
    # position: the name is rewritten and the brackets (with any spaces) are skipped.
    optional: set[int] = set()  # start of each optional parameter's name
    dropped: set[int] = set()  # its brackets and the spaces around the name
    for is_name, word, i in _words(formula):
        end = i + len(word)
        scopes = [sc for sc in scopes if sc[0] > i]
        upper = word.upper()
        if i in dropped:
            continue
        if is_name and i in optional:
            out.append(f"_xlop.{word}")
            continue
        if is_name and formula[end : end + 1] == "(" and upper in ("LET", "LAMBDA"):
            args, close = _call_args(formula, end)
            decl = args[0:-1:2] if upper == "LET" else args[:-1]
            names = {a.strip().strip("[]").strip().lower() for a in decl}
            scopes.append((close, {n for n in names if _IDENT.fullmatch(n)}))
            if upper == "LAMBDA":
                start = end + 1
                for arg in args[:-1]:
                    m = _OPTIONAL_PARAM.fullmatch(arg)
                    if m:
                        optional.add(start + m.start(2))
                        dropped.update(range(start + m.end(2), start + len(arg)))
                        dropped.update(range(start, start + m.start(2)))
                    start += len(arg) + 1
        elif is_name and formula[end : end + 1] != "!" and any(word.lower() in names for _, names in scopes):
            out.append(f"_xlpm.{word}")
            continue
        out.append(word)
    return "".join(out)


# Newer functions XlsxWriter (3.2.9) prefixes that the engine exporter's list lacks. The builder owns
# every prefix: XlsxWriter leaves a formula alone once it contains `_xlfn.`, so with its rewrite on,
# `=TRIMRANGE(A2)+DAYS(1,2)` stored a bare `DAYS`, which Excel reads as `#NAME?` (T-0052;
# docs/excel-semantics/xlsx-storage.md, "Newer functions need `_xlfn.`"). Its regexes also leave `.`
# unescaped (`NORM.DIST` matched `NORMSDIST`, T-0040), which the builder's tokenizer cannot do.
# tools/tests/test_spec2xlsx_prefixes.py checks this table still covers XlsxWriter's list.
_BUILDER_ONLY_PREFIXES = {
    name: f"_xlfn.{name}"
    for name in (
        "ANCHORARRAY",
        "FILTERXML",
        "IMAGE",
        "QUERYSTRING",
        "SINGLE",
        "WEBSERVICE",
    )
}


def prefix_future_functions(formula: str) -> str:
    """Writes every call of a newer function with its stored prefix (`TRIMRANGE(` → `_xlfn.TRIMRANGE(`)."""
    prefixes = {**future_function_prefixes(), **_BUILDER_ONLY_PREFIXES}
    out: list[str] = []
    for is_name, word, i in _words(formula):
        if is_name and formula[i + len(word) : i + len(word) + 1] == "(" and word.upper() in prefixes:
            out.append(prefixes[word.upper()])
            continue
        out.append(word)
    return "".join(out)


def table_refs_to_file_form(formula: str, tables: frozenset[str]) -> str:
    """Writes structured references in their file form (T-1607): a bare table name `Sales` as
    `Sales[]` and `@` as `#This Row` (`Sales[@A]` → `Sales[[#This Row],[A]]`, `Sales[@]` →
    `Sales[#This Row]`). `tables` holds the workbook's table names, upper-cased. Excel for the web
    refuses a file holding a bare table name as corrupt and reads a stored `@` as `#REF!`
    (docs/excel-semantics/xlsx-storage.md, "Structured references in the file"). References must
    name their table: an unqualified `[@A]` is left as is."""
    out: list[str] = []
    for is_name, word, i in _words(formula):
        end = i + len(word)
        if is_name and word.upper() in tables and formula[end : end + 1] not in ("[", "(", "!"):
            out.append(f"{word}[]")
        elif word.startswith("[@") and _opens_table_ref(formula, i):
            inner = word[2:-1]
            if not inner:
                out.append("[#This Row]")
            else:
                out.append(f"[[#This Row],{inner if inner.startswith('[') else f'[{inner}]'}]")
        else:
            out.append(word)
    return "".join(out)


def anchor_spill_refs(formula: str) -> str:
    """Rewrites spill references outside string literals to `_xlfn.ANCHORARRAY(ref)`."""
    out: list[str] = []
    pos = 0
    for lit in _STRING_LITERAL.finditer(formula):
        out.append(_SPILL_REF.sub(r"_xlfn.ANCHORARRAY(\1)", formula[pos : lit.start()]))
        out.append(lit.group(0))
        pos = lit.end()
    out.append(_SPILL_REF.sub(r"_xlfn.ANCHORARRAY(\1)", formula[pos:]))
    return "".join(out)


def parse_a1(a1: str) -> tuple[int, int]:
    """'B3' → (2, 1) zero-based (row, col)."""
    m = _A1.match(a1.strip())
    if not m:
        msg = f"bad cell address {a1!r}"
        raise ValueError(msg)
    col = 0
    for ch in m.group(1).upper():
        col = col * 26 + (ord(ch) - ord("A") + 1)
    return int(m.group(2)) - 1, col - 1


def build_xlsx(spec: dict[str, Any]) -> bytes:
    """Returns xlsx bytes for a case spec."""
    buf = io.BytesIO()
    wb = xlsxwriter.Workbook(
        buf,
        {
            "in_memory": True,
            "use_future_functions": False,
            "strings_to_numbers": False,
            "strings_to_formulas": False,
            "strings_to_urls": False,
            "date_1904": _date_1904(spec),
        },
    )
    wb.set_properties({"created": _FIXED_CREATED})
    patches = _Patches()
    tables = frozenset(str(t["name"]).upper() for sheet in spec["sheets"] for t in sheet.get("tables", ()))
    entered: list[tuple[str, str, str]] = []  # (sheet name, cell ref, text as typed)
    worksheets = []
    for number, sheet in enumerate(spec["sheets"], start=1):
        ws = wb.add_worksheet(sheet["name"])
        worksheets.append(ws)
        hidden = _hidden_rows(sheet)
        for row in sorted(hidden):
            ws.set_row(row - 1, None, None, {"hidden": True})
        for table in sheet.get("tables", ()):
            patches.filters.append(_add_table(ws, sheet, table, hidden, tables))
        formats = _number_formats(wb, sheet)
        legacy = _listed_formula_cells(sheet, "legacy_formulas")
        bare = _listed_formula_cells(sheet, "bare_function_names")
        if both := legacy & bare:
            msg = f"cells both legacy_formulas and bare_function_names: {sorted(both)}"
            raise ValueError(msg)
        totals_cells = _totals_cells(sheet)
        for a1, raw in sheet["cells"].items():
            row, col = parse_a1(a1)
            if a1 in totals_cells:
                continue  # written with its table (`_totals_column`)
            if a1 in bare:
                text = _formula_text(raw, tables, prefix_functions=False)
                ws.write_dynamic_array_formula(row, col, row, col, text, formats.get(a1))
                continue
            if a1 in legacy:
                ws.write_formula(row, col, _formula_text(raw, tables), formats.get(a1))
                continue
            if isinstance(raw, dict) and raw.get("t") == "e" and raw.get("stored"):
                # A placeholder formula cell with an error result; `_store_errors` swaps it for the constant.
                ws.write_formula(row, col, "=0", formats.get(a1), "#N/A")
                patches.stored.append((number, xlsxwriter.utility.xl_rowcol_to_cell(row, col), str(raw["v"])))
                continue
            if isinstance(raw, dict) and "entered" in raw:
                if a1 in formats:
                    msg = f"entered cell {a1} cannot have a number format (Excel picks it)"
                    raise ValueError(msg)
                entered.append(
                    (sheet["name"], xlsxwriter.utility.xl_rowcol_to_cell(row, col), str(raw["entered"]))
                )
                continue
            if (
                cell_number := _write(ws, row, col, raw, formats.get(a1), tables=tables)
            ) is not None and _is_lossy(cell_number):
                patches.lossy.append((number, xlsxwriter.utility.xl_rowcol_to_cell(row, col), cell_number))
        for a1, fmt in formats.items():
            if a1 not in sheet["cells"]:
                ws.write_blank(*parse_a1(a1), None, fmt)
        patches.data_tables.extend(
            (number, _add_data_table(ws, sheet, t)) for t in sheet.get("data_tables", ())
        )

    patches.pivots = pivot_tables(spec, _pivot_cell_writer(worksheets, patches, tables))
    for name, formula in spec.get("names", {}).items():
        wb.define_name(name, _formula_text(formula, tables))
    for n, entry in enumerate(entered, start=1):
        _set_json_property(wb, f"{ENTERED_PROPERTY}.{n}", entry, f"entered cell {entry[0]}!{entry[1]}")
    for n, op in enumerate(spec.get("sheet_ops", ()), start=1):
        _set_json_property(wb, f"{SHEET_OP_PROPERTY}.{n}", list(sheet_op(op)), f"sheet op {n}")
    wb.close()
    return _patch_package(buf.getvalue(), spec, patches)


@dataclass
class _Patches:
    """What `build_xlsx` writes into the package after XlsxWriter, which has no API for it."""

    stored: list[tuple[int, str, str]] = field(default_factory=list)  # (sheet number, cell ref, error name)
    # numbers XlsxWriter would round: (sheet number, cell ref, value)
    lossy: list[tuple[int, str, float]] = field(default_factory=list)
    # (table name, `<autoFilter>` element or "") in add order
    filters: list[tuple[str, str]] = field(default_factory=list)
    # (sheet number, (top-left cell, `<f>`)), T-0428
    data_tables: list[tuple[int, tuple[str, str]]] = field(default_factory=list)
    pivots: PivotParts = field(default_factory=PivotParts)  # T-1705


def _patch_package(xlsx: bytes, spec: dict[str, Any], patches: _Patches) -> bytes:
    """Writes what XlsxWriter cannot into the package `build_xlsx` wrote."""
    xlsx = _store_errors(xlsx, patches.stored) if patches.stored else xlsx
    xlsx = _filter_tables(xlsx, patches.filters) if any(el for _, el in patches.filters) else xlsx
    # Data tables and calculation settings are no-ops when the spec has none (T-0428).
    xlsx = _write_calculation(_write_data_tables(xlsx, patches.data_tables), spec.get("calculation"))
    xlsx = _write_pivots(xlsx, patches.pivots)
    return _store_numbers(xlsx, patches.lossy) if patches.lossy else xlsx


def _date_1904(spec: dict[str, Any]) -> bool:
    """Whether the spec asks for the 1904 date system (`"date_system": 1904`; 1900 is the default)."""
    system = spec.get("date_system", 1900)
    if system not in (1900, 1904) or isinstance(system, bool):
        msg = f"date_system must be 1900 or 1904: {system!r}"
        raise ValueError(msg)
    return bool(system == 1904)


def _number_formats(wb: Any, sheet: dict[str, Any]) -> dict[str, Any]:
    """The sheet's `number_formats` (A1 → format code) as XlsxWriter formats, one per code."""
    by_code: dict[str, Any] = {}
    formats: dict[str, Any] = {}
    for a1, code in sheet.get("number_formats", {}).items():
        parse_a1(a1)
        if not isinstance(code, str) or not code:
            msg = f"number format of {a1} must be a non-empty string: {code!r}"
            raise ValueError(msg)
        if code not in by_code:
            by_code[code] = wb.add_format({"num_format": code})
        formats[a1] = by_code[code]
    return formats


def _hidden_rows(sheet: dict[str, Any]) -> set[int]:
    """The sheet's 1-based `hidden_rows`, checked against Excel's row limit."""
    rows = sheet.get("hidden_rows", [])
    if not all(isinstance(r, int) and not isinstance(r, bool) and 1 <= r <= 1_048_576 for r in rows):
        msg = f"hidden_rows must be row numbers 1..1048576: {rows!r}"
        raise ValueError(msg)
    return set(rows)


def _range(ref: str) -> tuple[int, int, int, int]:
    """'A1:B6' → zero-based (first row, first col, last row, last col)."""
    first, _, last = ref.partition(":")
    (r1, c1), (r2, c2) = parse_a1(first), parse_a1(last or first)
    if r1 >= r2 or c1 > c2:
        msg = f"table ref {ref!r} needs a header row and at least one data row"
        raise ValueError(msg)
    return r1, c1, r2, c2


def _filter_value(raw: Any, a1: str) -> str:
    """The text a values filter lists for a first-column cell (General format: 3 → "3")."""
    if isinstance(raw, str) and raw and not raw.startswith("="):
        return raw
    if isinstance(raw, int | float) and not isinstance(raw, bool) and float(raw).is_integer():
        return str(int(raw))
    msg = f"filtered table column cell {a1} must be literal text or a whole number, not {raw!r}"
    raise ValueError(msg)


def _add_table(
    ws: Any, sheet: dict[str, Any], table: dict[str, Any], hidden: set[int], tables: frozenset[str]
) -> tuple[str, str]:
    """Adds a spec table; returns its name and, when it is filtered, the `<autoFilter>` element to patch in.
    With `"totals_row": true` the last row of `ref` is the table's totals row, written by
    `_totals_column`."""
    name, ref = str(table["name"]), str(table["ref"])
    r1, c1, r2, c2 = _range(ref)
    totals = bool(table.get("totals_row"))
    if totals and r2 - r1 < 2:
        msg = f"table {name} at {ref} needs a header row, a data row and the totals row"
        raise ValueError(msg)
    cells = sheet["cells"]
    cell = xlsxwriter.utility.xl_rowcol_to_cell
    headers = [cells.get(cell(r1, c)) for c in range(c1, c2 + 1)]
    if not all(isinstance(h, str) and h and not h.startswith("=") for h in headers):
        msg = f"table {name} header row {ref} must be literal text: {headers!r}"
        raise ValueError(msg)
    # XlsxWriter warns and returns a negative code (-1, -2, ...) instead of raising.
    columns: list[dict[str, Any]] = [{"header": h} for h in headers]
    if totals:
        for options_column, c in zip(columns, range(c1, c2 + 1), strict=True):
            options_column.update(_totals_column(cells.get(cell(r2, c)), cell(r2, c), tables))
    options = {"name": name, "columns": columns, "total_row": totals}
    if ws.add_table(r1, c1, r2, c2, options):
        msg = f"XlsxWriter rejected table {name} at {ref} (see its warning)"
        raise ValueError(msg)
    if not table.get("filter"):
        return name, ""
    kept: set[str] = set()
    dropped: set[str] = set()
    last_data = r2 - 1 if totals else r2
    for row in range(r1 + 1, last_data + 1):
        a1 = cell(row, c1)
        (dropped if row + 1 in hidden else kept).add(_filter_value(cells.get(a1), a1))
    if kept & dropped:
        msg = f"table {name}: values {sorted(kept & dropped)} are both kept and hidden by the filter"
        raise ValueError(msg)
    values = "".join(f'<filter val="{escape(v, {chr(34): "&quot;"})}"/>' for v in sorted(kept))
    ref_abs = f"{cell(r1, c1)}:{cell(last_data, c2)}"
    column = f'<filterColumn colId="0"><filters>{values}</filters></filterColumn>'
    return name, f'<autoFilter ref="{ref_abs}">{column}</autoFilter>'


def _totals_column(raw: Any, a1: str, tables: frozenset[str]) -> dict[str, str]:
    """XlsxWriter's options for one totals-row cell: a label as `totalsRowLabel`, a formula as a
    `custom` `totalsRowFunction` with its `totalsRowFormula` (XlsxWriter writes the cell itself, as a
    plain `<f>`). Excel for the web refuses a totals row whose cells lack that metadata as corrupt
    (T-1607, docs/excel-semantics/xlsx-storage.md)."""
    if raw is None:
        return {}
    if isinstance(raw, str) and raw.startswith("="):
        return {"total_function": _formula_text(raw, tables)}
    if isinstance(raw, str) and raw:
        return {"total_string": raw}
    msg = f"totals row cell {a1} must be a formula or literal text, not {raw!r}"
    raise ValueError(msg)


def _totals_cells(sheet: dict[str, Any]) -> set[str]:
    """The cells of the sheet's totals rows, which `_add_table` writes."""
    found: set[str] = set()
    for table in sheet.get("tables", ()):
        if table.get("totals_row"):
            _, c1, r2, c2 = _range(str(table["ref"]))
            found.update(xlsxwriter.utility.xl_rowcol_to_cell(r2, c) for c in range(c1, c2 + 1))
    return found


def _add_data_table(ws: Any, sheet: dict[str, Any], table: dict[str, Any]) -> tuple[str, str]:
    """Writes a data table's placeholder cells; returns its top-left cell and the `<f>` it gets.

    excel-graph reads a one-variable column table written as `<f t="dataTable" ref=… dt2D="0" dtr="0" r1=input>`,
    a row table with `dtr="1"`, and a two-variable table with `dt2D="1" dtr="1" r1=row r2=column`.
    """
    first, _, last = table["ref"].partition(":")
    (r1, c1), (r2, c2) = parse_a1(first), parse_a1(last or first)
    # The normalized range, so nothing but a range reaches the attribute.
    ref = f"{xlsxwriter.utility.xl_rowcol_to_cell(r1, c1)}:{xlsxwriter.utility.xl_rowcol_to_cell(r2, c2)}"
    row_input, column_input = table.get("row_input"), table.get("column_input")
    row_input, column_input = (
        None if a1 is None else xlsxwriter.utility.xl_rowcol_to_cell(*parse_a1(a1))
        for a1 in (row_input, column_input)
    )
    if row_input and column_input:
        inputs = f'dt2D="1" dtr="1" r1="{row_input}" r2="{column_input}"'
    elif row_input or column_input:
        inputs = f'dt2D="0" dtr="{1 if row_input else 0}" r1="{row_input or column_input}"'
    else:
        msg = f"data table {ref} needs a row_input or a column_input"
        raise ValueError(msg)
    for row in range(r1, r2 + 1):
        for col in range(c1, c2 + 1):
            a1 = xlsxwriter.utility.xl_rowcol_to_cell(row, col)
            if a1 in sheet["cells"]:
                msg = f"data table {ref} overlaps cell {a1}"
                raise ValueError(msg)
            ws.write_number(row, col, 0)
    top_left = xlsxwriter.utility.xl_rowcol_to_cell(r1, c1)
    return top_left, f'<f t="dataTable" ref="{ref}" {inputs}/>'


def _write_data_tables(xlsx: bytes, tables: list[tuple[int, tuple[str, str]]]) -> bytes:
    """Puts each data table's `<f t="dataTable">` into its top-left placeholder cell."""
    if not tables:
        return xlsx

    def patch(ref: str, formula: str) -> Callable[[str], str]:
        def apply(text: str) -> str:
            text, count = re.subn(rf'(<c r="{ref}"[^>]*>)', lambda m: m.group(1) + formula, text, count=1)
            if count != 1:
                msg = f"data table cell {ref} not found"
                raise ValueError(msg)
            return text

        return apply

    return _patch_sheets(xlsx, [(number, patch(ref, formula)) for number, (ref, formula) in tables])


def _pivot_cell_writer(
    worksheets: list[Any], patches: _Patches, tables: frozenset[str]
) -> Callable[[int, int, int, str | float], None]:
    """Writes a rendered pivot cell through `_write`, so a pivot's aggregates keep every digit (`_store_numbers`)."""

    def write(number: int, row: int, col: int, value: str | float) -> None:
        cell_number = _write(worksheets[number - 1], row, col, value, tables=tables)
        if cell_number is not None and _is_lossy(cell_number):
            patches.lossy.append((number, xlsxwriter.utility.xl_rowcol_to_cell(row, col), cell_number))

    return write


def _write_pivots(xlsx: bytes, pivots: PivotParts) -> bytes:
    """Adds the pivot parts and links them from the workbook, its sheets and the content types (T-1705)."""
    if not pivots.parts:
        return xlsx

    def insert(before: str, elements: list[str]) -> Callable[[str], str]:
        def apply(text: str) -> str:
            if text.count(before) != 1:
                msg = f"cannot place pivot parts: {before} not found once"
                raise ValueError(msg)
            return text.replace(before, "".join(elements) + before)

        return apply

    def after_calc_pr(text: str) -> str:
        text, count = re.subn(
            r"(<calcPr\b[^>]*/>)",
            lambda m: m.group(1) + f"<pivotCaches>{''.join(pivots.workbook_caches)}</pivotCaches>",
            text,
        )
        if count != 1:
            msg = "cannot place pivot caches: no <calcPr/> in the workbook"
            raise ValueError(msg)
        return text

    existing = set(read_parts(xlsx))
    added = dict(pivots.parts)
    patches: list[tuple[str, Callable[[str], str]]] = [
        ("xl/workbook.xml", after_calc_pr),
        ("xl/_rels/workbook.xml.rels", insert("</Relationships>", pivots.workbook_rels)),
        ("[Content_Types].xml", insert("</Types>", pivots.content_types)),
    ]
    for number, rels in pivots.sheet_rels.items():
        part = f"xl/worksheets/_rels/sheet{number}.xml.rels"
        if part in existing:
            patches.append((part, insert("</Relationships>", rels)))
        else:
            added[part] = relationships_xml(*rels)
    return patch_parts(xlsx, patches, added)


def _write_calculation(xlsx: bytes, calculation: dict[str, Any] | None) -> bytes:
    """Writes the spec's iterative-calculation settings on `<calcPr>`."""
    if calculation is None:
        return xlsx
    unknown = set(calculation) - {"iterate", "iterate_count", "iterate_delta"}
    count, delta = calculation.get("iterate_count", 1), calculation.get("iterate_delta", 0.0)
    if (
        unknown
        or not isinstance(calculation.get("iterate", False), bool)
        or not isinstance(count, int)
        or isinstance(count, bool)
        or count < 1
        or not isinstance(delta, int | float)
        or isinstance(delta, bool)
        or not math.isfinite(delta)
        or delta < 0
    ):
        msg = (
            "calculation takes a boolean iterate, a positive integer iterate_count and a finite "
            f"iterate_delta >= 0: {calculation!r}"
        )
        raise ValueError(msg)
    attributes = ""
    if calculation.get("iterate"):
        attributes += ' iterate="1"'
    if "iterate_count" in calculation:
        attributes += f' iterateCount="{int(calculation["iterate_count"])}"'
    if "iterate_delta" in calculation:
        attributes += f' iterateDelta="{float(calculation["iterate_delta"])!r}"'

    def apply(text: str) -> str:
        text, count = re.subn(r"<calcPr", lambda _: "<calcPr" + attributes, text, count=1)
        if count != 1:
            msg = "workbook.xml has no <calcPr>"
            raise ValueError(msg)
        return text

    return patch_parts(xlsx, [("xl/workbook.xml", apply)])


def _filter_tables(xlsx: bytes, filters: list[tuple[str, str]]) -> bytes:
    """Swaps each filtered table's empty `<autoFilter/>` for its values filter (XlsxWriter writes no table
    filters). `filters` holds `(name, element)` per table in `add_table` order, which is XlsxWriter's
    `tableN.xml` numbering; tables without a filter have an empty element."""

    def patch(name: str, element: str) -> Callable[[str], str]:
        def apply(text: str) -> str:
            text, count = re.subn(r'<autoFilter ref="[^"]*"/>', lambda _: element, text)
            if f' name="{name}"' not in text or count != 1:
                msg = f"cannot filter table {name}: table part or autoFilter not found"
                raise ValueError(msg)
            return text

        return apply

    return patch_parts(
        xlsx, [(f"xl/tables/table{n}.xml", patch(name, el)) for n, (name, el) in enumerate(filters, 1) if el]
    )


def _set_json_property(wb: xlsxwriter.Workbook, name: str, value: object, what: str) -> None:
    text = json.dumps(value, ensure_ascii=False)
    if len(text) > 255:  # Excel's custom-property limit; fail here, not at the oracle
        msg = f"{what} is too long for a custom property ({len(text)} > 255)"
        raise ValueError(msg)
    wb.set_custom_property(name, text)


def _json_properties(xlsx: bytes, prefix: str) -> list[Any]:
    """The JSON values of the custom properties `<prefix>.<n>` in `xlsx`, in order of n.

    Raises ValueError when the package or a property is malformed (xlsx inputs are untrusted).
    """
    try:
        parts = read_parts(xlsx)
        if "docProps/custom.xml" not in parts:
            return []
        custom = parts["docProps/custom.xml"].decode("utf-8")
        # A regex, not an XML parser: the part is untrusted, and only XlsxWriter's own layout matters.
        pattern = re.compile(rf'name="{re.escape(prefix)}\.(\d+)"><vt:lpwstr>(.*?)</vt:lpwstr>', re.S)
        found = sorted(
            (int(n), json.loads(unescape(v, {"&quot;": '"', "&apos;": "'"})))
            for n, v in pattern.findall(custom)
        )
        return [value for _, value in found]
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as e:
        msg = f"malformed {prefix} property: {e}"
        raise ValueError(msg) from e


def entered_cells(xlsx: bytes) -> list[tuple[str, str, str]]:
    """The `(sheet, a1, text)` typed entries `build_xlsx` recorded in `xlsx`, in spec order.

    Raises ValueError when the package or the property is malformed (xlsx inputs are untrusted).
    """
    try:
        return [(str(s), str(a1), str(t)) for s, a1, t in _json_properties(xlsx, ENTERED_PROPERTY)]
    except (TypeError, ValueError) as e:
        msg = f"malformed {ENTERED_PROPERTY} property: {e}"
        raise ValueError(msg) from e


def sheet_op(op: Any) -> tuple[str, str, str | int | None]:
    """A spec's sheet op as `(op, sheet, to)`: `to` is the new name of a rename, the 0-based
    position of a move and None for an add. Raises ValueError for an unknown or malformed op."""
    if isinstance(op, (list, tuple)) and len(op) == 3:
        op = {"op": op[0], "sheet": op[1], **({} if op[2] is None else {"to": op[2]})}
    if not isinstance(op, dict) or op.get("op") not in SHEET_OPS or not isinstance(op.get("sheet"), str):
        msg = f"unknown sheet op {op!r}: expected op {'/'.join(SHEET_OPS)} with a sheet name"
        raise ValueError(msg)
    kind, sheet, to = op["op"], op["sheet"], op.get("to")
    if set(op) - {"op", "sheet", "to"}:
        msg = f"sheet op {op!r} has unknown keys"
        raise ValueError(msg)
    if kind == "add" and to is None:
        return kind, sheet, None
    if kind == "rename" and isinstance(to, str) and to:
        return kind, sheet, to
    if kind == "move" and isinstance(to, int) and not isinstance(to, bool) and to >= 0:
        return kind, sheet, to
    msg = f"sheet op {op!r}: `to` must be absent for add, a name for rename, a position >= 0 for move"
    raise ValueError(msg)


def sheet_ops(xlsx: bytes) -> list[tuple[str, str, str | int | None]]:
    """The `(op, sheet, to)` sheet edits `build_xlsx` recorded in `xlsx`, in spec order (T-1608).

    Raises ValueError when the package or a property is malformed or an op is unknown.
    """
    return [sheet_op(op) for op in _json_properties(xlsx, SHEET_OP_PROPERTY)]


def _patch_sheets(xlsx: bytes, patches: list[tuple[int, Callable[[str], str]]]) -> bytes:
    """Rewrites worksheet XML parts: each `(sheet number, patch)` maps the sheet's text to its new text and
    raises ValueError when its cell is not there (XlsxWriter has no API for what the patches write)."""
    return patch_parts(xlsx, [(f"xl/worksheets/sheet{number}.xml", patch) for number, patch in patches])


def _store_errors(xlsx: bytes, stored: list[tuple[int, str, str]]) -> bytes:
    """Replaces the placeholder cells by stored error constants."""

    def patch(ref: str, name: str) -> Callable[[str], str]:
        def apply(text: str) -> str:
            placeholder = f'<c r="{ref}" t="e"><f>0</f><v>#N/A</v></c>'
            if text.count(placeholder) != 1:
                msg = f"cannot store error {name} in {ref}: placeholder not found"
                raise ValueError(msg)
            return text.replace(placeholder, f'<c r="{ref}" t="e"><v>{escape(name)}</v></c>')

        return apply

    return _patch_sheets(xlsx, [(number, patch(ref, name)) for number, ref, name in stored])


def _is_lossy(value: float) -> bool:
    """True when XlsxWriter's 16-significant-digit text (`{number:.16G}`) is not the same double."""
    return math.isfinite(value) and float(f"{value:.16G}") != value


def _store_numbers(xlsx: bytes, lossy: list[tuple[int, str, float]]) -> bytes:
    """Rewrites the `<v>` of numbers that need 17 digits to the shortest text that parses back to the same
    double, as Excel's own files hold them (T-0289)."""

    def patch(ref: str, value: float) -> Callable[[str], str]:
        def apply(text: str) -> str:
            pattern = re.compile(rf'(<c r="{ref}"(?: s="\d+")?><v>)[^<]*(</v></c>)')
            text, count = pattern.subn(rf"\g<1>{value!r}\g<2>", text)
            if count != 1:
                msg = f"cannot store number {value!r} in {ref}: cell not found"
                raise ValueError(msg)
            return text

        return apply

    return _patch_sheets(xlsx, [(number, patch(ref, value)) for number, ref, value in lossy])


def _listed_formula_cells(sheet: dict[str, Any], key: str) -> set[str]:
    """The cells a sheet lists under `key` (`legacy_formulas`, `bare_function_names`), each checked
    to be a formula cell of the sheet."""
    listed = set(sheet.get(key, ()))
    if unknown := listed - sheet["cells"].keys():
        msg = f"{key} names cells not in the sheet: {sorted(unknown)}"
        raise ValueError(msg)
    for a1 in sorted(listed):
        raw = sheet["cells"][a1]
        if not (isinstance(raw, str) and raw.startswith("=")):
            msg = f"{key} cell {a1} is not a formula"
            raise ValueError(msg)
    return listed


def _formula_text(raw: str, tables: frozenset[str], *, prefix_functions: bool = True) -> str:
    """The formula as written to the file: structured references (to `tables`, upper-cased) in
    file form, spill refs anchored, bound names and (unless `prefix_functions` is false, for
    `bare_function_names` cells) future functions prefixed."""
    text = prefix_bound_names(anchor_spill_refs(table_refs_to_file_form(raw, tables)))
    return prefix_future_functions(text) if prefix_functions else text


def _write(ws: Any, row: int, col: int, raw: Any, fmt: Any = None, *, tables: frozenset[str]) -> float | None:
    """Writes one spec cell (with number format `fmt`, if any; formulas may name `tables`); returns
    the number when it went through `write_number`, else None."""
    if isinstance(raw, bool):
        ws.write_boolean(row, col, raw, fmt)
    elif isinstance(raw, int | float):
        ws.write_number(row, col, float(raw), fmt)
        return float(raw)
    elif isinstance(raw, str):
        if raw.startswith("="):
            ws.write_dynamic_array_formula(row, col, row, col, _formula_text(raw, tables), fmt)
        elif raw == "":
            ws.write_formula(row, col, '=""', fmt)
        else:
            ws.write_string(row, col, raw, fmt)
    elif isinstance(raw, dict):
        tag, value = raw.get("t"), raw.get("v")
        if tag == "s" and value == "":
            ws.write_formula(row, col, '=""', fmt)
        elif tag == "s":
            ws.write_string(row, col, str(value), fmt)
        elif tag == "n":
            ws.write_number(row, col, float(value), fmt)  # type: ignore[arg-type]
            return float(value)  # type: ignore[arg-type]
        elif tag == "b":
            ws.write_boolean(row, col, bool(value), fmt)
        elif tag == "e":
            ws.write_formula(row, col, f"={value}", fmt)
        elif tag == "empty":
            if fmt is not None:
                ws.write_blank(row, col, None, fmt)
        else:
            msg = f"unknown typed cell {raw!r}"
            raise ValueError(msg)
    else:
        msg = f"unsupported cell input {raw!r}"
        raise TypeError(msg)
    return None


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("spec", type=Path)
    ap.add_argument("-o", "--out", type=Path)
    args = ap.parse_args()
    spec = json.loads(args.spec.read_text())
    out = args.out or args.spec.with_name(args.spec.name.removesuffix(".case.json") + ".xlsx")
    out.write_bytes(build_xlsx(spec))
    print(out)


if __name__ == "__main__":
    main()
