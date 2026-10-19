"""The parity page generator's bucket logic and output (T-2041), on a small synthetic report."""

from __future__ import annotations

import json
import math
from pathlib import Path
from typing import Any

import pytest

from rustsheet_tools import parity_report
from rustsheet_tools.parity_report import ParityReportError, compute, functions_in, md_cell, table
from rustsheet_tools.parity_svg import Bar, bar_chart, nice_step

ONE_ULP = math.nextafter(1.0, 2.0)
THREE_ULP = math.nextafter(math.nextafter(ONE_ULP, 2.0), 2.0)

SCOPE: dict[str, Any] = {
    "functions": [
        {"name": "ABS", "category": "math", "milestone": "M2", "scope": "in"},
        {"name": "SUM", "category": "math", "milestone": "M2", "scope": "in"},
        {"name": "LN", "category": "math", "milestone": "M5", "scope": "in"},
        {"name": "BETA.DIST", "category": "statistical", "milestone": "M9", "scope": "in"},
        {"name": "UPPER", "category": "text", "milestone": "M5", "scope": "in"},
        {"name": "INFO", "category": "information", "milestone": "M5", "scope": "in"},
        {"name": "CALL", "category": "external", "milestone": "M14", "scope": "out"},
    ]
}


def num(v: float) -> dict[str, Any]:
    return {"t": "n", "v": v}


def cell(formula: str, verdict: str, expected: float = 1.0, actual: float = 1.0) -> dict[str, Any]:
    return {"formula": formula, "verdict": verdict, "expected": num(expected), "actual": num(actual)}


def fn(name: str, cells: int, passed: int, **verdicts: int) -> dict[str, Any]:
    return {"name": name, "cells": cells, "passed": passed, "verdicts": {"pass": passed, **verdicts}}


def report() -> dict[str, Any]:
    """ABS: 1 ulp. LN: one 1-ulp and one 3-ulp cell. SUM: shares ABS's and BETA.DIST's cells.
    BETA.DIST: 1e-9 and an off cell, plus a deviation. UPPER: bit-exact but one unsupported.
    CALL is out of scope."""
    cells = [
        cell("SUM(ABS(A1))", "numeric_ulp", 1.0, ONE_ULP),
        cell("LN(2)", "numeric_ulp", 1.0, ONE_ULP),
        cell('LN(3)&"ABS(x)"', "numeric_ulp", 1.0, THREE_ULP),
        cell("_xlfn.BETA.DIST(0.5,2,3,TRUE)", "numeric_close", 0.1, 0.1000000001),
        cell("SUM(BETA.DIST(0.5,2,3,TRUE))", "type_mismatch"),
        cell("UPPER(CALL(1))", "unsupported"),
        cell("CALL(1)", "numeric_mismatch"),
    ]
    return {
        "functions": [
            fn("ABS", 5, 4, numeric_ulp=1),
            fn("SUM", 4, 2, numeric_ulp=1, type_mismatch=1),
            fn("LN", 6, 4, numeric_ulp=2),
            fn("BETA.DIST", 6, 3, numeric_close=1, type_mismatch=1, deviation=1),
            fn("UPPER", 3, 2, unsupported=1),
            fn("CALL", 2, 0, unsupported=1, numeric_mismatch=1),
        ],
        "cases": [{"id": "s/a", "cells": cells[:4]}, {"id": "s/b", "cells": cells[4:]}],
        "suites": [
            {
                "name": "s",
                "cases": 2,
                "cells": 30,
                "cells_passed": 22,
                "verdicts": {"pass": 22, "numeric_ulp": 3, "numeric_close": 1, "unsupported": 1},
            }
        ],
    }


def by_name(r: dict[str, Any] | None = None) -> dict[str, parity_report.FunctionParity]:
    return {f.name: f for f in compute(r or report(), SCOPE).functions}


def test_numeric_ulp_cells_split_at_one_ulp() -> None:
    ln = by_name()["LN"]
    assert (ln.counts["ulp1"], ln.counts["ulp4"]) == (1, 1)
    assert ln.worst == "ulp4"


def test_function_is_placed_by_its_worst_cell() -> None:
    f = by_name()
    assert f["ABS"].worst == "ulp1"
    assert f["BETA.DIST"].worst == "off"
    assert f["UPPER"].worst == "pass"


def test_deviation_counts_as_pass() -> None:
    beta = by_name()["BETA.DIST"]
    assert beta.counts == {"pass": 4, "ulp1": 0, "ulp4": 0, "close": 1, "off": 1}


def test_unsupported_cells_are_dropped() -> None:
    upper = by_name()["UPPER"]
    assert upper.cells == 2
    assert upper.counts["pass"] == 2


def test_a_cell_counts_for_every_function_it_calls() -> None:
    f = by_name()
    assert f["SUM"].counts == {"pass": 2, "ulp1": 1, "ulp4": 0, "close": 0, "off": 1}
    assert f["ABS"].counts["ulp1"] == 1
    # A name inside a string literal is not a call; the _xlfn. prefix is.
    assert functions_in('LN(3)&"ABS(x)"') == {"LN"}
    assert functions_in("_xlfn.BETA.DIST(1)+sum (2)") == {"BETA.DIST", "SUM"}


