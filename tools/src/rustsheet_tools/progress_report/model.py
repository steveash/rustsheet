"""Input loading and validation for the progress report (SPEC-0022, T-0070, T-0081).

Reads the eval report, the catalog and the eval baselines and checks they are the shape this
generator reads; every failure is a `ProgressReportError` naming the input and, for an old report
shape, the task whose field is missing. Verdicts, buckets, tiers, attribution and the Phase 1 rate
all come from the report (crates/rustsheet-eval); this module only counts and checks them.
"""

from __future__ import annotations

import json
import re
from collections import Counter, defaultdict
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Any

EN = "\N{EN DASH}"
LE = "\N{LESS-THAN OR EQUAL TO}"
GE = "\N{GREATER-THAN OR EQUAL TO}"
# The closeness buckets of a cell, best first (crates/rustsheet-eval/src/closeness.rs `Closeness`).
BUCKETS = ("pass", "ulp1", "ulp4", "digits15", "visible_numeric", "visible_other")
VISIBLE = ("visible_numeric", "visible_other")
# The parity tiers, best first: the report's `Tier` names (crates/rustsheet-eval/src/
# attribution.rs, `Tier::ALL`). The report places every function; this list only orders them.
TIERS = ("exact", "ulp4", "digits15", "rate999", "rate99", "below99", "unscored")
# Which cells placed a function's tier (`Attribution`): its attributed cells, or every Excel cell.
ATTRIBUTIONS = ("sole", "all")
# The workbook counts of a suite (`Workbooks` in attribution.rs).
WORKBOOKS = ("scored", "all_exact", "all_within15", "any_visible")


class ProgressReportError(Exception):
    """An input is missing, malformed or not the shape this generator reads."""


class ReportShapeError(ProgressReportError):
    """The eval report lacks a field a later task added: an older report shape. The run is
    incomplete, not broken (SPEC-0022 "Failure handling"): the page is marked stale."""


def field(obj: Any, key: str, where: str, task: str | None = None) -> Any:
    """`obj[key]`; a non-object `obj` or a missing key is a `ProgressReportError` naming `where`
    and, when given, the task that added the key (an older report shape)."""
    if not isinstance(obj, dict):
        msg = f"{where} is not a JSON object"
        raise ProgressReportError(msg)
    if key not in obj:
        msg = f"{where} has no `{key}`"
        if task:
            msg += f"; the report predates {task}, re-run `make eval`"
            raise ReportShapeError(msg)
        raise ProgressReportError(msg)
    return obj[key]


def count(obj: Any, key: str, where: str, task: str | None = None) -> int:
    """`field(...)` checked to be a non-negative integer (JSON booleans are not counts)."""
    value = field(obj, key, where, task)
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        msg = f"{where}.{key} is not a count: {value!r}"
        raise ProgressReportError(msg)
    return value


def flag(obj: Any, key: str, where: str, task: str | None = None) -> bool:
    """`field(...)` checked to be a JSON boolean."""
    value = field(obj, key, where, task)
    if not isinstance(value, bool):
        msg = f"{where}.{key} is not true or false: {value!r}"
        raise ProgressReportError(msg)
    return value


def items(obj: Any, key: str, where: str, task: str | None = None) -> list[Any]:
    """`field(...)` checked to be a JSON array."""
    value = field(obj, key, where, task)
    if not isinstance(value, list):
        msg = f"{where}.{key} is not a list"
        raise ProgressReportError(msg)
    return value


def mapping(obj: Any, key: str, where: str, task: str | None = None) -> dict[str, Any]:
    """`field(...)` checked to be a JSON object."""
    value = field(obj, key, where, task)
    if not isinstance(value, dict):
        msg = f"{where}.{key} is not a JSON object"
        raise ProgressReportError(msg)
    return value


def one_of(obj: Any, key: str, where: str, allowed: tuple[str, ...], task: str | None = None) -> str:
    """`field(...)` checked to be one of `allowed` (a name the report writes)."""
    value = field(obj, key, where, task)
    if value not in allowed:
        msg = f"{where}.{key} is {value!r}, not one of {', '.join(allowed)}"
        raise ProgressReportError(msg)
    return str(value)


def closeness(obj: Any, key: str, where: str, task: str) -> dict[str, int]:
    """A `Closeness` object of the report: a count per bucket."""
    raw = field(obj, key, where, task)
    return {b: count(raw, b, f"{where}.{key}", task) for b in BUCKETS}


