"""Workbook date system and per-cell number formats (T-1268)."""

import io
import zipfile

import pytest

from rustsheet_tools.spec2xlsx import build_xlsx


def _parts(spec: dict[str, object]) -> dict[str, str]:
    z = zipfile.ZipFile(io.BytesIO(build_xlsx(spec)))
    return {n: z.read(n).decode() for n in z.namelist() if n.endswith(".xml")}


def test_date_system_1904_is_written_and_1900_is_not() -> None:
    sheets = [{"name": "S", "cells": {"A1": 1}}]
    assert 'date1904="1"' in _parts({"date_system": 1904, "sheets": sheets})["xl/workbook.xml"]
    assert "date1904" not in _parts({"date_system": 1900, "sheets": sheets})["xl/workbook.xml"]
    assert "date1904" not in _parts({"sheets": sheets})["xl/workbook.xml"]


@pytest.mark.parametrize("system", [1901, "1904", True])
def test_other_date_systems_are_refused(system: object) -> None:
    with pytest.raises(ValueError, match="date_system"):
        build_xlsx({"date_system": system, "sheets": [{"name": "S", "cells": {}}]})


def test_number_formats_reach_value_formula_and_blank_cells() -> None:
    parts = _parts(
        {
            "sheets": [
                {
                    "name": "S",
                    "cells": {"A1": -1.5, "B1": "=A1*2", "C1": "x"},
                    "number_formats": {"A1": "[h]:mm", "B1": "[h]:mm", "D4": "yyyy-mm-dd"},
                }
            ]
        }
    )
    styles = parts["xl/styles.xml"]
    assert 'formatCode="[h]:mm"' in styles
    assert 'formatCode="yyyy-mm-dd"' in styles
    sheet = parts["xl/worksheets/sheet1.xml"]
    # One style per format code: A1 and B1 share it; C1 keeps the default; D4 is a styled blank.
    assert '<c r="A1" s="1">' in sheet
    assert '<c r="B1" s="1"' in sheet
    assert '<c r="C1" t="s">' in sheet
    assert '<c r="D4" s="2"/>' in sheet


def test_an_entered_cell_cannot_have_a_number_format() -> None:
    sheet = {"name": "S", "cells": {"A1": {"entered": "(1)"}}, "number_formats": {"A1": "0"}}
    with pytest.raises(ValueError, match="entered"):
        build_xlsx({"sheets": [sheet]})
