"""The run history, docs/progress/history.json, and the `since-last-run` and `trends` blocks
(SPEC-0022, "History and deltas" and "Failure handling"; T-0071).

The history is a JSON array, oldest first, with at most one row per date. A complete run adds its
row with every measure; an incomplete one adds a `complete: false` row with no measures. A rerun
on the same date replaces that date's row, except that an incomplete rerun never replaces a
complete row: the page keeps that run's numbers, so its row must stay to name them. Older rows are
never rewritten. Deltas always compare against the last complete row before this run's date, and
the trend charts leave a gap for every date with no complete row, so a missed day shows.
"""

from __future__ import annotations

import datetime as dt
import json
import re
import subprocess
from collections.abc import Callable
from dataclasses import dataclass
from itertools import pairwise
from pathlib import Path
from typing import Any, Protocol

from rustsheet_tools.progress_report import codebase
from rustsheet_tools.progress_report.markdown import signed, table
from rustsheet_tools.progress_report.model import (
    COMMIT_SHA,
    GE,
    PARKED,
    RUN_DATE,
    STAGES,
    TASK_STATUSES,
    TIERS,
    VISIBLE,
    Parity,
    ProgressReportError,
    Run,
    SkippedSuite,
    Tasks,
    count,
    field,
    flag,
    mapping,
)
from rustsheet_tools.progress_report.parity import TIER_LABELS
from rustsheet_tools.progress_report.svg import AQUA, BLUE, RED, SLATE, VIOLET, Palette, Series, line_chart

HISTORY = "history.json"
# The page's link to the history, from docs/progress.md.
HISTORY_LINK = "[docs/progress/history.json](progress/history.json)"
# Every row has these keys; a complete row also has METRIC_KEYS (the SPEC-0022 row schema).
BASE_KEYS = ("date", "commit", "complete", "skipped_suites")
METRIC_KEYS = (
    "phase1",
    "v15",
    "tiers",
    "off_cells",
    "excel_cells",
    "suites",
    "tasks",
    "campaign",
    "milestones_exited",
)
STAGE_ROWS = (*STAGES, PARKED)
STAGE_KEYS = ("open", "blocked", "done")

Row = dict[str, Any]


class Git(Protocol):
    """The two git facts the page reads; None when git cannot say."""

    def head(self) -> str | None:
        """`git rev-parse --short=8 HEAD`."""

    def commits_between(self, since: str, until: str) -> int | None:
        """`git rev-list --count --first-parent <since>..<until>`."""


@dataclass(frozen=True)
class RepoGit:
    """`Git` for the repository at `root`."""

    root: Path

    def _run(self, *args: str) -> str | None:
        try:
            out = subprocess.run(
                ["git", *args], cwd=self.root, check=True, capture_output=True, text=True, timeout=60
            )
        except (OSError, subprocess.SubprocessError):
            return None
        return out.stdout.strip()

    def head(self) -> str | None:
        """`git rev-parse --short=8 HEAD`, the short form report.json `run.commit` uses."""
        sha = self._run("rev-parse", "--short=8", "HEAD")
        return sha if sha and COMMIT_SHA.fullmatch(sha) else None

    def commits_between(self, since: str, until: str) -> int | None:
        """The first-parent commits from `since` to `until`; None for a commit not in this clone."""
        n = self._run("rev-list", "--count", "--first-parent", f"{since}..{until}")
        return int(n) if n and re.fullmatch(r"[0-9]+", n) else None