@dataclass(frozen=True)
class FunctionParity:
    """One in-scope function: its tier and cells (T-0068) and its Phase 1 rate (T-0080)."""

    name: str
    category: str
    milestone: str
    tier: str
    attribution: str
    """`sole` when its attributed cells placed the tier, `all` for every Excel cell (fallback)."""
    attributed: dict[str, int]
    """Excel cells whose formula's sole non-glue call is this function, per bucket."""
    excel: dict[str, int]
    """Every Excel cell that calls this function, per bucket."""
    rate_cells: int
    """The scored cells the Phase 1 rate counts (`v15_checkpoint.functions[].rate_cells`)."""
    rate_visible: int
    """Of those, the visibly-off cells."""
    meets_rate: bool
    """Covered and at least 99.9% of `rate_cells` within 15 digits (decisions 0114, 0118)."""
    evidence: int
    """Every Excel-evidence cell that calls it, accepted deviations included
    (`v15_checkpoint.functions[].cells`; T-0073)."""
    covered: bool
    """Enough Excel cells for the fifteen-digit checkpoint (decision 0094; `covered`)."""
    prior: float | None
    """The catalog's real-world frequency `prior` (eval/catalog/functions.yaml); None when the
    catalog sets none or no priors were given (T-0073)."""
    real_world_calls: int = 0
    """Formula cells of the Excel-saved corpora (calipers, poi) that call it (report.json
    `functions[].real_world_calls`, T-0079)."""

    @property
    def attributed_cells(self) -> int:
        """Its scored attributed cells."""
        return sum(self.attributed.values())

    @property
    def excel_cells(self) -> int:
        """Its scored Excel cells."""
        return sum(self.excel.values())

    @property
    def rate_within15(self) -> int:
        """Of `rate_cells`, those within 15 digits."""
        return max(0, self.rate_cells - self.rate_visible)

    @property
    def attributed_within15(self) -> int:
        """Of its attributed cells, those within 15 digits (not visibly off)."""
        return self.attributed_cells - sum(self.attributed[b] for b in VISIBLE)


@dataclass(frozen=True)
class TierCount:
    """One row of report.json `tiers`: the in-scope functions in a tier, and the fallbacks."""

    tier: str
    functions: int
    fallback: int


@dataclass(frozen=True)
class Phase1:
    """The Phase 1 numbers of report.json `v15_checkpoint` (decisions 0114 and 0118, T-0080)."""

    in_scope: int
    at_rate: int
    met: bool
    at_strict_rate: int
    """Information only: the stricter every-function count (decision 0118 item 5)."""
    v15_met: bool
    """The fifteen-digit checkpoint (decision 0094): `v15_checkpoint.met`."""
    covered: int
    """In-scope functions with decision 0094's coverage."""
    visibly_off: int
    """In-scope visibly-off cells over the checkpoint's suites."""


@dataclass(frozen=True)
class Suite:
    """One eval suite's counts (report.json `suites[]`) and its known failures."""

    name: str
    cases: int
    cells: int
    passed: int
    deviations: int
    ulp: int
    """Cells within 4 ulp but not bit-exact (the `numeric_ulp` verdict)."""
    visible: int
    """In-scope visibly-off cells (`closeness`)."""
    workbooks: dict[str, int]
    timeouts: int
    known_failures: int | None
    """`F` lines of eval/baseline/<suite>.txt; None when the suite has no baseline."""
    text_known_failures: int
    """`TF` lines: displayed-text failures (decision 0070)."""

    @property
    def scored(self) -> int:
        """Cells in the bit-exact denominator: all but accepted deviations (decision 0036)."""
        return max(0, self.cells - self.deviations)


