"""Spill references are written in their file form (T-0042)."""

import io
import zipfile

from rustsheet_tools.spec2xlsx import anchor_spill_refs, build_xlsx


def test_rewrites_spill_references_outside_strings() -> None:
    assert anchor_spill_refs("=SUM(A1#)") == "=SUM(_xlfn.ANCHORARRAY(A1))"
    assert anchor_spill_refs("=$A$1#+B2#") == "=_xlfn.ANCHORARRAY($A$1)+_xlfn.ANCHORARRAY(B2)"
    assert anchor_spill_refs("=Sheet2!A1#") == "=_xlfn.ANCHORARRAY(Sheet2!A1)"
    assert anchor_spill_refs("='My Sheet'!A1# J2:K2") == ("=_xlfn.ANCHORARRAY('My Sheet'!A1) J2:K2")
    assert anchor_spill_refs('="A1#"&A1#') == '="A1#"&_xlfn.ANCHORARRAY(A1)'
    assert anchor_spill_refs("=SUM(A1:A2)") == "=SUM(A1:A2)"


def test_build_xlsx_stores_the_file_form() -> None:
    xlsx = build_xlsx({"sheets": [{"name": "Sheet1", "cells": {"B1": "=A1#"}}]})
    sheet = zipfile.ZipFile(io.BytesIO(xlsx)).read("xl/worksheets/sheet1.xml").decode()
    assert "_xlfn.ANCHORARRAY(A1)" in sheet
    assert "_xlfn._xlfn" not in sheet
