"""The `goals` and `campaign` blocks (SPEC-0022 "Page layout", T-0072): the docs/milestones.md
parse, the milestone rows, the Phase 1 and checkpoint lines, the campaign stage and parked rows,
and the functions the semantic fixes alone would lift."""

from __future__ import annotations

import dataclasses
from pathlib import Path
from typing import Any

import pytest
from test_progress_report import GIT, MILESTONES, ZERO, data, exit_row, report, tasks, write_inputs

from rustsheet_tools import progress_report
from rustsheet_tools.progress_report import goals, history
from rustsheet_tools.progress_report.model import FunctionParity, Parity, ProgressReportError, Run, tasks_of

EN = "\N{EN DASH}"
GE = "\N{GREATER-THAN OR EQUAL TO}"


def docs() -> list[goals.MilestoneDoc]:
    return goals.parse_milestones(MILESTONES)


def test_milestones_md_gives_each_milestone_its_phase_and_checked_exit_criteria() -> None:
    d = {m.milestone: m for m in docs()}
    assert list(d) == ["M0", "M1", "M2", "M3", "M5", "M6", "M9"]
    # Only the boxes under **Exit criteria:**, up to the next bold label, count.
    assert (d["M0"].title, d["M0"].phase, d["M0"].checked, d["M0"].boxes) == ("Scaffolding", "1", 2, 3)
    assert (d["M1"].checked, d["M1"].boxes) == (0, 0)
    assert (d["M3"].phase, d["M3"].checked, d["M3"].boxes) == ("2", 0, 1)


@pytest.mark.parametrize(
    ("edit", "error"),
    [
        (
            lambda t: t.replace("**Exit criteria:** ironcalc-calc at 99.95%.", "Done when it is done."),
            r"## M9 has no `\*\*Exit criteria:\*\*` list",
        ),
        (
            lambda t: t + "\n## M7 \N{EM DASH} Arrays\n\n**Exit criteria:** x.\n",
            "## M7 is in no row of the phase",
        ),
        (lambda t: t.replace("M5, M6, M9", "M5, M6, M9, M8"), "names M8, which have no ## heading"),
        (lambda t: t.replace("| M3 |", "| M3, M0 |"), "lists M0 twice"),
        (lambda t: t + "\n## M9 \N{EM DASH} Again\n\n**Exit criteria:** x.\n", "## M9 appears twice"),
        (lambda t: t.replace("## Phase order", "## Order"), "no `## Phase order` section"),
    ],
)
def test_the_milestones_md_parse_fails_loudly(edit: Any, error: str) -> None:
    with pytest.raises(ProgressReportError, match=error):
        goals.parse_milestones(edit(MILESTONES))


def test_the_phase1_and_checkpoint_lines_are_the_reports() -> None:
    g = goals.load(report())
    text = goals.goals_block(g, docs(), data(), tasks_of(tasks()), None)
    assert "**Phase 1 \\(decision 0114\\): not met \\(4/6 functions covered\\)**" in text
    assert "**Fifteen-digit parity: not met \\(coverage 5/6, visibly-off cells 3\\)**" in text
    assert "The history has no earlier complete run" in text
    assert "Milestones whose exit row in the report is met: M5." in text


def test_whether_the_previous_complete_run_also_met_the_checkpoint() -> None:
    last = history.complete_row(Run("2026-10-18", "0123abcd", ("s",), ()), data(), tasks_of(tasks()), None)
    assert goals.previous_checkpoint(last, False) == (
        "At the previous complete run, **2026-10-18** at commit `0123abcd`, the checkpoint was not met."
    )
    last["v15"]["met"] = True
    assert goals.previous_checkpoint(last, True).endswith("was met: it is met on two complete runs in a row.")
    assert goals.previous_checkpoint(last, False).endswith("was met.")


