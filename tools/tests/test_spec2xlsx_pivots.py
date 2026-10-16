"""Pivot tables in spec cases (T-1705)."""

import io
import re
import zipfile
from typing import Any

import pytest

from rustsheet_tools.spec2xlsx import build_xlsx

_DATA: dict[str, object] = {
    "A1": "Region",
    "B1": "Product",
    "C1": "Sales",
    "A2": "East",
    "B2": "A",
    "C2": 10,
    "A3": "West",
    "B3": "A",
    "C3": 20,
    "A4": "East",
    "B4": "B",
    "C4": 5.5,
}


def _spec(pivot: dict[str, Any], cells: dict[str, object] | None = None) -> dict[str, Any]:
    return {
        "sheets": [
            {"name": "Data", "cells": dict(_DATA)},
            {"name": "P", "cells": cells or {"H1": '=GETPIVOTDATA("Sales",$A$3)'}, "pivot_tables": [pivot]},
        ]
    }


_PIVOT = {
    "name": "Pv",
    "source": "Data!A1:C4",
    "location": "A3",
    "row": "Region",
    "column": "Product",
    "value": "Sales",
}


def _parts(spec: dict[str, Any]) -> dict[str, str]:
    z = zipfile.ZipFile(io.BytesIO(build_xlsx(spec)))
    return {n: z.read(n).decode() for n in z.namelist() if n.endswith((".xml", ".rels"))}


def _cell(sheet_xml: str, a1: str) -> str:
    m = re.search(rf'<c r="{a1}"[^>]*?(?:/>|>.*?</c>)', sheet_xml)
    assert m, a1
    return m.group(0)


def test_pivot_parts_are_linked_and_byte_stable() -> None:
    spec = _spec(_PIVOT)
    assert build_xlsx(spec) == build_xlsx(spec)
    parts = _parts(spec)
    assert (
        '<pivotCaches><pivotCache cacheId="1" r:id="rIdPivotCache1"/></pivotCaches>'
        in parts["xl/workbook.xml"]
    )
    assert 'Target="pivotCache/pivotCacheDefinition1.xml"' in parts["xl/_rels/workbook.xml.rels"]
    assert 'Target="../pivotTables/pivotTable1.xml"' in parts["xl/worksheets/_rels/sheet2.xml.rels"]
    assert 'Target="pivotCacheRecords1.xml"' in parts["xl/pivotCache/_rels/pivotCacheDefinition1.xml.rels"]
    for kind in ("pivotCacheDefinition", "pivotCacheRecords", "pivotTable"):
        assert f"spreadsheetml.{kind}+xml" in parts["[Content_Types].xml"]
    definition = parts["xl/pivotCache/pivotCacheDefinition1.xml"]
    assert '<worksheetSource ref="A1:C4" sheet="Data"/>' in definition
    assert '<sharedItems count="2"><s v="East"/><s v="West"/></sharedItems>' in definition
    assert 'minValue="5.5" maxValue="20"' in definition
    records = parts["xl/pivotCache/pivotCacheRecords1.xml"]
    assert '<r><x v="0"/><x v="0"/><n v="10"/></r>' in records
    assert '<r><x v="0"/><x v="1"/><n v="5.5"/></r>' in records
    table = parts["xl/pivotTables/pivotTable1.xml"]
    assert '<location ref="A3:D7" firstHeaderRow="1" firstDataRow="2" firstDataCol="1"/>' in table
    assert '<dataField name="Sum of Sales" fld="2" baseField="0" baseItem="0"/>' in table


def test_pivot_cells_are_rendered_in_compact_form() -> None:
    # Layout as excel-graph rendered it (probes/getpivotdata-real-pivot, T-1705): caption, column
    # labels, items, aggregates, blank for an empty intersection, grand totals.
    sheet = _parts(_spec(_PIVOT))["xl/worksheets/sheet2.xml"]
    assert 't="s"' in _cell(sheet, "A3")
    assert "<v>15.5</v>" in _cell(sheet, "D5")  # East total: 10 + 5.5
    assert "<v>35.5</v>" in _cell(sheet, "D7")  # grand total
    assert not re.search(r'<c r="C6"', sheet)  # West has no B
    assert "<v>5.5</v>" in _cell(sheet, "C7")


def test_rows_only_average_pivot() -> None:
    pivot = {k: v for k, v in _PIVOT.items() if k != "column"} | {"function": "average", "location": "A10"}
    parts = _parts(_spec(pivot))
    table = parts["xl/pivotTables/pivotTable1.xml"]
    assert '<location ref="A10:B13" firstHeaderRow="1" firstDataRow="1" firstDataCol="1"/>' in table
    assert 'subtotal="average"' in table
    assert '<colItems count="1"><i/></colItems>' in table
    assert "<v>11.833333333333334</v>" in _cell(parts["xl/worksheets/sheet2.xml"], "B13")


@pytest.mark.parametrize(
    ("change", "message"),
    [
        ({"row": "Sales"}, "must hold only text"),
        ({"value": "Region"}, "must hold only numbers"),
        ({"column": "Region"}, "must differ"),
        ({"function": "median"}, "is not one of"),
        ({"source": "Nope!A1:C4"}, "is not in the spec"),
        ({"source": "Data!A1:C1"}, "no data rows"),
        ({"source": "Data!A1:C5"}, "literal text or number"),
        ({"location": "G1"}, "overlaps"),
        ({"filter": "x"}, "unknown keys"),
    ],
)
def test_malformed_pivots_are_refused(change: dict[str, Any], message: str) -> None:
    with pytest.raises(ValueError, match=message):
        build_xlsx(_spec(_PIVOT | change, {"H1": "=1"}))
