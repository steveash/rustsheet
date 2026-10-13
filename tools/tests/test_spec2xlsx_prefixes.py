"""LET/LAMBDA bound names get `_xlpm.` and engine-known future functions `_xlfn.` (T-0166)."""

import inspect
import io
import re
import zipfile
from pathlib import Path

import pytest
import xlsxwriter.worksheet

from rustsheet_tools.gen.generator import load_excel_functions
from rustsheet_tools.spec2xlsx import (
    build_xlsx,
    future_function_prefixes,
    prefix_bound_names,
    prefix_future_functions,
)


def test_let_names_are_prefixed() -> None:
    assert prefix_bound_names("=LET(x,A2:B2,x)") == "=LET(_xlpm.x,A2:B2,_xlpm.x)"
    assert prefix_bound_names("=LET(x,1,y,x+1,x*y)") == ("=LET(_xlpm.x,1,_xlpm.y,_xlpm.x+1,_xlpm.x*_xlpm.y)")
    assert prefix_bound_names("=A1:A3 LET(x,A2:B2,x)") == "=A1:A3 LET(_xlpm.x,A2:B2,_xlpm.x)"


def test_non_ascii_names_are_prefixed() -> None:
    # Unprefixed, Excel for the web refused `=LET(é,1,é+1)` as a corrupt file (T-1387).
    assert prefix_bound_names("=LET(é,1,É+1)") == "=LET(_xlpm.é,1,_xlpm.É+1)"
    assert prefix_bound_names("=LAMBDA(ß,[ü],ß+ü)(1)") == "=LAMBDA(_xlpm.ß,_xlop.ü,_xlpm.ß+_xlpm.ü)(1)"


def test_lambda_parameters_and_calls_are_prefixed() -> None:
    assert prefix_bound_names("=LAMBDA(a,b,a+b)(1,2)") == "=LAMBDA(_xlpm.a,_xlpm.b,_xlpm.a+_xlpm.b)(1,2)"
    assert prefix_bound_names("=LET(f,LAMBDA(a,a*a),f(2))") == (
        "=LET(_xlpm.f,LAMBDA(_xlpm.a,_xlpm.a*_xlpm.a),_xlpm.f(2))"
    )


def test_lambda_optional_parameters_are_stored_as_xlop() -> None:
    # Excel for the web stores `[y]` as `_xlop.y` and its uses as `_xlpm.y` (T-1274).
    assert prefix_bound_names("=LAMBDA(x,[y],ISOMITTED(y))(1)") == (
        "=LAMBDA(_xlpm.x,_xlop.y,ISOMITTED(_xlpm.y))(1)"
    )
    assert prefix_bound_names("=LAMBDA([a], [ b ],a+b)()") == "=LAMBDA(_xlop.a,_xlop.b,_xlpm.a+_xlpm.b)()"
    assert prefix_bound_names('=LAMBDA(x,x&"[y]")(1)') == '=LAMBDA(_xlpm.x,_xlpm.x&"[y]")(1)'


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


def test_build_xlsx_writes_optional_parameters_as_excel_does() -> None:
    spec = {"sheets": [{"name": "Sheet1", "cells": {"A1": "=LAMBDA(x,[y],IF(ISOMITTED(y),x,x+y))(1)"}}]}
    sheet = zipfile.ZipFile(io.BytesIO(build_xlsx(spec))).read("xl/worksheets/sheet1.xml").decode()
    assert "_xlfn.LAMBDA(_xlpm.x,_xlop.y,IF(_xlfn.ISOMITTED(_xlpm.y),_xlpm.x,_xlpm.x+_xlpm.y))(1)" in sheet


def test_engine_prefix_list_is_not_empty() -> None:
    prefixes = future_function_prefixes()
    assert {"LET", "TRIMRANGE", "FILTER"} <= prefixes.keys()
    assert "CEILING" not in prefixes  # classic: stored without a prefix (T-0172)


def test_engine_prefix_list_names_only_real_functions() -> None:
    # The list is parsed from xlsx_names.rs with a regex, so a comment that looks like an arm would
    # add a fake name whose calls the builder would then write as `_xlfn.<name>` (#NAME? in Excel).
    excel = load_excel_functions(Path(__file__).resolve().parents[2] / "eval/catalog/excel-functions.tsv")
    assert set(future_function_prefixes()) <= set(excel)