@dataclass(frozen=True)
class Parity:
    """The parity section's data: every in-scope function, the tier counts, Phase 1, suites."""

    functions: list[FunctionParity]
    tiers: list[TierCount]
    phase1: Phase1
    suites: list[Suite]
    milestones_exited: tuple[str, ...]
    """The milestones whose report.json `milestone_exit` row is met (decision 0085)."""
    real_world_suites: tuple[str, ...] = ()
    """The suites whose formula cells `real_world_calls` counts (report.json `real_world_suites`)."""

    def tier_functions(self, tier: str) -> int:
        """In-scope functions in `tier` (report.json `tiers`)."""
        return next(t.functions for t in self.tiers if t.tier == tier)

    def categories(self) -> list[tuple[str, Counter[str]]]:
        """Functions per tier for each catalog category, the largest category first."""
        groups: dict[str, Counter[str]] = defaultdict(Counter)
        for f in self.functions:
            groups[f.category][f.tier] += 1
        return sorted(groups.items(), key=lambda kv: (-kv[1].total(), kv[0]))

    def attributed_cells(self) -> Counter[str]:
        """Attributed Excel cells per bucket over every in-scope function. A cell is attributed to
        at most one function, so this counts each cell once."""
        total: Counter[str] = Counter(dict.fromkeys(BUCKETS, 0))
        for f in self.functions:
            total.update(f.attributed)
        return total


def _tiers(report: dict[str, Any]) -> list[TierCount]:
    rows = items(report, "tiers", "report.json", "T-0068")
    tiers = [
        TierCount(
            one_of(r, "tier", f"report.json tiers[{i}]", TIERS),
            count(r, "functions", f"report.json tiers[{i}]"),
            count(r, "fallback", f"report.json tiers[{i}]"),
        )
        for i, r in enumerate(rows)
    ]
    if [t.tier for t in tiers] != list(TIERS):
        msg = (
            f"report.json `tiers` lists {[t.tier for t in tiers]}, not every tier once in order {list(TIERS)}"
        )
        raise ProgressReportError(msg)
    return tiers


def _phase1(v15: Any) -> Phase1:
    where = "report.json v15_checkpoint"
    return Phase1(
        count(v15, "in_scope", where),
        count(v15, "at_rate", where, "T-0080"),
        flag(v15, "phase1_met", where, "T-0080"),
        count(v15, "at_strict_rate", where, "T-0080"),
        flag(v15, "met", where),
        count(v15, "covered", where),
        count(v15, "visibly_off", where),
    )


def known_failures(path: Path) -> tuple[int | None, int]:
    """The (`F`, `TF`) line counts of an eval baseline file: its known value and displayed-text
    failures (crates/rustsheet-eval/src/baseline.rs). (None, 0) when the suite has no baseline."""
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except FileNotFoundError:
        return None, 0
    except (OSError, UnicodeDecodeError) as e:
        msg = f"{path}: {e}"
        raise ProgressReportError(msg) from e
    kinds = Counter(line.split("\t", 1)[0] for line in lines if not line.startswith("#"))
    return kinds["F"], kinds["TF"]


def _suite(raw: Any, i: int, baseline_dir: Path) -> Suite:
    where = f"report.json suites[{i}]"
    name = field(raw, "name", where)
    if not isinstance(name, str) or not re.fullmatch(r"[A-Za-z0-9._-]+", name):
        msg = f"{where}.name is not a suite name: {name!r}"
        raise ProgressReportError(msg)
    where = f"report.json suite {name!r}"
    verdicts = mapping(raw, "verdicts", where)
    statuses = mapping(raw, "statuses", where)
    visible = closeness(raw, "closeness", where, "T-2036")
    workbooks_raw = field(raw, "workbooks", where, "T-0068")
    workbooks = {k: count(workbooks_raw, k, f"{where}.workbooks", "T-0068") for k in WORKBOOKS}
    ulp = count(verdicts, "numeric_ulp", f"{where}.verdicts") if "numeric_ulp" in verdicts else 0
    timeouts = count(statuses, "timeout", f"{where}.statuses") if "timeout" in statuses else 0
    known, text_known = known_failures(baseline_dir / f"{name}.txt")
    return Suite(
        name,
        count(raw, "cases", where),
        count(raw, "cells", where),
        count(raw, "cells_passed", where),
        count(raw, "deviations", where),
        ulp,
        sum(visible[b] for b in VISIBLE),
        workbooks,
        timeouts,
        known,
        text_known,
    )