def test_function_call_regex_edges() -> None:
    # Prefixes are dropped; dotted names stay whole; `""` inside a literal does not end it.
    assert functions_in("_xlfn._xlws.SORT(A1:A3)") == {"SORT"}
    assert functions_in("_xlws.FILTER(A1:A3,B1:B3)") == {"FILTER"}
    assert functions_in("NORM.S.DIST(1,TRUE)+F.TEST(A1:A3,B1:B3)") == {"NORM.S.DIST", "F.TEST"}
    assert functions_in('LEN("say ""SUM(1)"" twice")&ABS(-1)') == {"LEN", "ABS"}
    assert functions_in('"a"""&UPPER("x")') == {"UPPER"}
    assert functions_in("A1+B2") == set()


def test_only_in_scope_catalog_functions_count() -> None:
    p = compute(report(), SCOPE)
    assert "CALL" not in {f.name for f in p.functions}
    assert p.unscored == ["INFO"]
    assert p.in_scope == 6


def test_buckets_that_do_not_sum_to_the_scored_cells_fail_loudly() -> None:
    r = report()
    r["functions"][0]["cells"] = 7
    with pytest.raises(ParityReportError, match="ABS"):
        compute(r, SCOPE)


def test_totals_and_near_off() -> None:
    p = compute(report(), SCOPE)
    assert p.functions_per_bucket() == {"ulp1": 1, "ulp4": 1, "off": 2, "pass": 1}
    assert sum(p.cells_per_bucket().values()) == 5 + 4 + 6 + 6 + 2
    assert p.near_off() == 0


def test_pct_never_rounds_to_the_ends() -> None:
    assert parity_report.pct(9999, 10000) == "99.99%"
    assert parity_report.pct(99999, 100000) == "99.99%"
    assert parity_report.pct(1, 100000) == "0.01%"
    assert parity_report.pct(5, 5) == "100.00%"


def test_main_writes_the_pages_and_charts(tmp_path: Path) -> None:
    rep, scope = tmp_path / "report.json", tmp_path / "scope.json"
    rep.write_text(json.dumps(report()))
    scope.write_text(json.dumps(SCOPE))
    out = tmp_path / "out"
    parity_report.main(
        [
            *("--report", str(rep), "--scope", str(scope), "--out-dir", str(out)),
            *("--commit", "abc12345", "--run-date", "2026-10-19"),
        ]
    )
    page = (out / "parity.md").read_text()
    assert "2026-10-19" in page
    assert "`abc12345`" in page
    assert "(parity/functions.md)" in page
    for chart in parity_report.CHARTS:
        name = chart.name
        assert f"(parity/{name})" in page
        svg = (out / "parity" / name).read_text()
        assert "@media (prefers-color-scheme: dark)" in svg
        assert all(" fill=" in shape for shape in svg.split("<rect")[1:])
    table = (out / "parity" / "functions.md").read_text()
    rows = [line for line in table.splitlines() if line.startswith("| ")]
    assert [r.split(" | ")[0] for r in rows[2:]] == ["| ABS", "| BETA.DIST", "| LN", "| SUM", "| UPPER"]
    assert all(r.count("|") == 12 for r in rows)


def test_nice_step_picks_1_2_5_steps() -> None:
    assert [nice_step(t) for t in (0, 3, 5, 6, 100, 149, 238, 258_865)] == [1, 1, 1, 2, 20, 50, 50, 100_000]


def test_stacked_inner_labels_need_room() -> None:
    # 60 of 100 is about 280px wide: labelled. 1 of 100 is under MIN_INNER_LABEL_W: not labelled.
    svg = bar_chart("t", "d", [Bar("cat", (("pass", 60), ("ulp4", 1), ("off", 39)), "100")])
    assert 'class="num in-pass"' in svg
    assert ">60</text>" in svg
    assert 'class="num in-off"' in svg
    assert 'class="num in-ulp4"' not in svg
    # A one-segment bar has no inner label, only its end label.
    single = bar_chart("t", "d", [Bar("Off", (("off", 7),), "7 (1%)")])
    assert "in-off" not in single.split("</style>")[1]
    assert ">7 (1%)</text>" in single


def test_inner_label_ink_follows_the_theme() -> None:
    svg = bar_chart("t", "d", [Bar("c", (("close", 50), ("pass", 50)), "100")])
    light, dark = svg.split("@media (prefers-color-scheme: dark)")
    assert ".in-close{fill:#ffffff}" in light
    assert ".in-close{fill:#000000}" in dark


def test_chart_contents() -> None:
    p = compute(report(), SCOPE)
    svgs = parity_report.charts(p)
    stacked = svgs["functions-per-category.svg"]
    for label in parity_report.LABELS.values():
        assert f">{label}</text>" in stacked
    assert ">math</text>" in stacked
    assert ">Within 2\N{EN DASH}4 ulp</text>" in svgs["functions-per-bucket.svg"]
    assert ">2 (40.00%)</text>" in svgs["functions-per-bucket.svg"]


def test_hostile_labels_are_escaped() -> None:
    hostile = '<script>"&|`\nx'
    svg = bar_chart(hostile, hostile, [Bar(hostile, (("pass", 1),), hostile)])
    assert "<script>" not in svg
    assert "&lt;script&gt;&quot;&amp;|" in svg
    md = table(["h"], [[hostile]])
    assert "<script>" not in md
    assert md.splitlines()[2] == '| &lt;script&gt;"&\\|&#96; x |'
    assert md_cell("a|b") == "a\\|b"


@pytest.mark.parametrize(("flag", "value"), [("--commit", "abc; rm -rf /"), ("--run-date", "19 Oct")])
def test_snapshot_args_are_validated(tmp_path: Path, flag: str, value: str) -> None:
    with pytest.raises(SystemExit):
        parity_report.main(["--out-dir", str(tmp_path), flag, value])
