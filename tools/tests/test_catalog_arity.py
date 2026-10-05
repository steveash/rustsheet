from __future__ import annotations

from pathlib import Path

import pytest
import yaml

CATALOG = Path(__file__).resolve().parents[2] / "eval" / "catalog" / "functions.yaml"

# Excel's minimum argument counts. A formula with fewer is a syntax error, and Excel for the web
# then rejects the whole workbook (T-0203: one `=IF(x)` cell failed every gen-m2 case).
EXCEL_MIN_ARGS = {
    "IF": 2,
    "IFERROR": 2,
    "ROUND": 2,
    "ROUNDUP": 2,
    "ROUNDDOWN": 2,
    "VLOOKUP": 3,
    "HLOOKUP": 3,
    "INDEX": 2,
    "MATCH": 2,
    "SUMIF": 2,
    "COUNTIF": 2,
    "LEFT": 1,
    "RIGHT": 1,
    "MID": 3,
    "LEN": 1,
    "YEAR": 1,
    "MONTH": 1,
    "DAY": 1,
    "NOT": 1,
    "ABS": 1,
}


def _required(fn: dict) -> int:  # type: ignore[type-arg]
    total = 0
    for p in fn["params"]:
        if p.get("optional"):
            break
        total += int(p.get("min", 1)) if p.get("repeat") else 1
    return total


@pytest.mark.parametrize("name", sorted(EXCEL_MIN_ARGS))
def test_catalog_requires_at_least_excels_minimum(name: str) -> None:
    catalog = {f["name"]: f for f in yaml.safe_load(CATALOG.read_text())["functions"]}
    if name not in catalog:
        pytest.skip(f"{name} not in the catalog")
    assert _required(catalog[name]) == EXCEL_MIN_ARGS[name]