def test_one_row_per_milestone_with_measures_probes_and_tasks() -> None:
    text = goals.milestone_table(goals.load(report()), docs(), tasks_of(tasks()))
    rows = {line.split(" | ")[0][2:]: line for line in text.splitlines()[2:]}
    assert list(rows) == [
        "M0 Scaffolding",
        "M1 Core semantics",
        "M2 Top functions",
        "M3 Dependency graph (Phase 2)",
        "M5 Function families I",
        "M6 Lookup & reference",
        "M9 Numerical parity",
    ]
    assert rows["M0 Scaffolding"] == (
        f"| M0 Scaffolding | 1 | 2/3 | {EN} | {EN} | probes M0-M2: 1 visibly off, 3 known failures | 3 | 1 | 2 | 10 |"
    )
    assert (
        "| frozen list m1-probes: 2 known failures in 1 cases (9 of 9 run); probes M0-M2:"
        in rows["M1 Core semantics"]
    )
    assert rows["M2 Top functions"].startswith("| M2 Top functions | 1 | prose | 2/2 |")
    assert rows["M2 Top functions"].endswith("| 3 | 1 | 2 | 5 |")
    assert rows["M5 Function families I"].startswith(
        "| M5 Function families I | 1 | prose | \N{EN DASH} | **exit met**; coverage 3/3 (met); generated suite "
        "4 visibly off of 20,000, allowance 20 (met); ironcalc-calc 0 visibly off (met); probes 0 visibly off "
        "(met) | probes M5: 0 visibly off, 0 known failures | 0 | 0 | 0 | 0 |"
    )
    # A measure whose input the run lacked is null in the report: never met.
    assert "generated suite 4 visibly off of ?, allowance 20 (not in run)" in rows["M6 Lookup & reference"]
    assert "| no functions | **exit not met**" in rows["M6 Lookup & reference"]


@pytest.mark.parametrize(
    ("mutate", "error"),
    [
        (
            lambda _r, t: t["by_milestone"].update(M4=t["by_milestone"]["M0"]),
            "tasks.json by_milestone counts milestone M4",
        ),
        (
            lambda r, _t: r["milestone_exit"].append(r["milestone_exit"][0] | {"milestone": "M7"}),
            "milestone_exit counts milestone M7",
        ),
        (
            lambda r, _t: r["catalog_coverage"][0].update(milestone="M8"),
            "catalog_coverage counts milestone M8",
        ),
    ],
)
def test_a_milestone_without_a_heading_fails_loudly(mutate: Any, error: str) -> None:
    r, t = report(), tasks()
    mutate(r, t)
    with pytest.raises(ProgressReportError, match=error):
        goals.milestone_table(goals.load(r), docs(), tasks_of(t))


@pytest.mark.parametrize(
    ("mutate", "error"),
    [
        (lambda r: r["v15_checkpoint"].pop("rate_num"), "has no `rate_num`; the report predates T-0072"),
        (
            lambda r: r["v15_checkpoint"].pop("phase1_line"),
            "has no `phase1_line`; the report predates T-0072",
        ),
        (lambda r: r["v15_checkpoint"].update(rate_den=0), "rate 999/0 is not a share"),
        (lambda r: r["probe_milestones"][0].update(milestone="M2-M0"), "not M<n> or M<a>-M<b>"),
        (lambda r: r["milestone_lists"][0].update(name="probes"), "not a frozen list"),
    ],
)
def test_a_report_without_the_goals_fields_fails(mutate: Any, error: str) -> None:
    r = report()
    mutate(r)
    with pytest.raises(ProgressReportError, match=error):
        goals.load(r)


def function(
    name: str, tier: str, cells: dict[str, int], *, sole: bool = True, excel_other: int = 0
) -> FunctionParity:
    """A function whose counted cells are `cells` (attributed when `sole`, else every Excel cell,
    which holds `excel_other` more type, error or text cells than its attributed ones)."""
    attributed = {**ZERO, **cells}
    excel = {**attributed, "visible_other": attributed["visible_other"] + excel_other}
    counted = attributed if sole else excel
    visible = counted["visible_numeric"] + counted["visible_other"]
    return FunctionParity(
        name, "statistical", "M9", tier, "sole" if sole else "all", attributed, excel,
        sum(counted.values()), visible, False, sum(excel.values()), True, None,
    )  # fmt: skip


