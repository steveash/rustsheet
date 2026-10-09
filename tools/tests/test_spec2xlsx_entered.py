"""Typed-entry cells (T-0280)."""

import io
import zipfile

import pytest

from rustsheet_tools.spec2xlsx import build_xlsx, entered_cells


def test_entered_cells_are_blank_and_listed_in_order() -> None:
    spec = {
        "sheets": [{"name": "S", "cells": {"A1": {"entered": "(1)"}, "A2": {"entered": "($1)"}, "B1": "=A1"}}]
    }
    xlsx = build_xlsx(spec)
    sheet = zipfile.ZipFile(io.BytesIO(xlsx)).read("xl/worksheets/sheet1.xml").decode()
    assert '<c r="A1"' not in sheet
    assert entered_cells(xlsx) == [("S", "A1", "(1)"), ("S", "A2", "($1)")]


def test_workbook_without_entries_lists_none() -> None:
    assert entered_cells(build_xlsx({"sheets": [{"name": "S", "cells": {"A1": 1}}]})) == []


def test_malformed_property_is_a_value_error() -> None:
    xlsx = build_xlsx({"sheets": [{"name": "S", "cells": {"A1": {"entered": "(1)"}}}]})
    src = zipfile.ZipFile(io.BytesIO(xlsx))
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as dst:
        for info in src.infolist():
            data = src.read(info.filename)
            if info.filename == "docProps/custom.xml":
                data = data.replace(b'["S",', b'"S",')
            dst.writestr(info, data)
    with pytest.raises(ValueError, match="malformed"):
        entered_cells(out.getvalue())


def test_overlong_entry_is_refused() -> None:
    with pytest.raises(ValueError, match="too long"):
        build_xlsx({"sheets": [{"name": "S", "cells": {"A1": {"entered": "x" * 300}}}]})
