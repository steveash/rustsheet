"""Pivot tables in spec cases (T-1705), so GETPIVOTDATA can be probed against a real pivot.

A sheet's `"pivot_tables"` lists pivots placed on that sheet:

    {"name": "Pivot1", "source": "Data!A1:C7", "location": "E3",
     "row": "Region", "column": "Product", "value": "Sales", "function": "sum"}

`source` is a range whose first row holds the field names (text) and whose other rows are literal
constants in `cells` (no formulas, no blanks); without a sheet name it is on the pivot's own sheet.
`row` names a text field, `column` (optional) another one, and `value` a number field aggregated by
`function` (`sum`, the default, `count`, `average`, `max` or `min`). Items show in ascending
case-folded order.

The builder renders the pivot as Excel lays out a new compact-form pivot (the caption cells, the
item labels, the aggregates and the grand totals) into the sheet's cells, and writes the pivot
cache definition, its records and the pivot table definition into the package. The cache is marked
`refreshOnLoad`, so Excel rebuilds it from `source` on open. The engine adapter ignores pivot tables
(the engine has none), so a case should only score formulas that read the pivot through
GETPIVOTDATA, never the rendered cells.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any
from xml.sax.saxutils import quoteattr

import xlsxwriter.utility

_MAIN_NS = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
_REL_NS = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
_PACKAGE_REL_NS = "http://schemas.openxmlformats.org/package/2006/relationships"
_CONTENT_TYPE = "application/vnd.openxmlformats-officedocument.spreadsheetml.{}+xml"
_FUNCTIONS: dict[str, tuple[str, Callable[[list[float]], float]]] = {
    "sum": ("Sum", sum),
    "count": ("Count", lambda xs: float(len(xs))),
    "average": ("Average", lambda xs: sum(xs) / len(xs)),
    "max": ("Max", max),
    "min": ("Min", min),
}
_KEYS = {"name", "source", "location", "row", "column", "value", "function"}

Cell = str | float


@dataclass
class PivotParts:
    """What the pivots of a workbook add to its package: new parts and the edits to existing ones."""

    parts: dict[str, bytes] = field(default_factory=dict)
    workbook_caches: list[str] = field(default_factory=list)  # `<pivotCache>` elements
    workbook_rels: list[str] = field(default_factory=list)  # `<Relationship>` elements
    content_types: list[str] = field(default_factory=list)  # `<Override>` elements
    sheet_rels: dict[int, list[str]] = field(default_factory=dict)  # sheet number → `<Relationship>`s


def _split_range(ref: str) -> tuple[int, int, int, int]:
    first, _, last = ref.partition(":")
    r1, c1 = xlsxwriter.utility.xl_cell_to_rowcol(first.replace("$", ""))
    r2, c2 = xlsxwriter.utility.xl_cell_to_rowcol((last or first).replace("$", ""))
    if r2 < r1 or c2 < c1:
        msg = f"pivot source {ref} is not top-left to bottom-right"
        raise ValueError(msg)
    return r1, c1, r2, c2


def _source(spec: dict[str, Any], own_sheet: str, source: str) -> tuple[str, str, list[list[Any]]]:
    """The source's sheet name, normalized range and rows of constants, header row first."""
    sheet_name, bang, ref = source.rpartition("!")
    sheet_name = sheet_name.strip("'").replace("''", "'") if bang else own_sheet
    by_name = {s["name"]: s for s in spec["sheets"]}
    if sheet_name not in by_name:
        msg = f"pivot source sheet {sheet_name!r} is not in the spec"
        raise ValueError(msg)
    r1, c1, r2, c2 = _split_range(ref)
    if r2 == r1:
        msg = f"pivot source {source} has no data rows"
        raise ValueError(msg)
    cells = by_name[sheet_name]["cells"]
    rows: list[list[Any]] = []
    for row in range(r1, r2 + 1):
        values = []
        for col in range(c1, c2 + 1):
            a1 = xlsxwriter.utility.xl_rowcol_to_cell(row, col)
            raw = cells.get(a1)
            if (
                isinstance(raw, bool)
                or not isinstance(raw, str | int | float)
                or raw in ("",)
                or (isinstance(raw, str) and raw.startswith("="))
            ):
                msg = f"pivot source cell {sheet_name}!{a1} must be a literal text or number: {raw!r}"
                raise ValueError(msg)
            values.append(raw)
        rows.append(values)
    normalized = (
        f"{xlsxwriter.utility.xl_rowcol_to_cell(r1, c1)}:{xlsxwriter.utility.xl_rowcol_to_cell(r2, c2)}"
    )
    return sheet_name, normalized, rows


