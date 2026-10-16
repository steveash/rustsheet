"""Hidden rows and filtered tables (T-1036)."""

import io
import re
import zipfile
from xml.sax.saxutils import unescape

import pytest

from rustsheet_tools.spec2xlsx import build_xlsx

_CELLS: dict[str, object] = {
    "A1": "Key",
    "B1": "Val",
    "A2": "a",
    "B2": 1,
    "A3": 7,
    "B3": 2,
    "A4": "c",
    "B4": 4,
}


def _stored(sheet_xml: str, a1: str) -> str:
    """The `<f>` text of cell `a1`, unescaped."""
    m = re.search(rf'<c r="{a1}"[^>]*><f[^>]*>(.*?)</f>', sheet_xml)
    assert m, a1
    return unescape(m.group(1))


def _parts(sheet: dict[str, object]) -> dict[str, str]:
    xlsx = build_xlsx({"sheets": [{"name": "S", "cells": dict(_CELLS)} | sheet]})
    z = zipfile.ZipFile(io.BytesIO(xlsx))
    return {n: z.read(n).decode() for n in z.namelist() if n.endswith(".xml")}


def test_hidden_rows_are_written_hidden() -> None:
    sheet = _parts({"hidden_rows": [3, 9]})["xl/worksheets/sheet1.xml"]
    assert '<row r="3" spans="1:2" hidden="1">' in sheet
    assert '<row r="9" spans="1:2" hidden="1"/>' in sheet
    assert '<row r="2" spans="1:2">' in sheet


def test_filtered_table_keeps_the_visible_rows() -> None:
    table = _parts({"hidden_rows": [3], "tables": [{"name": "Data", "ref": "A1:B4", "filter": True}]})[
        "xl/tables/table1.xml"
    ]
    assert (
        '<autoFilter ref="A1:B4"><filterColumn colId="0"><filters><filter val="a"/><filter val="c"/>'
        "</filters></filterColumn></autoFilter>"
    ) in table
    assert '<tableColumn id="1" name="Key"/><tableColumn id="2" name="Val"/>' in table


def test_unfiltered_table_has_an_empty_autofilter() -> None:
    parts = _parts({"tables": [{"name": "Plain", "ref": "A1:B4"}]})
    assert '<autoFilter ref="A1:B4"/>' in parts["xl/tables/table1.xml"]


def test_second_table_is_patched_in_its_own_part() -> None:
    spec = {
        "sheets": [
            {"name": "P", "cells": dict(_CELLS), "tables": [{"name": "Plain", "ref": "A1:B4"}]},
            {
                "name": "F",
                "cells": dict(_CELLS),
                "hidden_rows": [2],
                "tables": [{"name": "Data", "ref": "A1:B4", "filter": True}],
            },
        ]
    }
    z = zipfile.ZipFile(io.BytesIO(build_xlsx(spec)))
    assert "<filterColumn" not in z.read("xl/tables/table1.xml").decode()
    assert '<filter val="7"/><filter val="c"/>' in z.read("xl/tables/table2.xml").decode()


@pytest.mark.filterwarnings("ignore:Name looks like a cell name")
@pytest.mark.parametrize(
    ("sheet", "match"),
    [
        ({"hidden_rows": [0]}, "hidden_rows"),
        ({"hidden_rows": [True]}, "hidden_rows"),
        ({"tables": [{"name": "Data", "ref": "A1:B1"}]}, "data row"),
        ({"tables": [{"name": "Data", "ref": "A1:C4"}]}, "header row"),
        ({"tables": [{"name": "T1", "ref": "A1:B4"}]}, "rejected"),
        (
            {"tables": [{"name": "Data", "ref": "A1:B4", "filter": True}], "cells": {**_CELLS, "A3": 2.5}},
            "whole number",
        ),
        (
            {"tables": [{"name": "Data", "ref": "A1:B4", "filter": True}], "cells": {**_CELLS, "A3": "=1"}},
            "whole number",
        ),
    ],
)
def test_invalid_specs_are_rejected(sheet: dict[str, object], match: str) -> None:
    with pytest.raises(ValueError, match=match):
        _parts(sheet)


def test_a_value_both_kept_and_hidden_is_rejected() -> None:
    cells = {**_CELLS, "A4": "a"}
    spec = {
        "sheets": [
            {
                "name": "S",
                "cells": cells,
                "hidden_rows": [4],
                "tables": [{"name": "Data", "ref": "A1:B4", "filter": True}],
            }
        ]
    }
    with pytest.raises(ValueError, match="both kept and hidden"):
        build_xlsx(spec)


