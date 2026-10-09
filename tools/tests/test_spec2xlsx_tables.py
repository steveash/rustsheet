"""Hidden rows and filtered tables (T-1036)."""

import io
import zipfile

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
