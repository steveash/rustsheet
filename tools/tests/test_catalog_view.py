"""The harness's catalog view stays in step with the catalog (T-0051, decision 0040)."""

from __future__ import annotations

import json

from rustsheet_tools.catalog_view import VIEW, render


def test_committed_view_matches_the_catalog() -> None:
    assert VIEW.read_text() == render(), "stale: run `cd tools && uv run rustsheet-catalog-view`"


def test_view_keeps_scope_reason_and_oracle() -> None:
    rows = {f["name"]: f for f in json.loads(render())["functions"]}
    assert rows["SUM"] == {"name": "SUM", "category": "math", "milestone": "M2", "scope": "in"}
    assert rows["CALL"]["scope"] == "out" and rows["CALL"]["reason"]
    assert rows["INFO"]["oracle"] == "none"