def campaign_data() -> Parity:
    return dataclasses.replace(
        data(),
        functions=[
            # 997 of 1,000 within 15 digits; fixing its 2 text cells reaches exactly 99.9%.
            function("AT.BAR", "rate99", {"pass": 997, "visible_other": 2, "visible_numeric": 1}),
            # 998 of 1,000 after its 1 text cell: still below.
            function("SHORT", "rate99", {"pass": 997, "visible_other": 1, "visible_numeric": 2}),
            # Placed by every Excel cell (fallback): its 5 text cells there lift it, though its
            # attributed cells hold none.
            function("FALLBACK", "below99", {"pass": 10, "visible_numeric": 0}, sole=False, excel_other=5),
            # At the rate already, or unscored: not below the bar, so in no count.
            function("FINE", "rate999", {"pass": 9999, "visible_other": 1}),
            function("NONE", "unscored", {}),
            # Its tier counts a cell the rate leaves out (decision 0134), so its rate cells hold no
            # off cell and it meets the rate: not below the bar either.
            dataclasses.replace(
                function("LEFT.OUT", "rate99", {"pass": 465, "visible_other": 1}),
                rate_cells=465,
                rate_visible=0,
                meets_rate=True,
            ),
        ],
    )


def test_the_semantic_fixes_lift_count_and_the_off_cells() -> None:
    g = goals.load(report())
    lifted = {f.name: goals.lifts(g, f) for f in campaign_data().functions}
    assert lifted == {
        "AT.BAR": True,
        "SHORT": False,
        "FALLBACK": True,
        "FINE": True,
        "NONE": False,
        "LEFT.OUT": True,
    }
    text = goals.campaign_block(g, campaign_data(), tasks_of(tasks()))
    assert "**3** in-scope functions are below the Phase 1 rate" in text
    # Attributed cells only: FALLBACK's 5 are Excel cells, not attributed ones.
    assert "**3** visibly-off type, error or text cells and **3** visibly-off numbers" in text
    assert "would lift **2** of them" in text
    assert "- **Lifted by the semantic fixes alone:** AT.BAR, FALLBACK." in text


def test_the_rate_comes_from_the_report() -> None:
    r = report()
    r["v15_checkpoint"]["rate_num"] = 990
    g = goals.load(r)
    assert g.rate == "99%"
    text = goals.campaign_block(g, campaign_data(), tasks_of(tasks()))
    assert "would lift **3** of them to \N{GREATER-THAN OR EQUAL TO} 99%" in text


def test_the_stage_and_parked_rows() -> None:
    text = goals.campaign_block(goals.load(report()), campaign_data(), tasks_of(tasks()))
    rows = {line.split(" | ")[0][2:]: line for line in text.splitlines() if line.startswith("| ")}
    # Open is todo, in progress and blocked; ready, blocked and done are tasks.json's.
    assert rows["Semantics"].endswith("| 6 | 2 | 1 | 7 | 3 type, error or text | 2 of 3 |")
    assert rows["Accuracy"].endswith(f"| 3 | 2 | 0 | 1 | 3 numbers, accuracy and visible together | {EN} |")
    assert rows["Visible"].endswith(f"| 6 | 2 | 2 | 9 | in the accuracy row | {EN} |")
    assert rows["Parked (defer: phase-1b)"].endswith(f"| 9 | 2 | 3 | 0 | {EN} | {EN} |")
    assert "Visibly-off cells (functions under 99.9%)" in text


def test_main_writes_both_blocks(tmp_path: Path) -> None:
    out = tmp_path / "out"
    progress_report.main([*write_inputs(tmp_path, report()), "--out-dir", str(out)], GIT)
    text = (out / "progress.md").read_text()
    assert text.index("## Bottom line") < text.index("## Goals") < text.index("## Campaign stages")
    assert text.index("## Campaign stages") < text.index("## Since the last run")
    assert "- **Goals.** The Phase 1 and checkpoint lines" in text


def test_a_report_before_t0072_is_a_stale_run(tmp_path: Path) -> None:
    r = report()
    del r["v15_checkpoint"]["verdict_line"]
    out = tmp_path / "out"
    progress_report.main([*write_inputs(tmp_path, r), "--out-dir", str(out)], GIT)
    text = (out / "progress.md").read_text()
    assert "**Stale.**" in text
    assert "predates T-0072" in text
    assert "## Goals" not in text


def test_a_coverage_count_without_a_criterion_reads_not_required() -> None:
    # Decision 0135: M2's row shows its coverage counts, but coverage is not an M2 criterion.
    row = exit_row("M2", True) | {"covered": 30, "functions": 37, "coverage_met": None}
    assert "coverage 30/37 (not required)" in goals.exit_measures("M2", row)
    absent = exit_row("M6", False) | {"covered": None, "functions": None, "coverage_met": None}
    assert "coverage ?/? (not in run)" in goals.exit_measures("M6", absent)


