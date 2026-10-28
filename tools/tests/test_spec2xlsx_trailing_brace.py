"""A cell formula ending in an array constant keeps its closing brace in the file (T-3051)."""

import io
import zipfile

from rustsheet_tools.spec2xlsx import build_xlsx


def test_build_xlsx_keeps_a_trailing_array_constant_brace() -> None:
    xlsx = build_xlsx(
        {
            "sheets": [
                {
                    "name": "S",
                    "cells": {"A1": "=1+{2;3}", "A2": "=-{2;3}", "A3": "={1,2}"},
                    "legacy_formulas": ["A2"],
                    "bare_function_names": ["A3"],
                }
            ]
        }
    )
    sheet = zipfile.ZipFile(io.BytesIO(xlsx)).read("xl/worksheets/sheet1.xml").decode()
    assert '<f t="array" ref="A1">1+{2;3}</f>' in sheet
    assert "<f>-{2;3}</f>" in sheet
    assert '<f t="array" ref="A3">{1,2}</f>' in sheet
