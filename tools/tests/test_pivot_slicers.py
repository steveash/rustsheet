"""Slicers, timelines, a grouped field's raw groupItems and raw record values in spec cases (T-4767)."""

import io
import zipfile
from typing import Any

import pytest

from rustsheet_tools.spec2xlsx import build_xlsx

_DATA = {"A1": "Region", "B1": "Date", "C1": "Sales", "A2": "East", "B2": 43480, "C2": 1,
         "A3": "West", "B3": 43530, "C3": 2, "A4": "East", "B4": 43600, "C4": 4}  # fmt: skip


def _pivot(name: str, **definition: Any) -> dict[str, Any]:
    return {"name": name, "source": "Data!A1:C4", "location": "A3",
            "definition": {"rows": ["Region"], "data": [{"field": "Sales"}], **definition}}  # fmt: skip


def _parts(*sheets: dict[str, Any]) -> dict[str, str]:
    spec = {"sheets": [{"name": "Data", "cells": dict(_DATA)}, *sheets]}
    z = zipfile.ZipFile(io.BytesIO(build_xlsx(spec)))
    return {n: z.read(n).decode() for n in z.namelist()}


def test_a_slicer_is_a_cache_a_slicers_part_a_drawing_frame_and_the_pivot_caches_id() -> None:
    parts = _parts(
        {
            "name": "S",
            "cells": {},
            "pivot_tables": [_pivot("P_S")],
            "slicers": [
                {
                    "pivot": "P_S",
                    "field": "Region",
                    "anchor": "E3",
                    "selected": ["West"],
                    "tabular": {"sortOrder": "descending"},
                }
            ],
        },
    )
    cache = parts["xl/slicerCaches/slicerCache1.xml"]
    assert 'name="Slicer_Region" sourceName="Region"' in cache
    assert '<pivotTable tabId="2" name="P_S"/>' in cache
    assert (
        '<tabular pivotCacheId="1" sortOrder="descending"><items count="2"><i x="0"/><i x="1" s="1"/>'
        in cache
    )
    assert (
        '<slicer name="Region" cache="Slicer_Region" caption="Region" rowHeight="241300"/>'
        in parts["xl/slicers/slicer2.xml"]
    )
    assert '<x14:pivotCacheDefinition pivotCacheId="1"/>' in parts["xl/pivotCache/pivotCacheDefinition1.xml"]
    workbook = parts["xl/workbook.xml"]
    assert '<definedName name="Slicer_Region">#N/A</definedName>' in workbook
    assert '<x14:slicerCaches><x14:slicerCache r:id="rIdSlicerCache1"/></x14:slicerCaches>' in workbook
    sheet = parts["xl/worksheets/sheet2.xml"]
    assert '<drawing r:id="rIdSlicerDrawing2"/>' in sheet and '<x14:slicer r:id="rIdSlicer2_1"/>' in sheet
    assert 'Requires="a14"' in parts["xl/drawings/drawing2.xml"]
    assert "/xl/slicerCaches/slicerCache1.xml" in parts["[Content_Types].xml"]


def test_a_timeline_and_a_second_slicer_on_a_field_get_names_of_their_own() -> None:
    parts = _parts(
        {
            "name": "T",
            "cells": {},
            "pivot_tables": [_pivot("P_T")],
            "timelines": [
                {
                    "pivot": "P_T",
                    "field": "Date",
                    "anchor": "E3",
                    "bounds": [43466, 44197],
                    "selection": [43466, 43555],
                    "timeline_pivot_cache": {"timelineData": False},
                }
            ],
            "slicers": [{"pivot": "P_T", "field": "Region", "anchor": "E20"}],
        },
        {
            "name": "U",
            "cells": {},
            "pivot_tables": [_pivot("P_U")],
            "slicers": [{"pivot": "P_U", "field": "Region", "anchor": "E3"}],
        },
    )
    timeline = parts["xl/timelineCaches/timelineCache2.xml"]
    assert (
        'filterType="dateBetween"><selection startDate="2019-01-01T00:00:00" endDate="2019-03-31T00:00:00"/>'
        in (timeline)
    )
    assert '<x15:timelineCacheRef r:id="rIdSlicerCache2"/>' in parts["xl/workbook.xml"]
    assert (
        '<x15:timelinePivotCacheDefinition timelineData="0"/>'
        in parts["xl/pivotCache/pivotCacheDefinition1.xml"]
    )
    assert 'cache="Slicer_Region1"' in parts["xl/slicers/slicer3.xml"]
    assert '<slicer name="Region 1"' in parts["xl/slicers/slicer3.xml"]
    assert 'Requires="tsle"' in parts["xl/drawings/drawing2.xml"]