def _number(value: float) -> str:
    return str(int(value)) if float(value).is_integer() and abs(value) < 1e15 else repr(float(value))


class _Pivot:
    """One pivot: its fields, items, aggregates and layout."""

    def __init__(self, spec: dict[str, Any], sheet: dict[str, Any], pivot: dict[str, Any]) -> None:
        if unknown := set(pivot) - _KEYS:
            msg = f"pivot table has unknown keys {sorted(unknown)}"
            raise ValueError(msg)
        self.name = str(pivot["name"])
        self.source_sheet, self.source_ref, rows = _source(spec, sheet["name"], str(pivot["source"]))
        header, self.records = rows[0], rows[1:]
        if not all(isinstance(h, str) for h in header) or len(set(header)) != len(header):
            msg = f"pivot source header {header!r} must be distinct texts"
            raise ValueError(msg)
        self.fields: list[str] = header
        self.function = str(pivot.get("function", "sum"))
        if self.function not in _FUNCTIONS:
            msg = f"pivot function {self.function!r} is not one of {sorted(_FUNCTIONS)}"
            raise ValueError(msg)
        self.row = self._field(pivot["row"], text=True)
        self.column = None if pivot.get("column") is None else self._field(pivot["column"], text=True)
        self.value = self._field(pivot["value"], text=False)
        if len({self.row, self.column, self.value} - {None}) != (2 if self.column is None else 3):
            msg = "a pivot's row, column and value fields must differ"
            raise ValueError(msg)
        # Shared items of each text field in order of first appearance, as Excel's cache keeps them.
        self.shared: list[list[str]] = [[] for _ in self.fields]
        for record in self.records:
            for i, value in enumerate(record):
                if isinstance(value, str) and value not in self.shared[i]:
                    self.shared[i].append(value)
        self.top, self.left = xlsxwriter.utility.xl_cell_to_rowcol(str(pivot["location"]).replace("$", ""))

    def _field(self, name: Any, *, text: bool) -> int:
        if name not in self.fields:
            msg = f"pivot field {name!r} is not in the source header {self.fields!r}"
            raise ValueError(msg)
        index = self.fields.index(name)
        if any(isinstance(r[index], str) != text for r in self.records):
            kind = "text" if text else "numbers"
            msg = f"pivot field {name!r} must hold only {kind}"
            raise ValueError(msg)
        return index

    def _order(self, index: int) -> list[str]:
        return sorted(self.shared[index], key=str.casefold)

    def _aggregate(self, row_item: str | None, column_item: str | None) -> float | None:
        values = [
            float(r[self.value])
            for r in self.records
            if (row_item is None or r[self.row] == row_item)
            and (column_item is None or self.column is None or r[self.column] == column_item)
        ]
        return _FUNCTIONS[self.function][1](values) if values else None

    @property
    def caption(self) -> str:
        return f"{_FUNCTIONS[self.function][0]} of {self.fields[self.value]}"

    def cells(self) -> dict[tuple[int, int], Cell]:
        """The rendered compact-form layout, by (row, column)."""
        rows, top, left = self._order(self.row), self.top, self.left
        out: dict[tuple[int, int], Cell] = {}

        def put(row: int, col: int, value: Cell | None) -> None:
            if value is not None:
                out[(row, col)] = value

        if self.column is None:
            put(top, left, "Row Labels")
            put(top, left + 1, self.caption)
            for i, item in enumerate([*rows, None]):
                put(top + 1 + i, left, item if item is not None else "Grand Total")
                put(top + 1 + i, left + 1, self._aggregate(item, None))
            return out
        columns = self._order(self.column)
        put(top, left, self.caption)
        put(top, left + 1, "Column Labels")
        put(top + 1, left, "Row Labels")
        for j, column in enumerate([*columns, None]):
            put(top + 1, left + 1 + j, column if column is not None else "Grand Total")
        for i, item in enumerate([*rows, None]):
            put(top + 2 + i, left, item if item is not None else "Grand Total")
            for j, column in enumerate([*columns, None]):
                put(top + 2 + i, left + 1 + j, self._aggregate(item, column))
        return out

    def location(self) -> str:
        rows = len(self.shared[self.row]) + 1
        if self.column is None:
            bottom, right, first_data_row = self.top + rows, self.left + 1, 1
        else:
            bottom, right, first_data_row = (
                self.top + 1 + rows,
                self.left + 1 + len(self.shared[self.column]),
                2,
            )
        ref = (
            f"{xlsxwriter.utility.xl_rowcol_to_cell(self.top, self.left)}:"
            f"{xlsxwriter.utility.xl_rowcol_to_cell(bottom, right)}"
        )
        return f'<location ref="{ref}" firstHeaderRow="1" firstDataRow="{first_data_row}" firstDataCol="1"/>'

    def cache_definition(self) -> str:
        fields = []
        for i, name in enumerate(self.fields):
            if self.shared[i]:
                items = "".join(f"<s v={quoteattr(v)}/>" for v in self.shared[i])
                shared = f'<sharedItems count="{len(self.shared[i])}">{items}</sharedItems>'
            else:
                numbers = [float(r[i]) for r in self.records]
                integer = ' containsInteger="1"' if all(n.is_integer() for n in numbers) else ""
                shared = (
                    f'<sharedItems containsSemiMixedTypes="0" containsString="0" containsNumber="1"{integer}'
                    f' minValue="{_number(min(numbers))}" maxValue="{_number(max(numbers))}"/>'
                )
            fields.append(f'<cacheField name={quoteattr(name)} numFmtId="0">{shared}</cacheField>')
        return (
            f'<pivotCacheDefinition xmlns="{_MAIN_NS}" xmlns:r="{_REL_NS}" r:id="rId1" refreshOnLoad="1"'
            f' createdVersion="6" refreshedVersion="6" minRefreshableVersion="3" recordCount="{len(self.records)}">'
            f'<cacheSource type="worksheet"><worksheetSource ref="{self.source_ref}"'
            f" sheet={quoteattr(self.source_sheet)}/></cacheSource>"
            f'<cacheFields count="{len(fields)}">{"".join(fields)}</cacheFields></pivotCacheDefinition>'
        )

    def cache_records(self) -> str:
        records = "".join(
            "<r>"
            + "".join(
                f'<x v="{self.shared[i].index(v)}"/>'
                if isinstance(v, str)
                else f'<n v="{_number(float(v))}"/>'
                for i, v in enumerate(record)
            )
            + "</r>"
            for record in self.records
        )
        return (
            f'<pivotCacheRecords xmlns="{_MAIN_NS}" count="{len(self.records)}">{records}</pivotCacheRecords>'
        )

    def _axis_items(self, index: int) -> tuple[str, str]:
        """A field's `<items>` (display order, shared-item indices) and its `<rowItems>`/`<colItems>` body."""
        order = self._order(index)
        items = "".join(f'<item x="{self.shared[index].index(v)}"/>' for v in order)
        lines = "".join("<i><x/></i>" if d == 0 else f'<i><x v="{d}"/></i>' for d in range(len(order)))
        return (
            f'<items count="{len(order) + 1}">{items}<item t="default"/></items>',
            f'{lines}<i t="grand"><x/></i>',
        )

    def table_definition(self, cache_id: int) -> str:
        pivot_fields = []
        for i in range(len(self.fields)):
            if i in (self.row, self.column):
                axis = "axisRow" if i == self.row else "axisCol"
                pivot_fields.append(
                    f'<pivotField axis="{axis}" showAll="0">{self._axis_items(i)[0]}</pivotField>'
                )
            elif i == self.value:
                pivot_fields.append('<pivotField dataField="1" showAll="0"/>')
            else:
                pivot_fields.append('<pivotField showAll="0"/>')
        row_lines = self._axis_items(self.row)[1]
        rows = f'<rowFields count="1"><field x="{self.row}"/></rowFields>'
        rows += f'<rowItems count="{len(self.shared[self.row]) + 1}">{row_lines}</rowItems>'
        if self.column is None:
            columns = '<colItems count="1"><i/></colItems>'
        else:
            column_lines = self._axis_items(self.column)[1]
            columns = f'<colFields count="1"><field x="{self.column}"/></colFields>'
            columns += f'<colItems count="{len(self.shared[self.column]) + 1}">{column_lines}</colItems>'
        subtotal = "" if self.function == "sum" else f' subtotal="{self.function}"'
        return (
            f'<pivotTableDefinition xmlns="{_MAIN_NS}" name={quoteattr(self.name)} cacheId="{cache_id}"'
            ' applyNumberFormats="0" applyBorderFormats="0" applyFontFormats="0" applyPatternFormats="0"'
            ' applyAlignmentFormats="0" applyWidthHeightFormats="1" dataCaption="Values" updatedVersion="6"'
            ' minRefreshableVersion="3" useAutoFormatting="1" itemPrintTitles="1" createdVersion="6" indent="0"'
            ' outline="1" outlineData="1" multipleFieldNames="0">'
            f'{self.location()}<pivotFields count="{len(pivot_fields)}">{"".join(pivot_fields)}</pivotFields>'
            f"{rows}{columns}"
            f'<dataFields count="1"><dataField name={quoteattr(self.caption)} fld="{self.value}"{subtotal}'
            ' baseField="0" baseItem="0"/></dataFields>'
            '<pivotTableStyleInfo name="PivotStyleLight16" showRowHeaders="1" showColHeaders="1"'
            ' showRowStripes="0" showColStripes="0" showLastColumn="1"/></pivotTableDefinition>'
        )


