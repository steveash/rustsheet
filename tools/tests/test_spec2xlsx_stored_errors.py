"""Stored error constants (T-0999)."""

import io
import zipfile

import pytest

from rustsheet_tools.spec2xlsx import _store_errors, build_xlsx


def _sheet(cells: dict[str, object]) -> str:
    xlsx = build_xlsx({"sheets": [{"name": "S", "cells": cells}]})
    return zipfile.ZipFile(io.BytesIO(xlsx)).read("xl/worksheets/sheet1.xml").decode()


def test_stored_error_is_a_constant() -> None:
    sheet = _sheet({"A1": {"t": "e", "v": "#BLOCKED!", "stored": True}, "A2": "=A1"})
    assert '<c r="A1" t="e"><v>#BLOCKED!</v></c>' in sheet
    assert ">A1</f>" in sheet


def test_plain_typed_error_stays_a_formula() -> None:
    sheet = _sheet({"A1": {"t": "e", "v": "#N/A"}})
    assert "<f>#N/A</f>" in sheet
    assert "<v>#BLOCKED!</v>" not in sheet


def test_build_is_deterministic() -> None:
    spec = {"sheets": [{"name": "S", "cells": {"A1": {"t": "e", "v": "#FIELD!", "stored": True}}}]}
    assert build_xlsx(spec) == build_xlsx(spec)


def test_missing_placeholder_is_an_error() -> None:
    xlsx = build_xlsx({"sheets": [{"name": "S", "cells": {"A1": 1}}]})
    with pytest.raises(ValueError, match="placeholder"):
        _store_errors(xlsx, [(1, "A1", "#BLOCKED!")])


def test_legacy_formula_has_no_array_metadata() -> None:
    spec = {"sheets": [{"name": "S", "cells": {"A1": "=(B1,C2)", "A2": "=B1"}, "legacy_formulas": ["A1"]}]}
    sheet = zipfile.ZipFile(io.BytesIO(build_xlsx(spec))).read("xl/worksheets/sheet1.xml").decode()
    assert '<c r="A1"><f>(B1,C2)</f>' in sheet
    assert 'cm="1"' not in sheet.split('r="A2"')[0]
    assert 'r="A2"' in sheet
    assert 'cm="1"' in sheet.split('r="A2"')[1]


def test_legacy_formula_address_must_exist() -> None:
    spec = {"sheets": [{"name": "S", "cells": {"A1": "=1"}, "legacy_formulas": ["B1"]}]}
    with pytest.raises(ValueError, match="not in the sheet"):
        build_xlsx(spec)


def test_legacy_cell_must_be_a_formula() -> None:
    spec = {"sheets": [{"name": "S", "cells": {"A1": 5}, "legacy_formulas": ["A1"]}]}
    with pytest.raises(ValueError, match="not a formula"):
        build_xlsx(spec)