def _check_row(row: Any, i: int) -> str:
    """Check row `i` against the schema; return its date."""
    where = f"history.json row {i}"
    for key in BASE_KEYS:
        field(row, key, where)
    date = row["date"]
    if not isinstance(date, str) or not RUN_DATE.fullmatch(date):
        msg = f"{where}.date is not a YYYY-MM-DD date: {date!r}"
        raise ProgressReportError(msg)
    where = f"history.json row {date}"
    commit = row["commit"]
    if commit is not None and (not isinstance(commit, str) or not COMMIT_SHA.fullmatch(commit)):
        msg = f"{where}.commit is not a commit sha or null: {commit!r}"
        raise ProgressReportError(msg)
    skipped = row["skipped_suites"]
    if not isinstance(skipped, list) or not all(
        isinstance(s, dict) and isinstance(s.get("name"), str) and isinstance(s.get("reason"), str)
        for s in skipped
    ):
        msg = f"{where}.skipped_suites is not a list of {{name, reason}}"
        raise ProgressReportError(msg)
    if not flag(row, "complete", where):
        return date
    for key in METRIC_KEYS:
        field(row, key, where)
    # The measures the deltas and charts read.
    count(mapping(row, "phase1", where), "at_rate", f"{where}.phase1")
    count(row["phase1"], "in_scope", f"{where}.phase1")
    for tier in TIERS:
        count(mapping(row, "tiers", where), tier, f"{where}.tiers")
    for kind in ("numeric", "other"):
        count(mapping(row, "off_cells", where), kind, f"{where}.off_cells")
    count(row, "excel_cells", where)
    for status in TASK_STATUSES:
        count(mapping(row, "tasks", where), status, f"{where}.tasks")
    for stage in STAGE_ROWS:
        per = mapping(mapping(row, "campaign", where), stage, f"{where}.campaign")
        for key in STAGE_KEYS:
            count(per, key, f"{where}.campaign.{stage}")
    if "codebase" in row:
        codebase.check_row(row["codebase"], f"{where}.codebase")
    return date


def parse(text: str) -> list[Row]:
    """The rows of a history file's text, checked: a JSON array of rows, each with every schema
    key, oldest first with at most one row per date."""
    try:
        rows = json.loads(text)
    except ValueError as e:
        msg = f"history.json is not valid JSON: {e}"
        raise ProgressReportError(msg) from e
    if not isinstance(rows, list):
        msg = "history.json is not a JSON array"
        raise ProgressReportError(msg)
    dates = [_check_row(row, i) for i, row in enumerate(rows)]
    for before, after in pairwise(dates):
        if after <= before:
            msg = f"history.json rows are not one per date, oldest first: {after} follows {before}"
            raise ProgressReportError(msg)
    return rows


def load(path: Path) -> list[Row]:
    """The rows of the history file at `path`; none when the file does not exist yet."""
    try:
        text = path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return []
    except (OSError, UnicodeDecodeError) as e:
        msg = f"{path}: {e}"
        raise ProgressReportError(msg) from e
    return parse(text)


def dump(rows: list[Row]) -> str:
    """The history file's text: one row per line, so a run's diff is one added or changed line."""
    lines = ",\n".join(json.dumps(row, ensure_ascii=False) for row in rows)
    return f"[\n{lines}\n]\n" if rows else "[]\n"


def add(rows: list[Row], row: Row) -> list[Row]:
    """`rows` with `row` added: appended for a new date, replacing the row of its own date (unless
    that one is complete and `row` is not). A date before the last row's is an error: older rows
    are never rewritten."""
    if not rows or rows[-1]["date"] < row["date"]:
        return [*rows, row]
    last = rows[-1]
    if last["date"] > row["date"]:
        msg = f"this run's date {row['date']} is before the history's last row, {last['date']}"
        raise ProgressReportError(msg)
    if last["complete"] and not row["complete"]:
        return rows
    return [*rows[:-1], row]


def last_complete(rows: list[Row], before: str) -> Row | None:
    """The last complete row dated before `before`."""
    return next((r for r in reversed(rows) if r["complete"] and r["date"] < before), None)


def latest_complete(rows: list[Row]) -> Row | None:
    """The last complete row."""
    return next((r for r in reversed(rows) if r["complete"]), None)


def _skipped(skipped: tuple[SkippedSuite, ...]) -> list[dict[str, str]]:
    return [{"name": s.name, "reason": s.reason} for s in skipped]