def test_an_m7_row_shows_calc_not_required_and_its_spill_extents() -> None:
    # Decision 0140: ironcalc-calc counts are shown without a verdict; spill extents are M7's.
    row = exit_row("M7", True) | {
        "calc_visibly_off": 3,
        "calc_met": None,
        "extent_failures": 0,
        "extent_met": True,
    }
    text = goals.exit_measures("M7", row)
    assert "ironcalc-calc 3 visibly off (not required)" in text
    assert text.endswith("spill extents 0 failures (met)")
    assert "spill extents" not in goals.exit_measures("M5", exit_row("M5", True))


def test_m1_and_m11_rows_name_their_suites() -> None:
    # Decision 0146: M1 adds smoke beside gen-m1; M11's one suite is corpus-bench.
    m1 = exit_row("M1", True) | {
        "gen_suite": "gen-m1",
        "more_suites": [{"name": "smoke", "scored": 46, "visibly_off": 0, "allowance": 0, "met": True}],
    }
    text = goals.exit_measures("M1", m1)
    assert "generated suite 4 visibly off of 20,000" in text
    assert "smoke 0 visibly off of 46, allowance 0 (met)" in text
    m11 = exit_row("M11", True) | {"gen_suite": "corpus-bench"}
    assert "corpus-bench 4 visibly off of 20,000, allowance 20 (met)" in goals.exit_measures("M11", m11)


def test_an_m9_row_shows_no_suite_its_calc_allowance_and_the_functions_below_the_rate() -> None:
    # Decision 0146 item 3: no required suite, ironcalc-calc at 0.9995, every M9 function at the rate.
    m9 = exit_row("M9", False) | {
        "gen_suite": "",
        "gen_scored": None,
        "calc_visibly_off": 138,
        "calc_scored": 73_150,
        "calc_allowance": 36,
        "calc_met": False,
        "rate_functions": 263,
        "below_rate": 82,
        "rate_met": False,
    }
    text = goals.exit_measures("M9", m9)
    assert "no suite required" in text
    assert "generated suite" not in text
    assert "ironcalc-calc 138 visibly off of 73,150, allowance 36 (not met)" in text
    assert text.endswith("functions at the Phase 1 rate 82 below of 263 (not met)")
    assert "Phase 1 rate" not in goals.exit_measures("M5", exit_row("M5", True))


def test_an_m8_row_shows_both_gen_suites_and_the_poi_subset_with_its_blocked_cases() -> None:
    # Decision 0146 item 2: gen-m8b and the poi subset are `more_suites` beside gen-m8a.
    m8 = exit_row("M8", False) | {
        "gen_suite": "gen-m8a",
        "more_suites": [
            {"name": "gen-m8b", "scored": 3_368, "visibly_off": 0, "allowance": 0, "met": True},
            {
                "name": "poi M8 subset",
                "scored": 2_000,
                "visibly_off": 3,
                "allowance": 10,
                "met": False,
                "blocked_cases": ["poi/a.xlsx!*", "poi/b.xls!*"],
            },
        ],
    }
    text = goals.exit_measures("M8", m8)
    assert "gen-m8a 4 visibly off of 20,000" in text
    assert "gen-m8b 0 visibly off of 3,368, allowance 0 (met)" in text
    assert "poi M8 subset 3 visibly off of 2,000, allowance 10, 2 cases blocked (not met)" in text


def test_an_m10_row_shows_the_text_items_of_gen_m10_and_its_probe_row() -> None:
    # Decision 0146 item 4: gen-m10's text items and the M10 probe row's, none failing.
    m10 = exit_row("M10", False) | {
        "gen_suite": "gen-m10",
        "gen_text_items": 10_400,
        "gen_text_failing": 76,
        "probe_text_items": 415,
        "probe_text_failing": 0,
    }
    text = goals.exit_measures("M10", m10)
    assert "generated suite text 76 failing of 10,400" in text
    assert ", text 0 failing of 415 (" in text
    plain = goals.exit_measures("M5", exit_row("M5", True))
    assert "text" not in plain