def test_a_table_slicer_names_its_table_and_column() -> None:
    parts = _parts(
        {"name": "TS", "cells": dict(_DATA), "tables": [{"name": "Sales", "ref": "A1:C4"}],
         "slicers": [{"table": "Sales", "column": "Date", "anchor": "F3", "table_slicer": {"sortOrder": "descending"}}]},
    )  # fmt: skip
    assert (
        '<x15:tableSlicerCache tableId="1" column="2" sortOrder="descending"/>'
        in parts["xl/slicerCaches/slicerCache1.xml"]
    )
    assert "{46BE6895-7355-4a93-B00E-2C351335B9C9}" in parts["xl/workbook.xml"]
    assert 'Requires="sle15"' in parts["xl/drawings/drawing2.xml"]


@pytest.mark.parametrize(
    ("entry", "message"),
    [
        ({"pivot": "P_S", "field": "Region", "anchor": "E3", "colour": 1}, "is an object with keys"),
        ({"pivot": "P_S", "field": "Region"}, "names a pivot or a table, and an anchor"),
        ({"pivot": "Nope", "field": "Region", "anchor": "E3"}, "has no listed field"),
        ({"pivot": "P_S", "field": "Region", "anchor": "E3", "selected": ["North"]}, "not in field"),
        ({"pivot": "P_S", "field": "Region", "anchor": "E3", "tabular": {"pivotCacheId": 3}}, "may not set"),
        ({"pivot": "P_S", "field": "Region", "anchor": "E3", "cache_ext": "<x/>"}, "one <extLst> element"),
        ({"table": "Nope", "column": "Region", "anchor": "E3"}, "has no column"),
    ],
)
def test_slicer_refusals(entry: dict[str, Any], message: str) -> None:
    with pytest.raises(ValueError, match=message):
        _parts({"name": "S", "cells": {}, "pivot_tables": [_pivot("P_S")], "slicers": [entry]})


@pytest.mark.parametrize(
    ("entry", "message"),
    [
        ({"pivot": "P_T", "field": "Date", "anchor": "E3"}, "bounds"),
        ({"pivot": "P_T", "field": "Date", "anchor": "E3", "bounds": [1, True]}, "date serial"),
        ({"pivot": "P_T", "field": "Nope", "anchor": "E3", "bounds": [1, 2]}, "one of its fields"),
        (
            {"pivot": "P_T", "field": "Date", "anchor": "E3", "bounds": [1, 2], "pivot_filter": "<x/>"},
            "one <timeline",
        ),
    ],
)
def test_timeline_refusals(entry: dict[str, Any], message: str) -> None:
    with pytest.raises(ValueError, match=message):
        _parts({"name": "T", "cells": {}, "pivot_tables": [_pivot("P_T")], "timelines": [entry]})


def test_a_raw_group_items_slot_replaces_the_groups_items_and_needs_a_group() -> None:
    items = '<groupItems count="2"><b v="1"/><m/></groupItems>'
    group = {"start": 0, "end": 4, "interval": 2}
    parts = _parts(
        {"name": "G", "cells": {}, "pivot_tables": [
            _pivot("P_G", rows=["Sales"], fields={"Sales": {"group": group, "xml": {"groupItems": items}}})]},
    )  # fmt: skip
    assert f'groupInterval="2"/>{items}</fieldGroup>' in parts["xl/pivotCache/pivotCacheDefinition1.xml"]
    with pytest.raises(ValueError, match="has no group for its xml groupItems"):
        _parts({"name": "G", "cells": {}, "pivot_tables": [
            _pivot("P_G", fields={"Region": {"xml": {"groupItems": items}}})]})  # fmt: skip


def test_records_xml_replaces_one_record_value() -> None:
    raw = '<n v="1"><tpls c="1"><tpl fld="0" hier="0" item="0"/></tpls></n>'
    parts = _parts(
        {"name": "R", "cells": {}, "pivot_tables": [
            _pivot("P_R", records_xml=[{"record": 0, "field": "Sales", "xml": raw}])]},
    )  # fmt: skip
    assert f'<r><x v="0"/><n v="43480"/>{raw}</r><r>' in parts["xl/pivotCache/pivotCacheRecords1.xml"]


@pytest.mark.parametrize(
    ("entry", "message"),
    [
        ({"record": 3, "field": "Sales", "xml": "<m/>"}, "is not in the cache"),
        ({"record": True, "field": "Sales", "xml": "<m/>"}, "is not in the cache"),
        ({"record": 0, "field": "Nope", "xml": "<m/>"}, "is not in the cache"),
        ({"record": 0, "field": "Sales", "xml": "<r/>"}, "each entry is one of"),
        ({"record": 0, "field": "Sales", "xml": "<m/><m/>"}, "one <m> element"),
        ({"record": 0, "field": "Sales"}, "entries are"),
    ],
)
def test_records_xml_refusals(entry: dict[str, Any], message: str) -> None:
    with pytest.raises(ValueError, match=message):
        _parts({"name": "R", "cells": {}, "pivot_tables": [_pivot("P_R", records_xml=[entry])]})
