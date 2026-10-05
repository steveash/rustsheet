"""Case-spec defined names are written as workbook-scoped names (T-0043)."""

import io
import zipfile

from rustsheet_tools.spec2xlsx import build_xlsx


def test_build_xlsx_writes_workbook_names() -> None:
    xlsx = build_xlsx(
        {
            "sheets": [{"name": "Sheet1", "cells": {"A1": 1}}],
            "names": {"isect": "=Sheet1!$A$1:$A$3 Sheet1!$A$2:$B$2"},
        }
    )
    workbook = zipfile.ZipFile(io.BytesIO(xlsx)).read("xl/workbook.xml").decode()
    assert '<definedName name="isect">Sheet1!$A$1:$A$3 Sheet1!$A$2:$B$2</definedName>' in workbook


def test_build_xlsx_without_names_has_none() -> None:
    xlsx = build_xlsx({"sheets": [{"name": "Sheet1", "cells": {"A1": 1}}]})
    workbook = zipfile.ZipFile(io.BytesIO(xlsx)).read("xl/workbook.xml").decode()
    assert "definedName" not in workbook
