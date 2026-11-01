"""The harness's catalog view stays in step with the catalog (T-0051, decision 0040)."""

from __future__ import annotations

import json

from rustsheet_tools.catalog_view import VIEW, render


def test_committed_view_matches_the_catalog() -> None:
    assert VIEW.read_text() == render(), "stale: run `cd tools && uv run rustsheet-catalog-view`"


def test_view_keeps_scope_reason_and_oracle() -> None:
    rows = {f["name"]: f for f in json.loads(render())["functions"]}
    assert rows["SUM"] == {
        "name": "SUM",
        "category": "math",
        "milestone": "M2",
        "scope": "in",
        "returns": "number",
        "params": [{"name": "number", "mode": "aggregate", "repeat": 1}],
    }
    assert rows["CALL"]["scope"] == "out" and rows["CALL"]["reason"]
    assert "oracle" not in rows["INFO"] and "oracle" not in rows["SHEETS"]
    assert rows["PI"]["v15"] == "exempt" and rows["PI"]["v15_reason"]
    # Gate A's missing-table count reads `returns` (SPEC-0029, T-3379).
    assert rows["UPPER"]["returns"] == "text"


def test_view_keeps_parameter_modes_and_repeating_groups() -> None:
    rows = {f["name"]: f for f in json.loads(render())["functions"]}
    assert rows["ABS"]["params"] == [{"name": "number", "mode": "value"}]
    assert rows["SUMIFS"]["params"][1] == {"name": "criteria_range", "mode": "range", "repeat": 2}
    assert rows["PI"]["params"] == []