@pytest.mark.parametrize(
    ("formula", "stored"),
    [
        # A bare table name is stored as `Data[]`; a stored bare name makes Excel for the web refuse
        # the file (T-1607, docs/excel-semantics/xlsx-storage.md).
        ("=ROWS(Data)", "ROWS(Data[])"),
        ("=ROWS(data)+1", "ROWS(data[])+1"),
        # `@` is stored as `#This Row`: a stored `@` reads as #REF!.
        ("=Data[@Val]*2", "Data[[#This Row],[Val]]*2"),
        ("=Data[@[Val]]", "Data[[#This Row],[Val]]"),
        ("=SUM(Data[@[Key]:[Val]])", "SUM(Data[[#This Row],[Key]:[Val]])"),
        ("=COUNTA(Data[@])", "COUNTA(Data[#This Row])"),
        # File forms already, escapes and text stay as they are.
        ("=SUM(Data[[#Headers],[Key]])", "SUM(Data[[#Headers],[Key]])"),
        ("=SUM(Data[C'#])+Data[[#This Row],[it''s]]", "SUM(Data[C'#])+Data[[#This Row],[it''s]]"),
        ('="Data"&Data[[#Totals],[Val]]', '"Data"&Data[[#Totals],[Val]]'),
        ("=SUM(Data[x'@y])", "SUM(Data[x'@y])"),
        ("=LET(x,SUM(Data['[a]),x+Data[@Val])", "_xlfn.LET(_xlpm.x,SUM(Data['[a]),_xlpm.x+Data[[#This Row],[Val]])"),
    ],
)
def test_structured_references_are_stored_in_file_form(formula: str, stored: str) -> None:
    sheet = _parts({"cells": {**_CELLS, "D3": formula}, "tables": [{"name": "Data", "ref": "A1:B4"}]})
    assert _stored(sheet["xl/worksheets/sheet1.xml"], "D3") == stored


def test_a_name_that_is_not_a_table_is_left_alone() -> None:
    sheet = _parts({"cells": {**_CELLS, "D3": "=ROWS(Other)+Data2"}, "tables": [{"name": "Data", "ref": "A1:B4"}]})
    assert _stored(sheet["xl/worksheets/sheet1.xml"], "D3") == "ROWS(Other)+Data2"


def test_totals_row_is_the_last_row_of_the_table() -> None:
    cells = {**_CELLS, "A5": "Total", "B5": "=SUBTOTAL(109,Data[Val])"}
    parts = _parts({"cells": cells, "tables": [{"name": "Data", "ref": "A1:B5", "totals_row": True}]})
    table = parts["xl/tables/table1.xml"]
    assert 'ref="A1:B5"' in table
    assert 'totalsRowCount="1"' in table
    assert '<autoFilter ref="A1:B4"/>' in table
    # Excel for the web refuses a totals row without its metadata: a label and a custom formula.
    assert '<tableColumn id="1" name="Key" totalsRowLabel="Total"/>' in table
    assert '<tableColumn id="2" name="Val" totalsRowFunction="custom">' in table
    assert "<totalsRowFormula>SUBTOTAL(109,Data[Val])</totalsRowFormula>" in table
    assert '<c r="B5"><f>SUBTOTAL(109,Data[Val])</f>' in parts["xl/worksheets/sheet1.xml"]


def test_filtered_table_with_a_totals_row_filters_only_its_data() -> None:
    cells = {**_CELLS, "A5": "Total"}
    table = _parts(
        {"cells": cells, "hidden_rows": [3], "tables": [{"name": "Data", "ref": "A1:B5", "filter": True, "totals_row": True}]}
    )["xl/tables/table1.xml"]
    assert '<autoFilter ref="A1:B4"><filterColumn colId="0"><filters><filter val="a"/><filter val="c"/>' in table


def test_totals_row_needs_a_data_row() -> None:
    with pytest.raises(ValueError, match="totals row"):
        _parts({"tables": [{"name": "Data", "ref": "A1:B2", "totals_row": True}]})


def test_totals_row_cells_are_labels_or_formulas() -> None:
    with pytest.raises(ValueError, match="totals row cell B4"):
        _parts({"tables": [{"name": "Data", "ref": "A1:B4", "totals_row": True}]})
