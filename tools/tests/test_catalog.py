"""The function catalog lists every Microsoft 365 Excel function exactly once (T-0040)."""

from __future__ import annotations

from collections import Counter
from pathlib import Path
from typing import Any

import yaml

from rustsheet_tools.gen.generator import MILESTONES, load_excel_functions

CATALOG = Path(__file__).resolve().parents[2] / "eval" / "catalog" / "functions.yaml"
SOURCE = CATALOG.with_name("excel-functions.tsv")
CATEGORIES = {
    "math", "logical", "text", "information", "web", "addin", "lookup", "date",
    "statistical", "financial", "engineering", "database", "cube",
}  # fmt: skip


def _functions() -> list[dict[str, Any]]:
    functions: list[dict[str, Any]] = yaml.safe_load(CATALOG.read_text())["functions"]
    return functions


def test_no_function_is_listed_twice() -> None:
    assert not [n for n, k in Counter(f["name"] for f in _functions()).items() if k > 1]


def test_catalog_lists_exactly_the_excel_function_list() -> None:
    names = {f["name"] for f in _functions()}
    source = set(load_excel_functions(SOURCE))
    assert not sorted(source - names), "in excel-functions.tsv but not in the catalog"
    assert not sorted(names - source), "in the catalog but not in excel-functions.tsv"


def test_every_entry_has_category_milestone_and_scope() -> None:
    for f in _functions():
        assert f["category"] in CATEGORIES, f["name"]
        assert f["milestone"] in MILESTONES, f["name"]
        assert f.get("scope", "in") in {"in", "out"}, f["name"]
        # decision 0027: an out-of-scope function says why; an in-scope one has no reason
        assert bool(f.get("reason")) == (f.get("scope") == "out"), f["name"]
        assert f.get("oracle", "none") == "none", f["name"]


def test_source_list_rows_are_well_formed() -> None:
    for name, row in load_excel_functions(SOURCE).items():
        assert name == name.upper() and " " not in name, name
        assert row["min_args"].isdigit(), name
        assert row["page"].startswith("https://support.microsoft.com/"), name
