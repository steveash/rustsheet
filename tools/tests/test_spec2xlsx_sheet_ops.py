"""Sheet edits replayed after load (T-1608)."""

from typing import Any

import pytest

from rustsheet_tools.oracle.backends import OracleError, recorded_sheet_ops
from rustsheet_tools.oracle.package import patch_parts
from rustsheet_tools.spec2xlsx import build_xlsx, sheet_ops


def _spec(ops: list[Any]) -> dict[str, Any]:
    return {
        "sheets": [{"name": "Data", "cells": {"A1": 1}}, {"name": "Calc", "cells": {"A1": "=Data!A1"}}],
        "sheet_ops": ops,
    }


def test_each_op_is_recorded_in_order() -> None:
    ops = [
        {"op": "rename", "sheet": "Data", "to": "Inputs"},
        {"op": "add", "sheet": "New"},
        {"op": "move", "sheet": "Calc", "to": 0},
    ]
    xlsx = build_xlsx(_spec(ops))
    expected = [("rename", "Data", "Inputs"), ("add", "New", None), ("move", "Calc", 0)]
    assert sheet_ops(xlsx) == expected
    assert recorded_sheet_ops(xlsx) == expected


def test_workbook_without_ops_lists_none() -> None:
    assert sheet_ops(build_xlsx({"sheets": [{"name": "S", "cells": {"A1": 1}}]})) == []


@pytest.mark.parametrize(
    "op",
    [
        {"op": "delete", "sheet": "Data"},
        {"op": "rename", "sheet": "Data"},
        {"op": "move", "sheet": "Data", "to": -1},
        {"op": "move", "sheet": "Data", "to": True},
        {"op": "add", "sheet": "New", "to": 1},
        {"op": "add", "sheet": "New", "extra": 1},
        {"op": "add"},
        "add",
    ],
)
def test_unknown_or_malformed_op_is_refused(op: Any) -> None:
    with pytest.raises(ValueError, match="sheet op"):
        build_xlsx(_spec([op]))


def test_backend_refuses_a_malformed_recorded_op() -> None:
    xlsx = build_xlsx(_spec([{"op": "add", "sheet": "New"}]))
    tampered = patch_parts(xlsx, [("docProps/custom.xml", lambda text: text.replace('"add"', '"drop"'))])
    with pytest.raises(OracleError, match="unknown sheet op"):
        recorded_sheet_ops(tampered)