def compute(
    report: dict[str, Any],
    scope: dict[str, Any],
    baseline_dir: Path,
    priors: Mapping[str, float | None] | None = None,
) -> Parity:
    """The parity section's data for every in-scope catalog function of `scope`.

    `priors` is the catalog's `prior` per function (evidence.load_priors); when given, every
    in-scope function must have an entry, so a catalog that is not scope.json's source fails.

    Each function's tier, attribution and cells are the report's `functions[]` entry (T-0068); a
    catalog function no cell calls has none and is `unscored`, as report.json `tiers` counts it.
    Its Phase 1 rate is its `v15_checkpoint.functions[]` entry (T-0080). The per-tier counts are
    checked against report.json `tiers`, so a catalog that is not the run's fails loudly.
    """
    catalog: dict[str, dict[str, Any]] = {}
    for i, f in enumerate(items(scope, "functions", "scope.json")):
        name = field(f, "name", f"scope.json functions[{i}]")
        if f.get("scope", "in") == "in":
            catalog[str(name)] = f
    stats = {
        str(field(f, "name", "report.json functions[]")): f for f in items(report, "functions", "report.json")
    }
    v15 = mapping(report, "v15_checkpoint", "report.json", "T-0080")
    rates = {
        str(field(f, "name", "report.json v15_checkpoint.functions[]")): f
        for f in items(v15, "functions", "report.json v15_checkpoint")
    }
    rows = []
    for name in sorted(catalog):
        cat = catalog[name]
        where = f"report.json function {name!r}"
        stat = stats.get(name)
        if stat is None:
            tier, attribution = "unscored", "all"
            attributed = excel = dict.fromkeys(BUCKETS, 0)
            calls = 0
        else:
            tier = one_of(stat, "tier", where, TIERS, "T-0068")
            attribution = one_of(stat, "attribution", where, ATTRIBUTIONS, "T-0068")
            attributed = closeness(stat, "attributed", where, "T-0068")
            excel = closeness(stat, "excel", where, "T-0068")
            calls = count(stat, "real_world_calls", where, "T-0079")
        rate = rates.get(name)
        if rate is None:
            msg = f"report.json v15_checkpoint has no entry for in-scope function {name}; is scope.json the run's catalog?"
            raise ProgressReportError(msg)
        rwhere = f"report.json v15_checkpoint function {name!r}"
        if priors is not None and name not in priors:
            msg = f"the catalog has no function {name}; is scope.json its view?"
            raise ProgressReportError(msg)
        rows.append(
            FunctionParity(
                name,
                str(field(cat, "category", f"scope.json {name}")),
                str(field(cat, "milestone", f"scope.json {name}")),
                tier,
                attribution,
                attributed,
                excel,
                count(rate, "rate_cells", rwhere, "T-0080"),
                count(rate, "rate_visible", rwhere, "T-0080"),
                flag(rate, "meets_rate", rwhere, "T-0080"),
                evidence=count(rate, "cells", rwhere),
                covered=flag(rate, "covered", rwhere),
                prior=priors.get(name) if priors is not None else None,
                real_world_calls=calls,
            )
        )
    tiers = _tiers(report)
    placed = Counter(f.tier for f in rows)
    if any(placed[t.tier] != t.functions for t in tiers):
        msg = (
            f"scope.json places {dict(placed)} functions per tier, but report.json `tiers` counts "
            f"{ {t.tier: t.functions for t in tiers} }; is scope.json the run's catalog?"
        )
        raise ProgressReportError(msg)
    suites = [_suite(s, i, baseline_dir) for i, s in enumerate(items(report, "suites", "report.json"))]
    exits = items(report, "milestone_exit", "report.json")
    exited = tuple(
        str(field(row, "milestone", f"report.json milestone_exit[{i}]"))
        for i, row in enumerate(exits)
        if flag(row, "exit", f"report.json milestone_exit[{i}]")
    )
    corpora = tuple(str(s) for s in items(report, "real_world_suites", "report.json", "T-0079"))
    return Parity(rows, tiers, _phase1(v15), suites, exited, corpora)


def load_object(path: Path) -> dict[str, Any]:
    """`path` parsed as a JSON object; a missing or malformed file is a `ProgressReportError`."""
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        msg = f"{path}: {e}"
        raise ProgressReportError(msg) from e
    if not isinstance(data, dict):
        msg = f"{path}: not a JSON object"
        raise ProgressReportError(msg)
    return data


# The run's commit and date are printed into the page, so they must be what `run_meta.rs` writes.
COMMIT_SHA = re.compile(r"[0-9a-f]{4,40}")
RUN_DATE = re.compile(r"[0-9]{4}-[0-9]{2}-[0-9]{2}")  # ASCII digits only, unlike `\d`


@dataclass(frozen=True)
class SkippedSuite:
    """A suite under eval/suites that the run did not score, and why (`run.skipped_suites`)."""

    name: str
    reason: str


