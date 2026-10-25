"""The progress report generator (SPEC-0022) on a small synthetic report: the parity block, the
all-functions page and the charts (T-0081), input validation, and the run record (T-0070).
test_progress_page.py covers the page and its judgment blocks."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import pytest

from rustsheet_tools import progress_report
from rustsheet_tools.progress_report import parity
from rustsheet_tools.progress_report.markdown import md_cell, md_text, pct, table
from rustsheet_tools.progress_report.model import (
    TIERS,
    Parity,
    ProgressReportError,
    compute,
    known_failures,
    run_of,
)
from rustsheet_tools.progress_report.svg import TIER_PALETTE, Bar, bar_chart, nice_step

SCOPE: dict[str, Any] = {
    "functions": [
        {"name": "ABS", "category": "math", "milestone": "M2", "scope": "in"},
        {"name": "SUM", "category": "math", "milestone": "M2", "scope": "in"},
        {"name": "LN", "category": "math", "milestone": "M5", "scope": "in"},
        {"name": "BETA.DIST", "category": "statistical", "milestone": "M9", "scope": "in"},
        {"name": "UPPER", "category": "text", "milestone": "M5", "scope": "in"},
        {"name": "INFO", "category": "information", "milestone": "M5"},
        {"name": "CALL", "category": "external", "milestone": "M14", "scope": "out"},
    ]
}

ZERO = {"pass": 0, "ulp1": 0, "ulp4": 0, "digits15": 0, "visible_numeric": 0, "visible_other": 0}
NO_BASELINES = Path("/nonexistent/eval/baseline")


def fn(name: str, tier: str, attribution: str = "sole", **attributed: int) -> dict[str, Any]:
    """A report `functions[]` entry (T-0068): its tier and attributed cells; its Excel cells are
    the attributed ones plus 10 bit-exact cells it shares with other functions."""
    cells = {**ZERO, **attributed}
    return {
        "name": name,
        "tier": tier,
        "attribution": attribution,
        "attributed": cells,
        "excel": {**cells, "pass": cells["pass"] + 10},
    }


def rate(name: str, cells: int, visible: int, meets: bool) -> dict[str, Any]:
    """A `v15_checkpoint.functions[]` entry (T-0080)."""
    return {"name": name, "rate_cells": cells, "rate_visible": visible, "meets_rate": meets}


def report() -> dict[str, Any]:
    """ABS exact, SUM within 4 ulp, LN within 15 digits, UPPER at 99.9%, BETA.DIST below 99% placed
    by every Excel cell (fallback). INFO is called by no cell (unscored). CALL is out of scope."""
    tiers = {"exact": 1, "ulp4": 1, "digits15": 1, "rate999": 1, "rate99": 0, "below99": 1, "unscored": 1}
    fallback = {"below99": 1, "unscored": 1}
    return {
        "functions": [
            fn("ABS", "exact", **{"pass": 30}),
            fn("SUM", "ulp4", **{"pass": 20, "ulp1": 2, "ulp4": 1}),
            fn("LN", "digits15", **{"pass": 25, "digits15": 3}),
            fn("UPPER", "rate999", **{"pass": 1999, "visible_other": 1}),
            fn("BETA.DIST", "below99", "all", **{"pass": 4, "visible_numeric": 1}),
            fn("CALL", "below99", "all", visible_other=1),
        ],
        "tiers": [{"tier": t, "functions": tiers[t], "fallback": fallback.get(t, 0)} for t in TIERS],
        "v15_checkpoint": {
            "in_scope": 6,
            "at_rate": 4,
            "phase1_met": False,
            "at_strict_rate": 3,
            "functions": [
                rate("ABS", 30, 0, True),
                rate("SUM", 23, 0, True),
                rate("LN", 28, 0, True),
                rate("UPPER", 2000, 1, True),
                rate("BETA.DIST", 15, 1, False),
                rate("INFO", 0, 0, False),
            ],
        },
        "cases": [],
        "run": {"commit": "abc12345", "date": "2026-10-19", "skipped_suites": []},
        "suites": [
            {
                "name": "s",
                "cases": 4,
                "cells": 100,
                "cells_passed": 90,
                "deviations": 2,
                "statuses": {"pass": 2, "fail": 1, "timeout": 1},
                "verdicts": {"pass": 88, "deviation": 2, "numeric_ulp": 4, "numeric_close": 1},
                "closeness": {**ZERO, "pass": 90, "visible_numeric": 1, "visible_other": 2},
                "workbooks": {"scored": 3, "all_exact": 1, "all_within15": 2, "any_visible": 1},
            }
        ],
    }


def data(r: dict[str, Any] | None = None, baselines: Path = NO_BASELINES) -> Parity:
    return compute(r or report(), SCOPE, baselines)


def test_functions_come_from_the_catalog_with_the_reports_tiers() -> None:
    p = data()
    f = {x.name: x for x in p.functions}
    assert sorted(f) == ["ABS", "BETA.DIST", "INFO", "LN", "SUM", "UPPER"]
    assert (f["BETA.DIST"].tier, f["BETA.DIST"].attribution) == ("below99", "all")
    assert (f["INFO"].tier, f["INFO"].attribution, f["INFO"].excel_cells) == ("unscored", "all", 0)
    assert (f["UPPER"].attributed_cells, f["UPPER"].excel_cells, f["UPPER"].rate_within15) == (
        2000,
        2010,
        1999,
    )


def test_tier_table() -> None:
    text = parity.tier_table(data())
    assert "| Exact | every cell bit-exact | 1 | 16.67% | 0 |" in text
    assert "| &lt; 99% | &lt; 99% of cells within 15 digits | 1 | 16.67% | 1 |" in text
    assert "| Unscored | no scored Excel cell | 1 | 16.67% | 1 |" in text
    assert "| **All** |  | **6** |  | 2 |" in text
    summary = parity.tier_summary(data())
    assert summary.startswith("**3 of 6** in-scope functions (50.00%) never differ")
    assert "**1** have fewer than 99%" in summary


def test_category_table() -> None:
    text = parity.category_table(data())
    lines = text.splitlines()
    assert lines[0] == "| Category | Functions | Exact | \N{LESS-THAN OR EQUAL TO} 4 ulp | 15 digits | " + (
        "\N{GREATER-THAN OR EQUAL TO} 99.9% | \N{GREATER-THAN OR EQUAL TO} 99% | &lt; 99% | Unscored |"
    )
    # The largest category first, then by name.
    assert lines[2] == "| math | 3 | 1 | 1 | 1 | 0 | 0 | 0 | 0 |"
    assert [line.split(" | ")[0] for line in lines[3:]] == ["| information", "| statistical", "| text"]


def test_attributed_cells_by_closeness() -> None:
    p = data()
    cells = p.attributed_cells()
    assert sum(cells.values()) == 30 + 23 + 28 + 2000 + 5
    assert (cells["visible_numeric"], cells["visible_other"]) == (1, 1)
    assert "| Visibly off: number | 1 | 0.05% |" in parity.closeness_table(p)
    assert parity.closeness_summary(p).startswith("**2,086** attributed Excel cells")


def test_phase1_lines_come_from_v15_checkpoint() -> None:
    text = parity.phase1_lines(data())
    assert text.startswith("**Phase 1 (decision 0114): not met.** **4 of 6** in-scope functions")
    assert "_Information, gates nothing:_ 3 of 6 functions" in text


def test_suites_table_with_known_failures(tmp_path: Path) -> None:
    (tmp_path / "s.txt").write_text(
        "# header\nC\tcase\t5\nF\tcase\tA1\nF\tcase\tA2\nTC\tcase\t1\nTF\tcase\tB1\n"
    )
    assert known_failures(tmp_path / "s.txt") == (2, 1)
    assert known_failures(tmp_path / "none.txt") == (None, 0)
    row = parity.suites_table(data(baselines=tmp_path)).splitlines()[2]
    # Bit-exact 90/98, within 4 ulp 94/98, 3 visibly off, workbooks 3/1/2/1, 1 timeout.
    assert row == "| s | 4 | 100 | 91.84% | 95.92% | 3 | 3 | 1 | 2 | 1 | 1 | 2; text 1 |"
    assert parity.suites_table(data()).splitlines()[2].endswith("| no baseline |")


@pytest.mark.parametrize(
    ("mutate", "error"),
    [
        (lambda r: r["functions"][0].pop("tier"), "function 'ABS' has no `tier`; the report predates T-0068"),
        (
            lambda r: r["functions"][1].pop("attributed"),
            "'SUM' has no `attributed`; the report predates T-0068",
        ),
        (lambda r: r["functions"][2]["excel"].pop("digits15"), "excel has no `digits15`; .*T-0068"),
        (lambda r: r.pop("tiers"), "report.json has no `tiers`; the report predates T-0068"),
        (
            lambda r: r["suites"][0].pop("workbooks"),
            "suite 's' has no `workbooks`; the report predates T-0068",
        ),
        (lambda r: r.pop("v15_checkpoint"), "has no `v15_checkpoint`; the report predates T-0080"),
        (lambda r: r["v15_checkpoint"].pop("at_rate"), "has no `at_rate`; the report predates T-0080"),
        (lambda r: r["v15_checkpoint"]["functions"].pop(), "no entry for in-scope function INFO"),
        (lambda r: r["functions"][0].update(tier="great"), "tier is 'great', not one of"),
        (lambda r: r["tiers"][0].update(functions=2), "report.json `tiers` counts"),
        (lambda r: r["tiers"].pop(), "not every tier once in order"),
        (lambda r: r["suites"][0].update(cells="many"), "cells is not a count"),
        (lambda r: r["suites"][0].update(name="../x"), "not a suite name"),
        (lambda r: r.update(suites={}), "suites is not a list"),
    ],
)
def test_a_report_without_the_fields_this_reads_fails_loudly(mutate: Any, error: str) -> None:
    r = report()
    mutate(r)
    with pytest.raises(ProgressReportError, match=error):
        data(r)


def test_pct_never_rounds_to_the_ends() -> None:
    assert pct(9999, 10000) == "99.99%"
    assert pct(99999, 100000) == "99.99%"
    assert pct(1, 100000) == "0.01%"
    assert pct(5, 5) == "100.00%"
    assert pct(0, 0) == "\N{EN DASH}"


def write_inputs(tmp_path: Path, r: dict[str, Any]) -> list[str]:
    rep, scope = tmp_path / "report.json", tmp_path / "scope.json"
    rep.write_text(json.dumps(r))
    scope.write_text(json.dumps(SCOPE))
    return ["--report", str(rep), "--scope", str(scope), "--baselines", str(NO_BASELINES)]


def test_main_writes_the_page_the_functions_page_and_the_charts(tmp_path: Path) -> None:
    out = tmp_path / "out"
    progress_report.main([*write_inputs(tmp_path, report()), "--out-dir", str(out)])
    page = (out / "progress.md").read_text()
    assert "## Parity" in page
    assert "(progress/functions.md)" in page
    for chart in parity.CHARTS:
        assert f"(progress/{chart.name})" in page
        svg = (out / "progress" / chart.name).read_text()
        assert "@media (prefers-color-scheme: dark)" in svg
        assert all(" fill=" in shape for shape in svg.split("<rect")[1:])
    functions = (out / "progress" / "functions.md").read_text()
    assert "2026-10-19" in functions
    assert "`abc12345`" in functions
    rows = [line for line in functions.splitlines() if line.startswith("| ")]
    names = [r.split(" | ")[0] for r in rows[2:]]
    assert names == ["| ABS", "| BETA.DIST", "| INFO", "| LN", "| SUM", "| UPPER"]
    assert all(r.count("|") == 10 for r in rows)
    assert "| BETA.DIST | statistical | M9 | &lt; 99% | every call | 5 | 15 | 93.33% | no |" in functions
    assert "| INFO | information | M5 | Unscored | every call | 0 | 0 | \N{EN DASH} | no |" in functions
    # The retired parity page is never written (T-0081: docs/parity.md is a static pointer).
    assert not (out / "parity.md").exists()
    assert not (out / "parity").exists()


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


def test_the_tier_charts_use_the_tier_palette() -> None:
    p = data()
    svgs = {c.name: c.build(p, c.title) for c in parity.CHARTS}
    stacked = svgs["tiers-by-category.svg"]
    for label in parity.TIER_LABELS.values():
        assert f">{label.replace('<', '&lt;')}</text>" in stacked
    assert ">math</text>" in stacked
    assert f'class="b-below99" fill="{TIER_PALETTE.fills["below99"][0]}"' in stacked
    assert ">1 (16.67%)</text>" in svgs["tiers.svg"]
    assert ">Within 2\N{EN DASH}4 ulp</text>" in svgs["cells-by-closeness.svg"]
    assert set(TIER_PALETTE.fills) == set(TIERS) == set(TIER_PALETTE.ink)


def test_hostile_labels_are_escaped() -> None:
    hostile = '<script>"&|`\nx'
    svg = bar_chart(hostile, hostile, [Bar(hostile, (("pass", 1),), hostile)])
    assert "<script>" not in svg
    assert "&lt;script&gt;&quot;&amp;|" in svg
    md = table(["h"], [[hostile]])
    assert "<script>" not in md
    assert md.splitlines()[2] == '| &lt;script&gt;"&\\|&#96; x |'
    assert md_cell("a|b") == "a\\|b"
    # md_text writes `&` as an entity before md_cell adds its own, and escapes a backslash.
    assert md_text("&lt; a\\*b <c>") == "&amp;lt; a\\\\\\*b &lt;c&gt;"


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
    with pytest.raises(SystemExit, match=field):
        progress_report.main([*write_inputs(tmp_path, r), "--out-dir", str(tmp_path / "out")])
    assert not (tmp_path / "out").exists()
