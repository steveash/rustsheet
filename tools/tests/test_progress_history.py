"""The run history (SPEC-0022 "History and deltas", T-0071): docs/progress/history.json, the
`since-last-run` deltas, the trend charts with their gaps, and the stale page of an incomplete run
("Failure handling")."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import pytest
from test_progress_report import GIT, FakeGit, data, report, tasks, write_inputs

from rustsheet_tools import progress_report
from rustsheet_tools.progress_report import blocks, history, page
from rustsheet_tools.progress_report.history import Git, Row
from rustsheet_tools.progress_report.markdown import md_text
from rustsheet_tools.progress_report.model import RUN_DATE, ProgressReportError, Run, tasks_of
from rustsheet_tools.progress_report.svg import Series, line_chart, y_axis


def row(date: str, at_rate: int = 4, *, done: int = 15, commit: str = "abc12345") -> Row:
    """A complete row of the synthetic report, dated `date`, with some measures changed."""
    r = history.complete_row(Run(date, commit, ("s",), ()), data(), tasks_of(tasks()), None)
    r["phase1"]["at_rate"] = at_rate
    r["tasks"]["done"] = done
    return r


def stale_row(date: str) -> Row:
    return history.incomplete_row(date, "abc12345", ())


def run_main(
    tmp_path: Path, r: dict[str, Any] | None = None, *, git: Git = GIT, today: str = "2026-10-19"
) -> Path:
    out = tmp_path / "pages"
    progress_report.main([*write_inputs(tmp_path, r or report(), today=today), "--out-dir", str(out)], git)
    return out


def rows_of(out: Path) -> list[Row]:
    return history.load(out / "progress" / "history.json")


def snapshot(out: Path) -> dict[str, bytes]:
    return {str(p.relative_to(out)): p.read_bytes() for p in sorted(out.rglob("*")) if p.is_file()}


def test_the_complete_row_has_every_schema_key() -> None:
    r = row("2026-10-19")
    assert tuple(r) == (*history.BASE_KEYS, *history.METRIC_KEYS)
    assert r["phase1"] == {"at_rate": 4, "in_scope": 6}
    assert r["v15"] == {"met": False, "covered": 5, "visibly_off": 3}
    assert r["tiers"] == {
        "exact": 1,
        "ulp4": 1,
        "digits15": 1,
        "rate999": 1,
        "rate99": 0,
        "below99": 1,
        "unscored": 1,
    }
    assert (r["off_cells"], r["excel_cells"]) == ({"numeric": 1, "other": 1}, 2086)
    assert r["suites"] == {"s": {"cells": 100, "passed": 90, "visible": 3, "known_failures": None}}
    # Tasks summed over the milestones; open is todo, in progress and blocked.
    assert r["tasks"] == {"todo": 6, "in-progress": 2, "blocked": 4, "done": 15}
    assert r["campaign"] == {
        "semantics": {"open": 6, "blocked": 1, "done": 7},
        "accuracy": {"open": 3, "blocked": 0, "done": 1},
        "visible": {"open": 6, "blocked": 2, "done": 9},
        "parked": {"open": 9, "blocked": 3, "done": 0},
    }
    assert r["milestones_exited"] == ["M5"]
    assert history.parse(history.dump([r])) == [r]


def test_the_first_run_creates_the_history_with_its_row_only(tmp_path: Path) -> None:
    out = run_main(tmp_path)
    rows = rows_of(out)
    assert [(r["date"], r["commit"], r["complete"]) for r in rows] == [("2026-10-19", "abc12345", True)]
    text = (out / "progress.md").read_text()
    assert "No earlier complete run: this is the first complete row" in text
    for trend in history.TRENDS:
        assert f"(progress/{trend.name})" in text
        assert (out / "progress" / trend.name).exists()


def test_add_appends_replaces_the_same_date_and_never_rewrites_older_rows() -> None:
    first, second = row("2026-10-18", 3), row("2026-10-19", 4)
    rows = history.add(history.add([], first), second)
    assert rows == [first, second]
    rerun = row("2026-10-19", 5)
    assert history.add(rows, rerun) == [first, rerun]
    # An incomplete rerun keeps the day's complete row: the page still shows its numbers.
    assert history.add(rows, stale_row("2026-10-19")) == rows
    assert history.add([first, stale_row("2026-10-19")], second) == [first, second]
    with pytest.raises(ProgressReportError, match="before the history's last row"):
        history.add(rows, row("2026-10-17"))
    # The older row's line is unchanged byte for byte.
    old, new = (
        history.dump(rows).splitlines(),
        history.dump(history.add(rows, row("2026-10-20"))).splitlines(),
    )
    assert new[1] == old[1]
    assert new[2] == old[2] + ","


def test_a_rerun_on_the_same_date_replaces_its_row(tmp_path: Path) -> None:
    run_main(tmp_path)
    r = report()
    r["v15_checkpoint"]["at_rate"] = 5
    out = run_main(tmp_path, r)
    rows = rows_of(out)
    assert len(rows) == 1
    assert rows[0]["phase1"]["at_rate"] == 5


@pytest.mark.parametrize(
    ("text", "error"),
    [
        ("[", "not valid JSON"),
        ("{}", "not a JSON array"),
        ("[1]", "row 0 is not a JSON object"),
        ('[{"date": "2026-10-19", "commit": null, "complete": false}]', "row 0 has no `skipped_suites`"),
        ('[{"date": "Oct 19", "commit": null, "complete": false, "skipped_suites": []}]', "not a YYYY-MM-DD"),
        (
            '[{"date": "2026-10-19", "commit": "XYZ", "complete": false, "skipped_suites": []}]',
            "commit is not a commit sha",
        ),
        (
            '[{"date": "2026-10-19", "commit": null, "complete": "yes", "skipped_suites": []}]',
            "complete is not true or false",
        ),
        (
            '[{"date": "2026-10-19", "commit": null, "complete": true, "skipped_suites": []}]',
            "has no `phase1`",
        ),
    ],
)
def test_an_invalid_history_fails_and_nothing_is_written(tmp_path: Path, text: str, error: str) -> None:
    with pytest.raises(ProgressReportError, match=error):
        history.parse(text)
    out = tmp_path / "pages"
    (out / "progress").mkdir(parents=True)
    (out / "progress" / "history.json").write_text(text)
    with pytest.raises(SystemExit, match=error):
        run_main(tmp_path)
    assert snapshot(out) == {"progress/history.json": text.encode()}


@pytest.mark.parametrize(
    ("mutate", "error"),
    [
        (lambda r: r["tiers"].pop("rate99"), "has no `rate99`"),
        (lambda r: r["campaign"]["parked"].pop("open"), "campaign.parked has no `open`"),
        (lambda r: r["tasks"].update(done=-1), "tasks.done is not a count"),
    ],
)
def test_a_complete_row_missing_a_measure_fails(mutate: Any, error: str) -> None:
    r = row("2026-10-19")
    mutate(r)
    with pytest.raises(ProgressReportError, match=error):
        history.parse(history.dump([r]))


def test_rows_must_be_one_per_date_oldest_first() -> None:
    for dates in (("2026-10-19", "2026-10-19"), ("2026-10-19", "2026-10-18")):
        with pytest.raises(ProgressReportError, match="one per date, oldest first"):
            history.parse(history.dump([row(d) for d in dates]))


def test_since_last_run_compares_against_the_last_complete_row() -> None:
    rows = [row("2026-10-17", 2, done=10, commit="0000aaaa"), stale_row("2026-10-18")]
    now = row("2026-10-19", 4, done=13)
    last = history.last_complete(history.add(rows, now), "2026-10-19")
    assert last is rows[0]
    text = history.since_block(now, last, 7)
    assert "Against the last complete run, **2026-10-17** at commit `0000aaaa`" in text
    assert (
        "**7** commits landed on `main` since then (`git rev-list --count --first-parent 0000aaaa..abc12345`)"
        in text
    )
    assert (
        "| Phase 1: functions at \N{GREATER-THAN OR EQUAL TO} 99.9% within 15 digits | 2 | 4 | +2 |" in text
    )
    assert "| Tasks done | 10 | 13 | +3 |" in text
    assert "| Functions in tier: Exact | 1 | 1 | 0 |" in text
    assert "| Visibly-off attributed cells: number | 1 | 1 | 0 |" in text
    assert "| Attributed Excel cells | 2,086 | 2,086 | 0 |" in text
    assert "Git could not count" in history.since_block(now, last, None)
    assert "| Tasks done | 13 | 10 | \N{MINUS SIGN}3 |" in history.since_block(rows[0], now, 1)
    assert "**1** commit landed" in history.since_block(rows[0], now, 1)
    # A rerun on the same date compares against the day before, not against itself.
    assert history.last_complete([rows[0], now], "2026-10-19") is rows[0]
    assert history.last_complete([stale_row("2026-10-18")], "2026-10-19") is None


def test_the_next_day_shows_deltas_and_commits_from_git(tmp_path: Path) -> None:
    run_main(tmp_path)
    r = report()
    r["run"] = {"commit": "def45678", "date": "2026-10-21", "skipped_suites": []}
    r["v15_checkpoint"]["at_rate"] = 5
    out = run_main(tmp_path, r, git=FakeGit("fedc9876", {("abc12345", "def45678"): 12}))
    assert [x["date"] for x in rows_of(out)] == ["2026-10-19", "2026-10-21"]
    text = (out / "progress.md").read_text()
    assert "Against the last complete run, **2026-10-19** at commit `abc12345`" in text
    assert "**12** commits landed on `main`" in text
    assert "| 4 | 5 | +1 |" in text
    # 2026-10-20 has no row: the trend line breaks there (two lone points, no polyline).
    svg = (out / "progress" / "trend-functions.svg").read_text()
    assert "<polyline" not in svg
    assert svg.count("<circle") == 2


def test_a_report_at_another_commit_is_complete_and_the_page_does_not_depend_on_head(tmp_path: Path) -> None:
    out = run_main(tmp_path, git=FakeGit("fedc9876"))
    text = (out / "progress.md").read_text()
    assert "**Stale.**" not in text
    assert "The eval run of **2026-10-19** at commit `abc12345`" in text
    assert "fedc9876" not in text
    assert rows_of(out)[0]["complete"] is True
    assert rows_of(out)[0]["commit"] == "abc12345"
    first = snapshot(out)
    # The same report regenerated from another HEAD (or with git unavailable) is byte-identical.
    run_main(tmp_path, git=FakeGit("0123abcd"))
    assert snapshot(out) == first
    run_main(tmp_path, git=FakeGit(None))
    assert snapshot(out) == first


def test_a_report_without_a_commit_falls_back_to_head_in_its_row(tmp_path: Path) -> None:
    r = report()
    r["run"]["commit"] = None
    out = run_main(tmp_path, r, git=FakeGit("fedc9876"))
    assert rows_of(out)[0]["commit"] == "fedc9876"


def test_the_stale_banner_names_the_generating_head(tmp_path: Path) -> None:
    run_main(tmp_path)
    r = report()
    _skipped(r)
    r["run"]["date"] = "2026-10-20"
    out = run_main(tmp_path, r, git=FakeGit("fedc9876"))
    status = page.parse((out / "progress.md").read_text())["status"].text
    assert "at commit `abc12345`, generated at commit `fedc9876`, is incomplete" in status
    same = page.stale_status("r", "2026-10-20", "abc12345", None, None, head="abc12345")
    assert "generated at" not in same


def _no_run(r: dict[str, Any]) -> None:
    del r["run"]


def _no_tiers(r: dict[str, Any]) -> None:
    del r["tiers"]


def _skipped(r: dict[str, Any]) -> None:
    r["run"]["skipped_suites"] = [{"name": "poi", "reason": "not fetched"}]


@pytest.mark.parametrize(
    ("make", "reason", "date"),
    [
        ("missing", "report.json is missing: the eval failed or produced no report", "2026-10-20"),
        ("garbage", "report.json is not valid JSON", "2026-10-20"),
        (_no_run, "an older report shape (report.json has no `run` record", "2026-10-20"),
        (_no_tiers, "an older report shape (report.json has no `tiers`", "2026-10-21"),
        (_skipped, "suites not run: poi (not fetched)", "2026-10-21"),
    ],
)
def test_an_incomplete_run_marks_only_the_status_block_stale(
    tmp_path: Path, make: Any, reason: str, date: str
) -> None:
    out = run_main(tmp_path)
    page_text = (out / "progress.md").read_text()
    written = page_text.replace(blocks.PLACEHOLDER, "\nSteward text.\n", 1)
    (out / "progress.md").write_text(written)
    before = snapshot(out)
    r = report()
    r["run"]["date"] = "2026-10-21"
    if callable(make):
        make(r)
    out_dir = ["--out-dir", str(out)]
    args = write_inputs(tmp_path, r, today="2026-10-20")
    rep = tmp_path / "report.json"
    if make == "missing":
        rep.unlink()
    elif make == "garbage":
        rep.write_text('{"suites": [')
    progress_report.main([*args, *out_dir], GIT)  # exits 0: no SystemExit
    after = snapshot(out)
    # Only the page and the history changed; functions.md and every chart are byte-identical.
    assert {k for k in after if after[k] != before.get(k)} == {"progress.md", "progress/history.json"}
    old, new = page.parse(written), page.parse(after["progress.md"].decode())
    assert {n for n in old if old[n].text != new[n].text} == {"status"}
    assert after["progress.md"].decode().replace(new["status"].text, old["status"].text) == written
    status = new["status"].text
    assert status.startswith(f"\n> **Stale.** The run of **{date}** at commit `abc12345` is incomplete: ")
    assert md_text(reason) in status
    assert "the last complete run, **2026-10-19** at commit `abc12345`" in status
    rows = rows_of(out)
    assert [(x["date"], x["complete"]) for x in rows] == [("2026-10-19", True), (date, False)]
    assert set(rows[1]) == set(history.BASE_KEYS)
    if make is _skipped:
        assert rows[1]["skipped_suites"] == [{"name": "poi", "reason": "not fetched"}]
        assert "- **Suites skipped (1):** poi (not fetched)." in status


def test_a_stale_run_with_no_page_yet_writes_placeholders(tmp_path: Path) -> None:
    r = report()
    del r["run"]
    out = run_main(tmp_path, r, today="2026-10-20")
    text = (out / "progress.md").read_text()
    parsed = page.parse(text)
    assert list(parsed) == [name for _, name, _ in page.LAYOUT]
    assert "No complete run is recorded" in parsed["status"].text
    assert parsed["parity"].text == f"\n{page.NO_COMPLETE_RUN}\n"
    assert parsed["risks"].text == blocks.PLACEHOLDER
    assert [x["complete"] for x in rows_of(out)] == [False]
    assert snapshot(out).keys() == {"progress.md", "progress/history.json"}


def test_a_stale_rerun_after_a_complete_run_on_the_same_day_names_that_run(tmp_path: Path) -> None:
    run_main(tmp_path)
    r = report()
    _skipped(r)
    out = run_main(tmp_path, r)
    assert [x["complete"] for x in rows_of(out)] == [True]
    assert (
        "the last complete run, **2026-10-19**"
        in page.parse((out / "progress.md").read_text())["status"].text
    )


def test_a_catalog_mismatch_is_not_a_stale_run(tmp_path: Path) -> None:
    r = report()
    r["tiers"][0]["functions"] = 2
    with pytest.raises(SystemExit, match=r"report\.json `tiers` counts"):
        run_main(tmp_path, r)
    assert not (tmp_path / "pages").exists()


def test_the_trend_charts_follow_the_theme_and_leave_gaps() -> None:
    rows = [row("2026-10-17", 3), row("2026-10-18", 4), stale_row("2026-10-19"), row("2026-10-20", 5)]
    rows.append(row("2026-10-22", 6))
    svgs = history.charts(rows)
    assert set(svgs) == {t.name for t in history.TRENDS}
    for svg in svgs.values():
        assert svg.count("@media (prefers-color-scheme: dark)") == 1
        assert all(" fill=" in shape for shape in svg.split("<rect")[1:])
        assert all(' stroke="#' in line and 'fill="none"' in line for line in svg.split("<polyline")[1:])
        assert all(" fill=" in shape for shape in svg.split("<circle")[1:])
        # Six date slots, 17 to 22; first and last labelled.
        assert ">2026-10-17</text>" in svg
        assert ">2026-10-22</text>" in svg
    tasks_svg = svgs["trend-open-tasks.svg"]
    assert ".s-semantics{stroke:#2a78d6}" in tasks_svg.split("@media")[0]
    assert ".s-semantics{stroke:#3987e5}" in tasks_svg.split("@media")[1]
    assert ">Parked</text>" in tasks_svg
    functions = svgs["trend-functions.svg"]
    # One line for 17-18; the 19th (stale) and the 21st (no row) break it; 20 and 22 stand alone.
    assert functions.count("<polyline") == 1
    assert functions.count("<circle") == 2  # 20 and 22; 18 ends the line, so no dot


def test_line_chart_runs_and_axis() -> None:
    svg = line_chart(
        "t", "d", ["a", "b", "c", "d"], [Series("at_rate", "x", (1, None, 2, 3))], history.FUNCTIONS_PALETTE
    )
    assert svg.count("<polyline") == 1  # c-d
    assert svg.count("<circle") == 2  # a alone, and d (the last point)
    assert y_axis([324]) == (315, 325, 5)
    assert y_axis([0, 5]) == (0, 5, 1)
    assert y_axis([47, 13]) == (0, 50, 10)
    hostile = line_chart("<x>", "<y>", ["<d>"], [Series("at_rate", "<s>", (1,))], history.FUNCTIONS_PALETTE)
    assert "<x>" not in hostile
    assert "<s>" not in hostile


def test_a_stale_run_older_than_the_last_row_is_not_recorded_and_exits_0(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    out = run_main(tmp_path)
    history_before = (out / "progress" / "history.json").read_bytes()
    r = report()
    r["run"]["date"] = "2026-10-18"
    _skipped(r)
    run_main(tmp_path, r)  # exits 0: no SystemExit
    assert (out / "progress" / "history.json").read_bytes() == history_before
    status = page.parse((out / "progress.md").read_text())["status"].text
    assert status.startswith("\n> **Stale.** The run of **2026-10-18**")
    assert "not recorded in history: the run's date 2026-10-18 is before the last row, 2026-10-19" in (
        capsys.readouterr().out
    )


def test_dates_and_shas_take_ascii_digits_only() -> None:
    arabic_indic = "\N{ARABIC-INDIC DIGIT TWO}" * 4 + "-10-19"
    assert RUN_DATE.fullmatch("2026-10-19")
    assert not RUN_DATE.fullmatch(arabic_indic)
    with pytest.raises(ProgressReportError, match="not a YYYY-MM-DD"):
        history.parse(history.dump([{**stale_row("2026-10-19"), "date": arabic_indic}]))