def test_classic_names_xlsxwriter_misreads_keep_their_function() -> None:
    # XlsxWriter's unescaped `.` turned NORMSDIST( into _xlfn.NORM.DIST( (T-0040); the builder's
    # tokenizer stores the classic names bare, as Excel does.
    assert prefix_future_functions("=NORMSDIST(1)+NORMSINV(0.5)") == "=NORMSDIST(1)+NORMSINV(0.5)"
    spec = {"sheets": [{"name": "Sheet1", "cells": {"A1": "=NORMSDIST(0)", "A2": "=NORM.DIST(0,0,1,TRUE)"}}]}
    sheet = zipfile.ZipFile(io.BytesIO(build_xlsx(spec))).read("xl/worksheets/sheet1.xml").decode()
    assert ">NORMSDIST(0)</f>" in sheet
    assert ">_xlfn.NORM.DIST(0,0,1,TRUE)</f>" in sheet


def test_every_call_is_prefixed_whatever_shares_the_formula() -> None:
    # XlsxWriter skipped a whole formula once one call was prefixed, leaving DAYS bare (T-0052).
    spec = {
        "sheets": [
            {
                "name": "Sheet1",
                "cells": {"A1": "=TRIMRANGE(A2)+DAYS(1,2)", "A2": "=NORMSDIST(0)+GAUSS(0)"},
            }
        ]
    }
    sheet = zipfile.ZipFile(io.BytesIO(build_xlsx(spec))).read("xl/worksheets/sheet1.xml").decode()
    assert ">_xlfn.TRIMRANGE(A2)+_xlfn.DAYS(1,2)</f>" in sheet
    assert ">NORMSDIST(0)+_xlfn.GAUSS(0)</f>" in sheet


def test_builder_table_covers_xlsxwriters_list() -> None:
    # With use_future_functions off the builder is the only source of prefixes, so it must know
    # every name XlsxWriter would have prefixed, under the same stored form. This scrapes XlsxWriter's
    # private source (3.2.9, locked in tools/uv.lock): a failure after a version bump may only mean
    # the scrape needs updating, or that XlsxWriter learned a name the table should add.
    source = inspect.getsource(xlsxwriter.worksheet.Worksheet._prepare_formula)
    theirs = {
        name.replace("\\.", "."): stored
        for name, stored in re.findall(
            r're\.sub\(r"\\[bS]?([A-Z0-9._\\]+)\\\(", "([\w.]+)\(", formula\)', source
        )
    }
    theirs["SINGLE"] = theirs.pop("INGLE")  # XlsxWriter's own regex is `\SINGLE\(`
    assert len(theirs) > 100
    for name, stored in theirs.items():
        assert prefix_future_functions(f"={name}(1)") == f"={stored}(1)", name


def test_bare_function_names_cell_keeps_the_typed_name() -> None:
    """T-0416: PERCENTOF is prefixed, except in a cell that probes the bare stored form."""
    spec = {
        "sheets": [
            {
                "name": "S",
                "cells": {"A1": "=PERCENTOF(1,2)", "A2": "=PERCENTOF(1,2)"},
                "bare_function_names": ["A1"],
            }
        ]
    }
    sheet = zipfile.ZipFile(io.BytesIO(build_xlsx(spec))).read("xl/worksheets/sheet1.xml").decode()
    a1, a2 = sheet.split('r="A2"')
    assert ">PERCENTOF(1,2)</f>" in a1
    assert ">_xlfn.PERCENTOF(1,2)</f>" in a2


def test_bare_function_names_cell_must_exist_and_be_a_formula() -> None:
    with pytest.raises(ValueError, match="not in the sheet"):
        build_xlsx({"sheets": [{"name": "S", "cells": {"A1": "=1"}, "bare_function_names": ["B1"]}]})
    with pytest.raises(ValueError, match="must be a"):
        build_xlsx({"sheets": [{"name": "S", "cells": {"A1": 5}, "bare_function_names": ["A1"]}]})
