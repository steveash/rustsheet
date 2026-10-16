"""Iterative-calculation settings and what-if data tables (T-0428)."""

import io
import zipfile

import pytest

from rustsheet_tools.spec2xlsx import build_xlsx


def _parts(spec: dict[str, object]) -> dict[str, str]:
    xlsx = build_xlsx({"sheets": [{"name": "S", "cells": {"A1": 1, "B1": 2}}]} | spec)
    z = zipfile.ZipFile(io.BytesIO(xlsx))
    return {n: z.read(n).decode() for n in z.namelist() if n.endswith(".xml")}


def test_calculation_settings_are_written_on_calc_pr() -> None:
    workbook = _parts({"calculation": {"iterate": True, "iterate_count": 7, "iterate_delta": 0.5}})[
        "xl/workbook.xml"
    ]
    assert '<calcPr iterate="1" iterateCount="7" iterateDelta="0.5" calcId=' in workbook


def test_no_calculation_leaves_calc_pr_alone() -> None:
    assert "iterate" not in _parts({})["xl/workbook.xml"]


@pytest.mark.parametrize(
    "calculation",
    [{"iterations": 3}, {"iterate_count": 0}, {"iterate_count": 2.5}, {"iterate_delta": float("nan")}],
)
def test_bad_calculation_settings_are_refused(calculation: dict[str, object]) -> None:
    with pytest.raises(ValueError, match="calculation"):
        _parts({"calculation": calculation})


def _sheet(tables: list[dict[str, str]]) -> str:
    spec = {"sheets": [{"name": "S", "cells": {"A1": 1, "B1": 2, "B3": "=A1*10+B1"}, "data_tables": tables}]}
    xlsx = build_xlsx(spec)
    return zipfile.ZipFile(io.BytesIO(xlsx)).read("xl/worksheets/sheet1.xml").decode()


def test_data_tables_get_their_formula_in_the_top_left_cell() -> None:
    sheet = _sheet(
        [
            {"ref": "B4:B6", "column_input": "A1"},
            {"ref": "C8:E8", "row_input": "B1"},
            {"ref": "B10:D11", "row_input": "B1", "column_input": "A1"},
        ]
    )
    assert '<c r="B4"><f t="dataTable" ref="B4:B6" dt2D="0" dtr="0" r1="A1"/><v>0</v></c>' in sheet
    assert '<c r="C8"><f t="dataTable" ref="C8:E8" dt2D="0" dtr="1" r1="B1"/><v>0</v></c>' in sheet
    assert '<f t="dataTable" ref="B10:D11" dt2D="1" dtr="1" r1="B1" r2="A1"/>' in sheet
    assert '<c r="D11"><v>0</v></c>' in sheet


def test_data_tables_are_refused_without_input_or_over_cells() -> None:
    with pytest.raises(ValueError, match="row_input or a column_input"):
        _sheet([{"ref": "B4:B6"}])
    with pytest.raises(ValueError, match="overlaps cell B3"):
        _sheet([{"ref": "B3:B6", "column_input": "A1"}])
