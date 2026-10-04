from __future__ import annotations

from pathlib import Path
from typing import Any

from rustsheet_tools.oracle.backends import OracleError
from rustsheet_tools.oracle.calibrate import NO_FUNCTION, calibrate, to_markdown


def _cell(
    cell: str, functions: list[str], verdict: str | None, cached: Any = 1.0, got: Any = 1.0
) -> dict[str, Any]:
    return {
        "sheet": "S",
        "cell": cell,
        "formula": f"{'+'.join(functions) or 'A1+1'}()",
        "functions": functions,
        "cached": {"t": "n", "v": cached},
        "recalculated": None if verdict is None else {"t": "n", "v": got},
        "verdict": verdict,
    }


COMPARISONS = {
    "a.xlsx": {
        "volatile_skipped": 2,
        "cells": [
            _cell("A1", ["SUM"], "pass"),
            _cell("A2", ["SUM", "ROUND"], "numeric_ulp", 0.1, 0.10000000000000002),
            _cell("A3", [], "pass"),
        ],
    },
    "b.xlsx": {
        "volatile_skipped": 0,
        "cells": [_cell("B1", ["DATEVALUE"], "type_mismatch"), _cell("B2", ["X"], None)],
    },
}


def _run(jobs: int = 1) -> Any:
    def recalculated(path: Path) -> Path:
        if path.name == "refused.xlsx":
            raise OracleError("refusing to recalculate: uses SHEETS()\nmore detail")
        return path

    return calibrate(
        [Path("a.xlsx"), Path("b.xlsx"), Path("refused.xlsx")],
        recalculated,
        lambda cached, _: COMPARISONS[cached.name],
        jobs,
    )


def test_totals_and_functions_are_aggregated() -> None:
    cal = _run(jobs=3)
    assert cal.workbooks == 2
    assert cal.refused == {"refused.xlsx": "refusing to recalculate: uses SHEETS()"}
    assert cal.volatile_skipped == 2
    assert cal.total.cells == 5
    assert dict(cal.total.verdicts) == {"pass": 2, "numeric_ulp": 1, "type_mismatch": 1, "missing": 1}
    assert (cal.by_function["SUM"].cells, cal.by_function["SUM"].disagreements) == (2, 1)
    assert cal.by_function["ROUND"].disagreements == 1
    assert cal.by_function[NO_FUNCTION].disagreements == 0
    assert [(d["workbook"], d["cell"]) for d in cal.disagreements] == [
        ("a.xlsx", "A2"),
        ("b.xlsx", "B1"),
        ("b.xlsx", "B2"),
    ]


def test_markdown_report() -> None:
    md = to_markdown(_run(), examples=2)
    assert "- Formula cells compared: 5; agree bit-exact: 2 (40.00%); volatile cells skipped: 2" in md
    assert "| DATEVALUE | 1 | 1 | type_mismatch 1 |" in md
    assert "| SUM | 2 | 1 | numeric_ulp 1 |" in md
    assert "Functions agreeing on every cell: 1." in md  # only "(operators only)"
    assert "- `refused.xlsx`: refusing to recalculate" in md
    assert "| b.xlsx | S!B1 |" in md
    assert "S!B2" not in md  # beyond the example limit
