"""Typed-entry cells (T-0280)."""

import io
import re
import zipfile
from collections.abc import Callable

import pytest

from rustsheet_tools.spec2xlsx import Entered, build_xlsx, entered_cells


def test_entered_cells_are_blank_and_listed_in_order() -> None:
    spec = {
        "sheets": [{"name": "S", "cells": {"A1": {"entered": "(1)"}, "A2": {"entered": "($1)"}, "B1": "=A1"}}]
    }
    xlsx = build_xlsx(spec)
    sheet = zipfile.ZipFile(io.BytesIO(xlsx)).read("xl/worksheets/sheet1.xml").decode()
    assert '<c r="A1"' not in sheet
    assert entered_cells(xlsx) == [Entered("S", "A1", "(1)"), Entered("S", "A2", "($1)")]


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


def _rewrite_custom(xlsx: bytes, edit: Callable[[bytes], bytes]) -> bytes:
    src = zipfile.ZipFile(io.BytesIO(xlsx))
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as dst:
        for info in src.infolist():
            data = src.read(info.filename)
            dst.writestr(info, edit(data) if info.filename == "docProps/custom.xml" else data)
    return out.getvalue()


def test_long_entry_is_split_over_properties_and_read_back() -> None:
    # T-2709: SUM of 256 arguments typed, past the 255-character custom-property limit.
    formula = "=SUM(" + ",".join(["1"] * 256) + ")"
    spec = {"sheets": [{"name": "S", "cells": {"A1": {"entered": formula}, "A2": {"entered": "=1"}}}]}
    xlsx = build_xlsx(spec)
    custom = zipfile.ZipFile(io.BytesIO(xlsx)).read("docProps/custom.xml").decode()
    values = re.findall(r"<vt:lpwstr>(.*?)</vt:lpwstr>", custom, re.S)
    assert len(values) == 4  # A1 in three pieces, A2 in one
    assert all(len(v) <= 255 for v in values)
    assert entered_cells(xlsx) == [Entered("S", "A1", formula), Entered("S", "A2", "=1")]


def test_split_never_cuts_a_surrogate_pair() -> None:
    # The JSON `["S", "A1", "=\"` and 238 letters are 254 units: the emoji (2 units) would end at 256,
    # so the first piece stops at 254 and the emoji opens the second.
    formula = '="' + "a" * 238 + '\U0001f600&<>"' * 30 + '"'
    xlsx = build_xlsx({"sheets": [{"name": "S", "cells": {"A1": {"entered": formula}}}]})
    custom = zipfile.ZipFile(io.BytesIO(xlsx)).read("docProps/custom.xml").decode()
    assert re.findall(r"<vt:lpwstr>(.*?)</vt:lpwstr>", custom, re.S)[0].endswith("a" * 238)
    assert entered_cells(xlsx) == [Entered("S", "A1", formula)]


def test_missing_piece_is_a_value_error() -> None:
    xlsx = build_xlsx({"sheets": [{"name": "S", "cells": {"A1": {"entered": "=" + "1+" * 400 + "1"}}}]})
    broken = _rewrite_custom(
        xlsx, lambda data: data.replace(b'name="rustsheet.entered.1.2"', b'name="other"')
    )
    with pytest.raises(ValueError, match="missing a piece"):
        entered_cells(broken)


def test_overlong_entry_is_refused() -> None:
    with pytest.raises(ValueError, match="too long"):
        build_xlsx({"sheets": [{"name": "S", "cells": {"A1": {"entered": "x" * 20_000}}}]})


def test_r1c1_entries_carry_their_mode_and_a1_entries_keep_three_elements() -> None:
    # T-3113, decision 0179: an A1 entry's property is unchanged, so older workbooks stay byte-identical.
    spec = {"sheets": [{"name": "S", "cells": {"A1": {"entered": "=1"}, "B2": {"entered_r1c1": "=R1C1+1"}}}]}
    xlsx = build_xlsx(spec)
    custom = zipfile.ZipFile(io.BytesIO(xlsx)).read("docProps/custom.xml").decode()
    values = re.findall(r"<vt:lpwstr>(.*?)</vt:lpwstr>", custom, re.S)
    assert values == ['["S", "A1", "=1"]', '["S", "B2", "=R1C1+1", "r1c1"]']
    assert entered_cells(xlsx) == [Entered("S", "A1", "=1"), Entered("S", "B2", "=R1C1+1", r1c1=True)]


def test_a_cell_cannot_be_entered_in_both_modes() -> None:
    with pytest.raises(ValueError, match="both"):
        build_xlsx({"sheets": [{"name": "S", "cells": {"A1": {"entered": "1", "entered_r1c1": "1"}}}]})


def test_an_unknown_entry_mode_is_a_value_error() -> None:
    xlsx = build_xlsx({"sheets": [{"name": "S", "cells": {"A1": {"entered_r1c1": "=R1C1"}}}]})
    broken = _rewrite_custom(xlsx, lambda data: data.replace(b'"r1c1"]', b'"a1"]'))
    with pytest.raises(ValueError, match="unknown entry mode"):
        entered_cells(broken)