def complete_row(
    run: Run, data: Parity, tasks: Tasks, head: str | None, counts: dict[str, int] | None = None
) -> Row:
    """The history row of a complete run. Its commit is the report's `run.commit`; only when the
    eval could not read it (null) does the row fall back to `head`, the commit generating the page,
    so the next run can still count the commits since this one. `counts` adds the optional
    `codebase` object (T-0078); rows from before it have none."""
    cells = data.attributed_cells()
    numeric, other = VISIBLE
    row: Row = {
        "date": run.date,
        "commit": run.commit or head,
        "complete": True,
        "skipped_suites": _skipped(run.skipped),
        "phase1": {"at_rate": data.phase1.at_rate, "in_scope": data.phase1.in_scope},
        "v15": {
            "met": data.phase1.v15_met,
            "covered": data.phase1.covered,
            "visibly_off": data.phase1.visibly_off,
        },
        "tiers": {t.tier: t.functions for t in data.tiers},
        "off_cells": {"numeric": cells[numeric], "other": cells[other]},
        "excel_cells": sum(cells.values()),
        "suites": {
            s.name: {
                "cells": s.cells,
                "passed": s.passed,
                "visible": s.visible,
                "known_failures": s.known_failures,
            }
            for s in data.suites
        },
        "tasks": dict(tasks.by_status),
        "campaign": {stage: dict(tasks.by_stage[stage]) for stage in STAGE_ROWS},
        "milestones_exited": list(data.milestones_exited),
    }
    if counts is not None:
        row["codebase"] = dict(counts)
    return row


def incomplete_row(date: str, commit: str | None, skipped: tuple[SkippedSuite, ...]) -> Row:
    """The history row of an incomplete run: no measures."""
    return {"date": date, "commit": commit, "complete": False, "skipped_suites": _skipped(skipped)}


def _tier_measure(tier: str) -> Callable[[Row], int]:
    return lambda r: int(r["tiers"][tier])


# The deltas: (label, the measure of a complete row).
MEASURES: tuple[tuple[str, Callable[[Row], int]], ...] = (
    (f"Phase 1: functions at {GE} 99.9% within 15 digits", lambda r: int(r["phase1"]["at_rate"])),
    *((f"Functions in tier: {TIER_LABELS[t]}", _tier_measure(t)) for t in TIERS),
    ("Visibly-off attributed cells: number", lambda r: int(r["off_cells"]["numeric"])),
    ("Visibly-off attributed cells: type, error or text", lambda r: int(r["off_cells"]["other"])),
    ("Attributed Excel cells", lambda r: int(r["excel_cells"])),
    ("Tasks done", lambda r: int(r["tasks"]["done"])),
)


def since_block(row: Row, last: Row | None, commits: int | None) -> str:
    """The `since-last-run` block: this run's measures against the last complete row `last`, and
    `commits`, the first-parent commits from its commit to this row's (None when git could not
    say). Both commits are the runs' own, so the block does not depend on the generating HEAD."""
    head = "## Since the last run"
    if last is None:
        return (
            f"{head}\n\nNo earlier complete run: this is the first complete row of {HISTORY_LINK}, "
            "so there is nothing to compare yet."
        )
    then = f"commit `{last['commit']}`" if last["commit"] else "an unknown commit"
    if commits is None:
        landed = "Git could not count the commits since then."
    else:
        landed = (
            f"**{commits:,}** commit{'s' if commits != 1 else ''} landed on `main` since then "
            f"(`git rev-list --count --first-parent {last['commit']}..{row['commit']}`)."
        )
    rows = [
        [label, f"{measure(last):,}", f"{measure(row):,}", signed(measure(row) - measure(last))]
        for label, measure in MEASURES
    ]
    return f"""{head}

Against the last complete run, **{last["date"]}** at {then} ({HISTORY_LINK}). {landed}

{table(["Measure", last["date"], row["date"], "Change"], rows)}"""


def _slots(rows: list[Row]) -> list[str]:
    """Every date from the history's first row to its last, one per day."""
    first, last = dt.date.fromisoformat(rows[0]["date"]), dt.date.fromisoformat(rows[-1]["date"])
    return [(first + dt.timedelta(days=d)).isoformat() for d in range((last - first).days + 1)]


def _series(rows: list[Row], slots: list[str], key: str, label: str, measure: Callable[[Row], int]) -> Series:
    """One value per date slot: the measure of that date's complete row, else a gap."""
    by_date = {r["date"]: r for r in rows if r["complete"]}
    values = tuple(measure(by_date[d]) if d in by_date else None for d in slots)
    return Series(key, label, values)


