from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import pytest
import yaml

from rustsheet_tools.gen.generator import load_excel_functions

CATALOG = Path(__file__).resolve().parents[2] / "eval" / "catalog" / "functions.yaml"
SOURCE = CATALOG.with_name("excel-functions.tsv")

# Excel's minimum argument counts. A formula with fewer is a syntax error, and Excel for the web
# then rejects the whole workbook (T-0203: one `=IF(x)` cell failed every gen-m2 case). The counts
# come from Microsoft's syntax lines, vetted in eval/catalog/excel-functions.tsv (T-0040). The
# probes/catalog/availability-* cases only show that Excel accepts each `min_args`; that one fewer
# is rejected was shown only for REDUCE, SCAN and PROB (docs/excel-semantics/syntax.md).
EXCEL_MIN_ARGS = {name: int(row["min_args"]) for name, row in load_excel_functions(SOURCE).items()}


def _required(fn: dict[str, Any]) -> int:
    total = 0
    for p in fn["params"]:
        if p.get("optional"):
            break
        total += int(p.get("min", 1)) if p.get("repeat") else 1
    return total


SIGNATURES = {f["name"]: f for f in yaml.safe_load(CATALOG.read_text())["functions"] if "params" in f}


@pytest.mark.parametrize("name", sorted(SIGNATURES))
def test_catalog_requires_at_least_excels_minimum(name: str) -> None:
    assert name in EXCEL_MIN_ARGS, f"{name} is not in eval/catalog/excel-functions.tsv"
    assert _required(SIGNATURES[name]) == EXCEL_MIN_ARGS[name]


LIFT_KINDS = CATALOG.with_name("lift-kinds.json")
# Typed entry refuses a call below the engine's lifting-table minimum (T-2568), so that minimum
# must never be above Excel's. ACCRINTM's `par` is optional in the engine, required on
# Microsoft's syntax line: the engine is the more permissive, so it never refuses what Excel takes.
LIFT_MIN_BELOW_EXCEL = {"ACCRINTM": 3}


def test_lifting_table_minimums_match_excel() -> None:
    table = json.loads(LIFT_KINDS.read_text())
    mismatched = {
        e["name"]: e["required"]
        for e in table
        if e["name"] in EXCEL_MIN_ARGS and e["required"] != EXCEL_MIN_ARGS[e["name"]]
    }
    assert mismatched == LIFT_MIN_BELOW_EXCEL
    assert all(EXCEL_MIN_ARGS[name] > count for name, count in mismatched.items())
