"""LET/LAMBDA bound names get `_xlpm.` and engine-known future functions `_xlfn.` (T-0166)."""

import io
import zipfile

from rustsheet_tools.spec2xlsx import build_xlsx, prefix_bound_names, prefix_future_functions


def test_let_names_are_prefixed() -> None:
    assert prefix_bound_names("=LET(x,A2:B2,x)") == "=LET(_xlpm.x,A2:B2,_xlpm.x)"
    assert prefix_bound_names("=LET(x,1,y,x+1,x*y)") == ("=LET(_xlpm.x,1,_xlpm.y,_xlpm.x+1,_xlpm.x*_xlpm.y)")
    assert prefix_bound_names("=A1:A3 LET(x,A2:B2,x)") == "=A1:A3 LET(_xlpm.x,A2:B2,_xlpm.x)"


def test_lambda_parameters_and_calls_are_prefixed() -> None:
    assert prefix_bound_names("=LAMBDA(a,b,a+b)(1,2)") == "=LAMBDA(_xlpm.a,_xlpm.b,_xlpm.a+_xlpm.b)(1,2)"
    assert prefix_bound_names("=LET(f,LAMBDA(a,a*a),f(2))") == (
        "=LET(_xlpm.f,LAMBDA(_xlpm.a,_xlpm.a*_xlpm.a),_xlpm.f(2))"
    )


def test_unbound_names_strings_and_sheets_are_left_alone() -> None:
    assert prefix_bound_names('=LET(x,"x",x&y)') == '=LET(_xlpm.x,"x",_xlpm.x&y)'
    assert prefix_bound_names("=LET(x,1,x+x!A1)") == "=LET(_xlpm.x,1,_xlpm.x+x!A1)"
    assert prefix_bound_names("=x+SUM(1)") == "=x+SUM(1)"


def test_future_functions_come_from_the_engine_list() -> None:
    assert prefix_future_functions("=A1:A3 TRIMRANGE(A2:B2)") == "=A1:A3 _xlfn.TRIMRANGE(A2:B2)"
    assert prefix_future_functions("=FILTER(A1:A3,B1:B3)") == "=_xlfn._xlws.FILTER(A1:A3,B1:B3)"
    assert prefix_future_functions("=_xlfn.TAKE(A1,1)") == "=_xlfn.TAKE(A1,1)"
    assert prefix_future_functions('=SUM(1)&"TAKE("&CEILING(1,2)') == '=SUM(1)&"TAKE("&CEILING(1,2)'


def test_build_xlsx_writes_the_file_form() -> None:
    spec = {
        "sheets": [
            {"name": "Sheet1", "cells": {"B1": "=A1:A3 LET(x,A2:B2,x)", "B2": "=A1:A3 TRIMRANGE(A2:B2)"}}
        ]
    }
    sheet = zipfile.ZipFile(io.BytesIO(build_xlsx(spec))).read("xl/worksheets/sheet1.xml").decode()
    assert "_xlfn.LET(_xlpm.x,A2:B2,_xlpm.x)" in sheet
    assert "_xlfn.TRIMRANGE(A2:B2)" in sheet
    assert "_xlfn._xlfn" not in sheet
    assert "_xlpm._xlpm" not in sheet
