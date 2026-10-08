"""Numbers reach the workbook as the exact double the case holds (T-0289)."""

import io
import re
import zipfile

import pytest

from rustsheet_tools.spec2xlsx import _store_numbers, build_xlsx


def _values(cells: dict[str, object]) -> dict[str, float]:
    xlsx = build_xlsx({"sheets": [{"name": "S", "cells": cells}]})
    sheet = zipfile.ZipFile(io.BytesIO(xlsx)).read("xl/worksheets/sheet1.xml").decode()
    return {ref: float(v) for ref, v in re.findall(r'<c r="([A-Z]+\d+)"[^>]*><v>([^<]*)</v>', sheet)}


def test_a_17_digit_number_parses_back_to_the_same_double() -> None:
    two_1000 = 2.0**1000
    got = _values({"A1": two_1000, "A2": {"t": "n", "v": two_1000}, "A3": 0.1 + 0.2})
    assert got == {"A1": two_1000, "A2": two_1000, "A3": 0.1 + 0.2}


def test_short_numbers_are_untouched() -> None:
    assert _values({"A1": 1, "A2": 0.5, "A3": -42.0}) == {"A1": 1.0, "A2": 0.5, "A3": -42.0}


def test_build_is_deterministic() -> None:
    spec = {"sheets": [{"name": "S", "cells": {"A1": 2.0**1000}}]}
    assert build_xlsx(spec) == build_xlsx(spec)


def test_missing_cell_is_an_error() -> None:
    xlsx = build_xlsx({"sheets": [{"name": "S", "cells": {"A1": 1}}]})
    with pytest.raises(ValueError, match="cell not found"):
        _store_numbers(xlsx, [(1, "B2", 2.0**1000)])
