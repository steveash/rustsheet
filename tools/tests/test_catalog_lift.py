"""The catalog's modes and `lift: arrays` keys follow the SPEC-0011 argument-kind table (T-1776)."""

from __future__ import annotations

import copy

from rustsheet_tools.catalog_lift import CATALOG, check, load_table, write
from rustsheet_tools.gen.generator import Catalog


def test_catalog_agrees_with_the_lifting_table() -> None:
    assert check(Catalog.load(CATALOG).functions, load_table()) == []


def test_write_leaves_the_committed_catalog_unchanged() -> None:
    text = CATALOG.read_text()
    assert write(text, load_table()) == text, "run `uv run rustsheet-catalog-lift --write`"


def test_a_seeded_mismatch_is_reported() -> None:
    functions = copy.deepcopy(Catalog.load(CATALOG).functions)
    edate = next(f for f in functions if f["name"] == "EDATE")
    del edate["params"][0]["lift"]
    edate["params"][1]["mode"] = "range"
    assert check(functions, load_table()) == [
        "EDATE(start_date): LiftArrays needs `lift: arrays`",
        "EDATE(months): mode range does not fit LiftArrays",
    ]


def test_params_past_the_table_take_the_repeat_kind() -> None:
    functions = copy.deepcopy(Catalog.load(CATALOG).functions)
    imsum = next(f for f in functions if f["name"] == "IMSUM")
    imsum["params"][0]["mode"] = "value"
    assert check(functions, load_table()) == [
        f"IMSUM({imsum['params'][0]['name']}): mode value does not fit WholeByCaller",
    ]
