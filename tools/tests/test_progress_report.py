"""The progress report generator (SPEC-0022) on a small synthetic report: the parity page's bucket
logic and output (T-2041) and the run record (T-0070). test_progress_page.py covers the page."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import pytest

from rustsheet_tools import progress_report
from rustsheet_tools.progress_report import parity
from rustsheet_tools.progress_report.markdown import md_cell, pct, table
from rustsheet_tools.progress_report.model import FunctionParity, ProgressReportError, compute, run_of
from rustsheet_tools.progress_report.svg import Bar, bar_chart, nice_step

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


ZERO = {"pass": 0, "ulp1": 0, "ulp4": 0, "digits15": 0, "visible_numeric": 0, "visible_other": 0}


def fn(
    name: str, cells: int, passed: int, unsupported: int = 0, blamed: int = 0, **buckets: int
) -> dict[str, Any]:
    """A report `functions[]` entry with its closeness counts (T-2036, T-0061); `passed` cells are
    bit-exact and `blamed` visibly-off cells are held against the function."""
    closeness = {**ZERO, "pass": passed, **buckets}
    verdicts = {"pass": passed, **({"unsupported": unsupported} if unsupported else {})}
    visible = {"numeric": blamed, "other": 0}
    return {
        "name": name,
        "cells": cells,
        "passed": passed,
        "verdicts": verdicts,
        "closeness": closeness,
        "visible": visible,
    }


def report() -> dict[str, Any]:
    """ABS: 1 ulp. LN: one 1-ulp and one 3-ulp cell. SUM: a 1-ulp and a visibly-off cell that is
    blamed on BETA.DIST, which has a 15-digit cell, a visibly-off text cell and a deviation (pinned
    value, counted as pass).
    UPPER: bit-exact but one unsupported. CALL is out of scope."""
    return {
        "functions": [
            fn("ABS", 5, 4, ulp1=1),
            fn("SUM", 4, 2, ulp1=1, visible_numeric=1),
            fn("LN", 6, 4, ulp1=1, ulp4=1),
            fn("BETA.DIST", 6, 3, blamed=2, digits15=1, visible_other=1, **{"pass": 4}),
            fn("UPPER", 3, 2, unsupported=1),
            fn("CALL", 2, 0, unsupported=1, visible_other=1),
        ],
        "cases": [],
        "run": {"commit": "abc12345", "date": "2026-10-19", "skipped_suites": []},
        "suites": [
            {
                "name": "s",
                "cases": 2,
                "cells": 30,
                "cells_passed": 22,
                "verdicts": {"pass": 22, "numeric_ulp": 3, "numeric_close": 1, "unsupported": 1},
                "closeness": {**ZERO, "visible_numeric": 1, "visible_other": 2},
            }
        ],
    }


def by_name(r: dict[str, Any] | None = None) -> dict[str, FunctionParity]:
    return {f.name: f for f in compute(r or report(), SCOPE).functions}


def test_ulp_buckets_are_kept_apart() -> None:
    ln = by_name()["LN"]
    assert (ln.counts["ulp1"], ln.counts["ulp4"]) == (1, 1)
    assert ln.worst == "ulp4"


def test_function_is_placed_by_its_worst_cell() -> None:
    f = by_name()
    assert f["ABS"].worst == "ulp1"
    assert f["BETA.DIST"].worst == "visible_other"
    assert f["SUM"].worst == "visible_numeric"
    assert f["UPPER"].worst == "pass"


def test_deviation_counts_as_pass() -> None:
    beta = by_name()["BETA.DIST"]
    assert beta.counts == {**ZERO, "pass": 4, "digits15": 1, "visible_other": 1}


def test_unsupported_cells_are_dropped() -> None:
    upper = by_name()["UPPER"]
    assert upper.cells == 2
    assert upper.counts["pass"] == 2


def test_buckets_come_from_the_report() -> None:
    f = by_name()
    assert f["SUM"].counts == {**ZERO, "pass": 2, "ulp1": 1, "visible_numeric": 1}
    assert f["ABS"].counts["ulp1"] == 1


def test_a_report_without_closeness_counts_fails_loudly() -> None:
    r = report()
    del r["functions"][0]["closeness"]
    with pytest.raises(ProgressReportError, match="T-2036"):
        compute(r, SCOPE)


def test_only_in_scope_catalog_functions_count() -> None:
    p = compute(report(), SCOPE)
    assert "CALL" not in {f.name for f in p.functions}
    assert p.unscored == ["INFO"]
    assert p.in_scope == 6


def test_buckets_that_do_not_sum_to_the_scored_cells_fail_loudly() -> None:
    r = report()
    r["functions"][0]["cells"] = 7
    with pytest.raises(ProgressReportError, match="ABS"):
        compute(r, SCOPE)


def test_totals_and_near_off() -> None:
    p = compute(report(), SCOPE)
    assert p.functions_per_bucket() == {
        "ulp1": 1,
        "ulp4": 1,
        "visible_numeric": 1,
        "visible_other": 1,
        "pass": 1,
    }
    assert sum(p.cells_per_bucket().values()) == 5 + 4 + 6 + 6 + 2
    assert p.near_off() == 0


def test_visibly_off_functions_are_the_blamed_ones() -> None:
    p = compute(report(), SCOPE)
    assert [(f.name, f.visible) for f in p.blamed()] == [("BETA.DIST", 2)]
    assert p.suite_visible_cells() == 3
    assert "**1 in-scope functions are visibly off**" in parity.takeaway(p)
    assert "| BETA.DIST | statistical | M9 | 2 | 6 |" in parity.visible_table(p)


def test_a_report_without_the_fifteen_digit_bucket_fails_loudly() -> None:
    r = report()
    del r["functions"][0]["closeness"]["digits15"]
    with pytest.raises(ProgressReportError, match="T-0061"):
        compute(r, SCOPE)


def test_pct_never_rounds_to_the_ends() -> None:
    assert pct(9999, 10000) == "99.99%"
    assert pct(99999, 100000) == "99.99%"
    assert pct(1, 100000) == "0.01%"
    assert pct(5, 5) == "100.00%"


def test_main_writes_the_pages_and_charts(tmp_path: Path) -> None:
    rep, scope = tmp_path / "report.json", tmp_path / "scope.json"
    rep.write_text(json.dumps(report()))
    scope.write_text(json.dumps(SCOPE))
    out = tmp_path / "out"
    progress_report.main(["--report", str(rep), "--scope", str(scope), "--out-dir", str(out)])
    page = (out / "parity.md").read_text()
    assert "2026-10-19" in page
    assert "`abc12345`" in page
    assert "(parity/functions.md)" in page
    for chart in parity.CHARTS:
        name = chart.name
        assert f"(parity/{name})" in page
        svg = (out / "parity" / name).read_text()
        assert "@media (prefers-color-scheme: dark)" in svg
        assert all(" fill=" in shape for shape in svg.split("<rect")[1:])
    table = (out / "parity" / "functions.md").read_text()
    rows = [line for line in table.splitlines() if line.startswith("| ")]
    assert [r.split(" | ")[0] for r in rows[2:]] == ["| ABS", "| BETA.DIST", "| LN", "| SUM", "| UPPER"]
    assert all(r.count("|") == 15 for r in rows)
    assert "## Phase 1 (decision 0114)" in page


def test_phase1_bands_read_the_within_fifteen_digit_share() -> None:
    # Decision 0114: BETA.DIST has 2 of 6 cells blamed visibly off; the others none.
    p = compute(report(), SCOPE)
    f = {x.name: x for x in p.functions}
    assert f["BETA.DIST"].within15_share == pytest.approx(4 / 6)
    assert f["BETA.DIST"].phase1_band == "Below 90%"
    assert f["SUM"].phase1_band.endswith("(Phase 1 exit)")
    per = p.functions_per_phase1_band()
    assert (per["Below 90%"], sum(per.values())) == (1, 5)
    assert "| Below 90% | 1 | 20.00% |" in parity.phase1_table(p)


def test_nice_step_picks_1_2_5_steps() -> None:
    assert [nice_step(t) for t in (0, 3, 5, 6, 100, 149, 238, 258_865)] == [1, 1, 1, 2, 20, 50, 50, 100_000]


def test_stacked_inner_labels_need_room() -> None:
    # 60 of 100 is about 280px wide: labelled. 1 of 100 is under MIN_INNER_LABEL_W: not labelled.
    svg = bar_chart("t", "d", [Bar("cat", (("pass", 60), ("ulp4", 1), ("visible_numeric", 39)), "100")])
    assert 'class="num in-pass"' in svg
    assert ">60</text>" in svg
    assert 'class="num in-visible_numeric"' in svg
    assert 'class="num in-ulp4"' not in svg
    # A one-segment bar has no inner label, only its end label.
    single = bar_chart("t", "d", [Bar("Off", (("visible_numeric", 7),), "7 (1%)")])
    assert "in-visible_numeric" not in single.split("</style>")[1]
    assert ">7 (1%)</text>" in single


def test_inner_label_ink_follows_the_theme() -> None:
    svg = bar_chart("t", "d", [Bar("c", (("digits15", 50), ("pass", 50)), "100")])
    light, dark = svg.split("@media (prefers-color-scheme: dark)")
    assert ".in-digits15{fill:#ffffff}" in light
    assert ".in-digits15{fill:#000000}" in dark


def test_chart_contents() -> None:
    p = compute(report(), SCOPE)
    svgs = parity.charts(p)
    stacked = svgs["functions-per-category.svg"]
    for label in parity.LABELS.values():
        assert f">{label}</text>" in stacked
    assert ">math</text>" in stacked
    assert ">Within 2\N{EN DASH}4 ulp</text>" in svgs["functions-per-bucket.svg"]
    assert ">1 (20.00%)</text>" in svgs["functions-per-bucket.svg"]


def test_hostile_labels_are_escaped() -> None:
    hostile = '<script>"&|`\nx'
    svg = bar_chart(hostile, hostile, [Bar(hostile, (("pass", 1),), hostile)])
    assert "<script>" not in svg
    assert "&lt;script&gt;&quot;&amp;|" in svg
    md = table(["h"], [[hostile]])
    assert "<script>" not in md
    assert md.splitlines()[2] == '| &lt;script&gt;"&\\|&#96; x |'
    assert md_cell("a|b") == "a\\|b"


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("commit", "abc; rm -rf /"),
        ("commit", "ABC12345"),
        ("date", "19 Oct"),
        ("date", None),
        ("skipped_suites", [{"name": "poi"}]),
        ("skipped_suites", None),
    ],
)
def test_the_run_record_is_validated(tmp_path: Path, field: str, value: object) -> None:
    # The run's commit, date and skipped suites are printed into the pages.
    r = report()
    r["run"][field] = value
    with pytest.raises(ProgressReportError, match=f"run.{field}"):
        run_of(r)
    rep, scope = tmp_path / "report.json", tmp_path / "scope.json"
    rep.write_text(json.dumps(r))
    scope.write_text(json.dumps(SCOPE))
    with pytest.raises(SystemExit, match=field):
        progress_report.main(
            ["--report", str(rep), "--scope", str(scope), "--out-dir", str(tmp_path / "out")]
        )
    assert not (tmp_path / "out").exists()