def rels_part(*relationships: str) -> bytes:
    """A relationships part holding `relationships`."""
    body = "".join(relationships)
    return f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<Relationships xmlns="{_PACKAGE_REL_NS}">{body}</Relationships>'.encode()


def _relationship(rid: str, kind: str, target: str) -> str:
    return f'<Relationship Id="{rid}" Type="{_REL_NS}/{kind}" Target="{target}"/>'


def _xml(text: str) -> bytes:
    return f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n{text}'.encode()


def pivot_tables(spec: dict[str, Any], write: Callable[[int, int, int, Cell], None]) -> PivotParts:
    """Renders every pivot of the spec through `write(sheet number, row, col, value)` and returns the
    package parts and edits they need. Raises ValueError for a malformed pivot or one that overlaps
    a spec cell or another pivot."""
    out = PivotParts()
    cache_id = 0
    for number, sheet in enumerate(spec["sheets"], start=1):
        taken = {xlsxwriter.utility.xl_cell_to_rowcol(a1) for a1 in sheet["cells"]}
        for raw in sheet.get("pivot_tables", ()):
            pivot = _Pivot(spec, sheet, raw)
            cells = pivot.cells()
            if overlap := taken & set(cells):
                msg = f"pivot {pivot.name} overlaps cells {sorted(overlap)}"
                raise ValueError(msg)
            taken |= set(cells)
            for (row, col), value in sorted(cells.items()):
                write(number, row, col, value)
            cache_id += 1
            n = cache_id
            definition = f"xl/pivotCache/pivotCacheDefinition{n}.xml"
            records = f"xl/pivotCache/pivotCacheRecords{n}.xml"
            table = f"xl/pivotTables/pivotTable{n}.xml"
            out.parts[definition] = _xml(pivot.cache_definition())
            out.parts[records] = _xml(pivot.cache_records())
            out.parts[table] = _xml(pivot.table_definition(n))
            out.parts[f"xl/pivotCache/_rels/pivotCacheDefinition{n}.xml.rels"] = rels_part(
                _relationship("rId1", "pivotCacheRecords", f"pivotCacheRecords{n}.xml")
            )
            out.parts[f"xl/pivotTables/_rels/pivotTable{n}.xml.rels"] = rels_part(
                _relationship("rId1", "pivotCacheDefinition", f"../pivotCache/pivotCacheDefinition{n}.xml")
            )
            out.workbook_caches.append(f'<pivotCache cacheId="{n}" r:id="rIdPivotCache{n}"/>')
            out.workbook_rels.append(
                _relationship(
                    f"rIdPivotCache{n}", "pivotCacheDefinition", f"pivotCache/pivotCacheDefinition{n}.xml"
                )
            )
            out.sheet_rels.setdefault(number, []).append(
                _relationship(f"rIdPivotTable{n}", "pivotTable", f"../pivotTables/pivotTable{n}.xml")
            )
            for part, kind in (
                (definition, "pivotCacheDefinition"),
                (records, "pivotCacheRecords"),
                (table, "pivotTable"),
            ):
                out.content_types.append(
                    f'<Override PartName="/{part}" ContentType="{_CONTENT_TYPE.format(kind)}"/>'
                )
    return out
