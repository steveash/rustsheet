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
# must never be above Excel's. ACCRINTM's `par` is optional in the engine's evaluation and required
# on Microsoft's syntax line; typed entry refuses three arguments by the catalog's count
# (functions/arg_counts.txt), as Excel does (T-2632).
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


ARG_COUNTS = (
    Path(__file__).resolve().parents[2]
    / "crates"
    / "rustsheet-engine"
    / "src"
    / "functions"
    / "arg_counts.txt"
)
# Excel's limit on a call's arguments, the maximum of a function with no usable signature.
MAX_ARGS = 255


def _syntax_max(syntax: str) -> int | None:
    """Parameters on a Microsoft syntax line, `None` when it repeats or does not parse."""
    if "..." in syntax or "…" in syntax or "<#>" in syntax or "(" not in syntax:
        return None
    depth = 0
    commas = 0
    inner = []
    for ch in syntax[syntax.index("(") :]:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        elif ch == "," and depth == 1:
            commas += 1
        if depth >= 1:
            inner.append(ch)
    return 0 if "".join(inner).strip("( ") == "" else commas + 1


def _signature_max(fn: dict[str, Any]) -> int:
    """Most arguments a functions.yaml signature takes: a repeated parameter or group counts its `max` cycles."""
    total = 0
    in_group: set[str] = set()
    for p in fn["params"]:
        if p["name"] in in_group:
            continue
        repeat = p.get("repeat")
        if isinstance(repeat, dict):
            group = repeat["group"]
            in_group.update(group)
            total += len(group) * int(repeat["max"])
        elif repeat:
            total += int(p["max"])
        else:
            total += 1
    return min(total, MAX_ARGS)


def expected_arg_counts() -> dict[str, tuple[int, int]]:
    """The engine's typed-entry count table (T-2632): Excel's minimum (excel-functions.tsv) and the
    most arguments its functions.yaml signature takes, else its syntax line's parameters, else 255."""
    rows = load_excel_functions(SOURCE)
    counts = {}
    for name, row in rows.items():
        least = int(row["min_args"])
        if name in SIGNATURES:
            most = _signature_max(SIGNATURES[name])
        else:
            most = _syntax_max(row["syntax"]) if row.get("syntax") else None
            if most is None or most < least:
                most = MAX_ARGS
        counts[name] = (least, most)
    return counts


def test_engine_arg_counts_match_the_catalog() -> None:
    table = {}
    for line in ARG_COUNTS.read_text().splitlines():
        if line.startswith("#"):
            continue
        name, least, most = line.split(" ")
        table[name] = (int(least), int(most))
    assert table == expected_arg_counts()