FUNCTIONS_PALETTE = Palette({"at_rate": BLUE})
OFF_PALETTE = Palette({"numeric": RED, "other": SLATE})
# The first three hues of the validated tier palette (svg.TIER_PALETTE), and the neutral slate
# for the parked tasks.
STAGE_PALETTE = Palette(dict(zip(STAGE_ROWS, (BLUE, AQUA, VIOLET, SLATE), strict=True)))
STAGE_LABELS = {"semantics": "Semantics", "accuracy": "Accuracy", "visible": "Visible", PARKED: "Parked"}


def _functions_chart(rows: list[Row], slots: list[str], title: str) -> str:
    series = _series(rows, slots, "at_rate", "Functions at rate", lambda r: int(r["phase1"]["at_rate"]))
    desc = (
        f"In-scope functions with {GE} 99.9% of their attributed Excel cells within 15 digits, per "
        "complete run; a date without one is a gap."
    )
    return line_chart(title, desc, slots, [series], FUNCTIONS_PALETTE)


def _off_chart(rows: list[Row], slots: list[str], title: str) -> str:
    series = [
        _series(rows, slots, "numeric", "Number", lambda r: int(r["off_cells"]["numeric"])),
        _series(rows, slots, "other", "Type, error or text", lambda r: int(r["off_cells"]["other"])),
    ]
    desc = "Visibly-off attributed Excel cells by kind, per complete run; a date without one is a gap."
    return line_chart(title, desc, slots, series, OFF_PALETTE)


def _open_measure(stage: str) -> Callable[[Row], int]:
    return lambda r: int(r["campaign"][stage]["open"])


def _stages_chart(rows: list[Row], slots: list[str], title: str) -> str:
    series = [_series(rows, slots, s, STAGE_LABELS[s], _open_measure(s)) for s in STAGE_ROWS]
    desc = (
        "Open tasks (todo, in progress or blocked) per decision-0114 campaign stage and parked "
        "(defer: phase-1b), per complete run; a date without one is a gap."
    )
    return line_chart(title, desc, slots, series, STAGE_PALETTE)


@dataclass(frozen=True)
class Trend:
    """One trend chart: its file name under progress/, its title and alt text, and its builder."""

    name: str
    title: str
    build: Callable[[list[Row], list[str], str], str]


TRENDS = (
    Trend("trend-functions.svg", f"Functions at {GE} 99.9% within 15 digits", _functions_chart),
    Trend("trend-visibly-off.svg", "Visibly-off attributed cells", _off_chart),
    Trend("trend-open-tasks.svg", "Open tasks per campaign stage", _stages_chart),
)


def trends_block(assets: str) -> str:
    """The `trends` block: the trend charts under `assets`/."""
    charts = "\n\n".join(f"![{t.title}]({assets}/{t.name})" for t in TRENDS)
    return f"""## Trends

One point per complete run in {HISTORY_LINK}.
A date with no complete run (a missed or stale day) leaves a gap in the lines; nothing is
interpolated.

{charts}"""


def charts(rows: list[Row]) -> dict[str, str]:
    """The trend charts by file name, from `rows` (which hold at least one complete row)."""
    slots = _slots(rows)
    return {t.name: t.build(rows, slots, t.title) for t in TRENDS}


METHOD = """- **History.** Each run adds one row to docs/progress/history.json, one per date: a
  rerun on the same date replaces it, except that an incomplete rerun never replaces a complete
  row of the same date, and older rows are never rewritten. A run is complete when
  report.json parses, has the `run`, `v15_checkpoint` and tier fields, and skipped no suite. An
  incomplete run adds a `complete: false` row without measures and rewrites only the status
  block, marked stale; the rest of the page keeps the last complete run's numbers. "Since the
  last run" compares against the last complete row; its commits are `git rev-list --count
  --first-parent <that row's commit>..<this run's commit>`. Its visibly-off and Excel cells are the parity
  section's attributed cells, and open tasks are todo, in progress or blocked (`scripts/tasks.py
  stats --json`)."""