@dataclass(frozen=True)
class Run:
    """The eval run the page shows: report.json `run` (T-0080) and the suites it scored."""

    date: str
    commit: str | None
    """`git rev-parse --short=8 HEAD` when the report was made; None when git failed."""
    suites: tuple[str, ...]
    skipped: tuple[SkippedSuite, ...]


def run_of(report: dict[str, Any]) -> Run:
    """The report's `run` record, checked (crates/rustsheet-eval/src/run_meta.rs)."""
    run = report.get("run")
    if not isinstance(run, dict):
        msg = "report.json has no `run` record; it predates T-0080, re-run `make eval`"
        raise ReportShapeError(msg)
    date, commit, skipped = run.get("date"), run.get("commit"), run.get("skipped_suites")
    if not isinstance(date, str) or not RUN_DATE.fullmatch(date):
        msg = f"report.json `run.date` is not a YYYY-MM-DD date: {date!r}"
        raise ProgressReportError(msg)
    if commit is not None and (not isinstance(commit, str) or not COMMIT_SHA.fullmatch(commit)):
        msg = f"report.json `run.commit` is not a commit sha (4-40 lower-case hex digits): {commit!r}"
        raise ProgressReportError(msg)
    if not isinstance(skipped, list) or not all(
        isinstance(s, dict) and isinstance(s.get("name"), str) and isinstance(s.get("reason"), str)
        for s in skipped
    ):
        msg = "report.json `run.skipped_suites` is not a list of {name, reason}"
        raise ProgressReportError(msg)
    suites = tuple(
        str(field(s, "name", "report.json suites[]")) for s in items(report, "suites", "report.json")
    )
    return Run(date, commit, suites, tuple(SkippedSuite(s["name"], s["reason"]) for s in skipped))


# The task statuses the page counts, and the open ones: scripts/tasks.py `STATUSES` without
# `dropped`, and its `OPEN` (not done or dropped).
TASK_STATUSES = ("todo", "in-progress", "blocked", "done")
OPEN_STATUSES = ("todo", "in-progress", "blocked")
# The decision-0114 campaign stages (scripts/tasks.py `CAMPAIGNS`), and the parked row: the
# `defer: phase-1b` tasks, tasks.json `deferred` (decision 0119).
STAGES = ("semantics", "accuracy", "visible")
PARKED = "parked"


@dataclass(frozen=True)
class Tasks:
    """Task counts from tasks.json (`scripts/tasks.py stats --json`, T-0069)."""

    by_status: dict[str, int]
    """Every task per status in TASK_STATUSES, summed over the milestones."""
    by_stage: dict[str, dict[str, int]]
    """Per stage of STAGES and PARKED: `open`, `blocked` and `done` tasks."""
    ready: dict[str, int]
    """Per stage of STAGES and PARKED: the tasks `ready()` on main, ignoring branch claims."""
    by_milestone: dict[str, dict[str, int]]
    """Per milestone tasks.json lists: its tasks per status in TASK_STATUSES."""


def _stage_counts(raw: Any, where: str) -> dict[str, int]:
    status = {s: count(raw, s, where) for s in TASK_STATUSES}
    return {
        "open": sum(status[s] for s in OPEN_STATUSES),
        "blocked": status["blocked"],
        "done": status["done"],
    }


def tasks_of(raw: dict[str, Any]) -> Tasks:
    """The counts of a parsed tasks.json, checked."""
    milestones = mapping(raw, "by_milestone", "tasks.json")
    by_milestone = {
        name: {s: count(per, s, f"tasks.json by_milestone.{name}") for s in TASK_STATUSES}
        for name, per in sorted(milestones.items())
    }
    by_status = {s: sum(per[s] for per in by_milestone.values()) for s in TASK_STATUSES}
    campaign = mapping(raw, "by_campaign", "tasks.json")
    stages = {
        s: (mapping(campaign, s, "tasks.json by_campaign"), f"tasks.json by_campaign.{s}") for s in STAGES
    }
    stages[PARKED] = (mapping(raw, "deferred", "tasks.json"), "tasks.json deferred")
    by_stage = {s: _stage_counts(per, where) for s, (per, where) in stages.items()}
    ready = {s: count(per, "ready", where) for s, (per, where) in stages.items()}
    return Tasks(by_status, by_stage, ready, by_milestone)
