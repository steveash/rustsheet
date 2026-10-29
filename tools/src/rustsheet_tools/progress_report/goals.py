"""The progress page's `goals` and `campaign` blocks (SPEC-0022 "Page layout", T-0072).

`goals` shows the Phase 1 exit (decision 0114, counted as decision 0118 says) and the
fifteen-digit checkpoint (decision 0094) as the eval report writes them, whether the previous
complete run in the history also met the checkpoint, and one row per milestone of
docs/milestones.md: its phase, its checked exit criteria, the report's exit measures (decision
0085), catalog coverage, frozen-list (decision 0043) and probe-milestone (decision 0051) failures,
and its tasks per status.

`campaign` shows one row per decision-0114 campaign stage and one for the parked
`defer: phase-1b` tasks: their task counts (tasks.json, `scripts/tasks.py stats --json`,
decision 0119), the visibly-off attributed cells of the functions below the Phase 1 rate, and the
functions that fixing their type, error and text cells alone would lift to the rate.

The rate is the report's (`v15_checkpoint.rate_num / rate_den`) and the functions below it are
the report's tiers (T-0068); this module only adds up the report's counts. docs/milestones.md is
read for its phase table and its exit checklists only.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import Any

from rustsheet_tools.progress_report.markdown import md_text, table
from rustsheet_tools.progress_report.model import (
    EN,
    GE,
    PARKED,
    STAGES,
    TASK_STATUSES,
    TIERS,
    FunctionParity,
    Parity,
    ProgressReportError,
    Tasks,
    count,
    field,
    flag,
    items,
    mapping,
)
from rustsheet_tools.progress_report.parity import TIER_LABELS

MILESTONES_MD = "docs/milestones.md"
# A milestone heading of docs/milestones.md: `## M<n> — <title>`.
MILESTONE_HEADING = re.compile(r"## (M\d+) \N{EM DASH} (.+)")
EXIT_CRITERIA = "**Exit criteria:**"
CHECKBOX = re.compile(r"\s*- \[([ xX])\] ")
# A bold label that starts the next part of a milestone section (`**Eval suites:**`).
LABEL = re.compile(r"\*\*[^*]+:\*\*")
PHASE_HEADING = "## Phase order"
# A phase-table label: `1. Parity (fifteen digits)`, `1b. Bit parity`.
PHASE_LABEL = re.compile(r"(\d+[a-z]?)\. .+")
MILESTONE_ID = re.compile(r"M(\d+)")
# A report probe-milestone row: `M5`, or a range `M0-M2` (decision 0051).
PROBE_ROW = re.compile(r"M(\d+)(?:-M(\d+))?")
# A frozen probe list: `m1-probes` is M1's (decision 0043).
FROZEN_LIST = re.compile(r"m(\d+)-[a-z0-9-]+")
# The tiers below the Phase 1 rate: under the `rate999` tier, scored (attribution.rs `Tier`).
BELOW_BAR = TIERS[TIERS.index("rate999") + 1 : TIERS.index("unscored")]
STAGE_LABELS = {
    "semantics": "Semantics",
    "accuracy": "Accuracy",
    "visible": "Visible",
    PARKED: "Parked (defer: phase-1b)",
}
STAGE_FIXES = {
    "semantics": "wrong type, error or text, and semantic rules",
    "accuracy": "more accurate kernels where Excel is close to correctly rounded",
    "visible": "Excel's own algorithm where Excel is the inaccurate one (time-boxed)",
    PARKED: "last-digit work on cells already within 15 digits (Phase 1b)",
}


@dataclass(frozen=True)
class MilestoneDoc:
    """One `## M<n>` section of docs/milestones.md."""

    milestone: str
    title: str
    phase: str
    """The phase-table label's number: `1`, `2`, ..."""
    checked: int
    """`- [x]` items of its exit criteria."""
    boxes: int
    """`- [x]` and `- [ ]` items of its exit criteria; 0 when they are prose."""


def _number(milestone: str) -> int:
    m = MILESTONE_ID.fullmatch(milestone)
    if m is None:
        msg = f"{milestone!r} is not a milestone id (M<n>)"
        raise ProgressReportError(msg)
    return int(m.group(1))


def _phases(lines: list[str]) -> dict[str, str]:
    """Milestone -> phase number, from the phase table under `## Phase order`."""
    start = next((i for i, line in enumerate(lines) if line.startswith(PHASE_HEADING)), None)
    if start is None:
        msg = f"{MILESTONES_MD} has no `{PHASE_HEADING}` section"
        raise ProgressReportError(msg)
    phases: dict[str, str] = {}
    for line in lines[start + 1 :]:
        if line.startswith("## "):
            break
        cells = [c.strip() for c in line.strip().strip("|").split("|")] if line.startswith("|") else []
        if len(cells) < 2 or not (label := PHASE_LABEL.fullmatch(cells[0])):
            continue
        for milestone in MILESTONE_ID.findall(cells[1]):
            name = f"M{milestone}"
            if name in phases:
                msg = f"{MILESTONES_MD}: the phase table lists {name} twice"
                raise ProgressReportError(msg)
            phases[name] = label.group(1)
    if not phases:
        msg = f"{MILESTONES_MD}: the `{PHASE_HEADING}` section has no phase table"
        raise ProgressReportError(msg)
    return phases


def _exit_boxes(milestone: str, section: list[str]) -> tuple[int, int]:
    """The (checked, total) checkboxes of a section's exit criteria, which run from the
    `**Exit criteria:**` line to the next bold label or the section's end."""
    try:
        start = next(i for i, line in enumerate(section) if line.startswith(EXIT_CRITERIA))
    except StopIteration:
        msg = f"{MILESTONES_MD}: ## {milestone} has no `{EXIT_CRITERIA}` list"
        raise ProgressReportError(msg) from None
    checked = boxes = 0
    for line in section[start + 1 :]:
        if LABEL.match(line):
            break
        if box := CHECKBOX.match(line):
            boxes += 1
            checked += box.group(1) != " "
    return checked, boxes


def parse_milestones(text: str) -> list[MilestoneDoc]:
    """Every `## M<n>` section of docs/milestones.md, by milestone number. Fails loudly on a
    milestone with no `**Exit criteria:**` line, a duplicate heading, or a milestone that is not
    in the phase table exactly once (or one the table names with no heading)."""
    lines = text.splitlines()
    phases = _phases(lines)
    starts = [(i, m) for i, line in enumerate(lines) if (m := MILESTONE_HEADING.fullmatch(line))]
    docs: dict[str, MilestoneDoc] = {}
    for i, m in starts:
        end = next((j for j in range(i + 1, len(lines)) if lines[j].startswith("## ")), len(lines))
        name = m.group(1)
        if name in docs:
            msg = f"{MILESTONES_MD}: ## {name} appears twice"
            raise ProgressReportError(msg)
        if name not in phases:
            msg = f"{MILESTONES_MD}: ## {name} is in no row of the phase table"
            raise ProgressReportError(msg)
        checked, boxes = _exit_boxes(name, lines[i + 1 : end])
        docs[name] = MilestoneDoc(name, m.group(2).strip(), phases[name], checked, boxes)
    if missing := sorted(set(phases) - set(docs), key=_number):
        msg = f"{MILESTONES_MD}: the phase table names {', '.join(missing)}, which have no ## heading"
        raise ProgressReportError(msg)
    if not docs:
        msg = f"{MILESTONES_MD} has no `## M<n>` milestone heading"
        raise ProgressReportError(msg)
    return sorted(docs.values(), key=lambda d: _number(d.milestone))


@dataclass(frozen=True)
class Goals:
    """The report's fields the two blocks read (report.json; T-0072 added the rate and lines)."""

    phase1_line: str
    verdict_line: str
    rate_num: int
    rate_den: int
    exit_rows: dict[str, dict[str, Any]]
    """report.json `milestone_exit` by milestone (decision 0085)."""
    coverage: dict[str, tuple[int, int]]
    """report.json `catalog_coverage`: milestone -> (covered, functions)."""
    probes: dict[str, list[str]]
    """Per milestone, its frozen-list and probe-milestone failures, as page text."""

    @property
    def rate(self) -> str:
        """The Phase 1 rate as a percentage: `99.9%`."""
        return f"{100 * self.rate_num / self.rate_den:g}%"

    def meets_rate(self, within: int, cells: int) -> bool:
        """`within` of `cells` reach the rate (milestone_exit.rs `meets_rate`); never with none."""
        return cells > 0 and within * self.rate_den >= cells * self.rate_num


def _text(obj: Any, key: str, where: str, task: str | None = None) -> str:
    value = field(obj, key, where, task)
    if not isinstance(value, str) or not value:
        msg = f"{where}.{key} is not a non-empty string: {value!r}"
        raise ProgressReportError(msg)
    return value


def _probe_milestones(row: str) -> list[str]:
    """The milestones a probe-milestone row covers: `M0-M2` is M0, M1 and M2."""
    m = PROBE_ROW.fullmatch(row)
    if m is None or (m.group(2) is not None and int(m.group(2)) < int(m.group(1))):
        msg = f"report.json probe_milestones has a row {row!r}, not M<n> or M<a>-M<b>"
        raise ProgressReportError(msg)
    last = int(m.group(2) or m.group(1))
    return [f"M{n}" for n in range(int(m.group(1)), last + 1)]


def _probes(report: dict[str, Any]) -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for i, row in enumerate(items(report, "milestone_lists", "report.json")):
        where = f"report.json milestone_lists[{i}]"
        name = _text(row, "name", where)
        m = FROZEN_LIST.fullmatch(name)
        if m is None:
            msg = f"{where}.name is {name!r}, not a frozen list m<n>-<name>"
            raise ProgressReportError(msg)
        failing = len(items(row, "failing_cases", where))
        out.setdefault(f"M{m.group(1)}", []).append(
            f"frozen list {name}: {count(row, 'known_failures', where):,} known failures in "
            f"{failing:,} cases ({count(row, 'cases_run', where):,} of {count(row, 'cases', where):,} run)"
        )
    for i, row in enumerate(items(report, "probe_milestones", "report.json")):
        where = f"report.json probe_milestones[{i}]"
        name = _text(row, "milestone", where)
        line = (
            f"probes {name}: {count(row, 'visibly_off', where):,} visibly off, "
            f"{count(row, 'known_failures', where):,} known failures"
        )
        for milestone in _probe_milestones(name):
            out.setdefault(milestone, []).append(line)
    return out


def load(report: dict[str, Any]) -> Goals:
    """The goals fields of a parsed report.json; a report from before T-0072 is an older shape
    (`ReportShapeError`), so the run is marked stale, not broken."""
    v15 = mapping(report, "v15_checkpoint", "report.json", "T-0080")
    where = "report.json v15_checkpoint"
    rate_num, rate_den = count(v15, "rate_num", where, "T-0072"), count(v15, "rate_den", where, "T-0072")
    if not 0 < rate_num <= rate_den:
        msg = f"{where} rate {rate_num}/{rate_den} is not a share in (0, 1]"
        raise ProgressReportError(msg)
    exit_rows = {}
    for i, row in enumerate(items(report, "milestone_exit", "report.json")):
        name = _text(row, "milestone", f"report.json milestone_exit[{i}]")
        exit_rows[name] = row
    coverage = {}
    for i, row in enumerate(items(report, "catalog_coverage", "report.json")):
        cwhere = f"report.json catalog_coverage[{i}]"
        coverage[_text(row, "milestone", cwhere)] = (
            count(row, "covered", cwhere),
            count(row, "functions", cwhere),
        )
    return Goals(
        _text(v15, "phase1_line", where, "T-0072"),
        _text(v15, "verdict_line", where, "T-0072"),
        rate_num,
        rate_den,
        exit_rows,
        coverage,
        _probes(report),
    )


def _optional(row: dict[str, Any], key: str, where: str, *, boolean: bool = False) -> int | bool | None:
    """A `milestone_exit` measure: null when its input was absent from the run (milestone_exit.rs)."""
    if field(row, key, where) is None:
        return None
    return flag(row, key, where) if boolean else count(row, key, where)


def _verdict(met: int | bool | None) -> str:
    return "not in run" if met is None else "met" if met else "not met"


def _num(n: int | bool | None) -> str:
    return "?" if n is None else f"{n:,}"


def exit_measures(milestone: str, row: dict[str, Any]) -> str:
    """One `milestone_exit` row as page text: the exit, then each measure met or not."""
    where = f"report.json milestone_exit {milestone}"

    def get(key: str) -> int | bool | None:
        return _optional(row, key, where)

    def met(key: str) -> str:
        return _verdict(_optional(row, key, where, boolean=True))

    def shown(count_key: str, met_key: str) -> str:
        """Counts without a verdict: not one of the milestone's criteria (coverage for M2,
        decision 0135; coverage and ironcalc-calc for M7, decision 0140)."""
        required = get(count_key) is None or field(row, met_key, where) is not None
        return met(met_key) if required else "not required"

    exited = "**exit met**" if flag(row, "exit", where) else "**exit not met**"
    # The first required suite is the generated one but for M11's corpus-bench (decision 0146).
    suite = row.get("gen_suite") or ""
    label = suite if suite and suite != f"gen-{milestone.lower()}" else "generated suite"
    measures = [
        exited,
        f"coverage {_num(get('covered'))}/{_num(get('functions'))} ({shown('covered', 'coverage_met')})",
    ]
    # A row that requires no suite (M9, decision 0146 item 3) has an empty `gen_suite`.
    if "gen_suite" in row and not suite:
        measures.append("no suite required")
    else:
        measures.append(
            f"{label} {_num(get('gen_visibly_off'))} visibly off of {_num(get('gen_scored'))}, "
            f"allowance {_num(get('gen_allowance'))} ({met('gen_met')})"
        )
    # The other required suites (M1's smoke, M8's gen-m8b and poi subset), each with its own
    # floor and rate; the poi subset also counts the cases that did not run (decision 0146).
    for i, more in enumerate(row.get("more_suites") or []):
        mwhere = f"{where} more_suites[{i}]"
        blocked = len(more.get("blocked_cases") or [])
        measures.append(
            f"{_text(more, 'name', mwhere)} {_num(_optional(more, 'visibly_off', mwhere))} visibly off of "
            f"{_num(_optional(more, 'scored', mwhere))}, allowance {_num(_optional(more, 'allowance', mwhere))}"
            f"{f', {blocked:,} cases blocked' if blocked else ''} "
            f"({_verdict(_optional(more, 'met', mwhere, boolean=True))})"
        )
    # M9's ironcalc-calc is a rate with an allowance (decision 0146 item 3).
    calc_of = (
        f" of {_num(get('calc_scored'))}, allowance {_num(get('calc_allowance'))}"
        if row.get("calc_allowance") is not None
        else ""
    )
    measures += [
        f"ironcalc-calc {_num(get('calc_visibly_off'))} visibly off{calc_of} "
        f"({shown('calc_visibly_off', 'calc_met')})",
        f"probes {_num(get('probe_visibly_off'))} visibly off ({met('probes_met')})",
    ]
    # Only M9 requires every function at the Phase 1 rate (decision 0146 item 3).
    if row.get("rate_functions") is not None or row.get("rate_met") is not None:
        measures.append(
            f"functions at the Phase 1 rate {_num(get('below_rate'))} below of "
            f"{_num(get('rate_functions'))} ({met('rate_met')})"
        )
    # Only a milestone with the criterion (M7, decision 0140) reports spill extents.
    if row.get("extent_failures") is not None or row.get("extent_met") is not None:
        measures.append(f"spill extents {_num(get('extent_failures'))} failures ({met('extent_met')})")
    return "; ".join(measures)


def _check_known(what: str, names: set[str], docs: list[MilestoneDoc]) -> None:
    """Fail on a milestone the input counts but docs/milestones.md has no heading for, so its
    numbers are never silently dropped from the table."""
    if unknown := sorted(names - {d.milestone for d in docs}):
        msg = f"{what} counts milestone {', '.join(unknown)}, which {MILESTONES_MD} has no heading for"
        raise ProgressReportError(msg)


def milestone_table(g: Goals, docs: list[MilestoneDoc], tasks: Tasks) -> str:
    """One row per milestone of docs/milestones.md."""
    _check_known("tasks.json by_milestone", set(tasks.by_milestone), docs)
    _check_known("report.json milestone_exit", set(g.exit_rows), docs)
    _check_known("report.json catalog_coverage", set(g.coverage), docs)
    _check_known("report.json milestone_lists and probe_milestones", set(g.probes), docs)
    rows = []
    for d in docs:
        covered, functions = g.coverage.get(d.milestone, (0, 0))
        coverage = (
            EN
            if d.milestone not in g.coverage
            else "no functions"
            if functions == 0
            else f"{covered:,}/{functions:,}"
        )
        exit_row = g.exit_rows.get(d.milestone)
        per = tasks.by_milestone.get(d.milestone, dict.fromkeys(TASK_STATUSES, 0))
        rows.append(
            [
                f"{d.milestone} {d.title}",
                d.phase,
                f"{d.checked}/{d.boxes}" if d.boxes else "prose",
                coverage,
                exit_measures(d.milestone, exit_row) if exit_row is not None else EN,
                "; ".join(g.probes.get(d.milestone, [])) or EN,
                *(f"{per[s]:,}" for s in TASK_STATUSES),
            ]
        )
    header = ["Milestone", "Phase", "Exit criteria checked", "Catalog coverage", "Exit measures (report)"]
    return table([*header, "Frozen list and probes", "Todo", "In progress", "Blocked", "Done"], rows, 6)


def previous_checkpoint(last: dict[str, Any] | None, met: bool) -> str:
    """Whether the previous complete run in the history also met the checkpoint (decision 0094
    asks for two runs in a row)."""
    if last is None:
        return (
            "The history has no earlier complete run, so whether the previous run met the "
            "checkpoint is not known yet."
        )
    where = f"history.json row {last['date']}"
    before = flag(mapping(last, "v15", where), "met", f"{where}.v15")
    then = f"commit `{last['commit']}`" if last["commit"] else "an unknown commit"
    line = (
        f"At the previous complete run, **{last['date']}** at {then}, the checkpoint was {_verdict(before)}"
    )
    if met and before:
        return line + ": it is met on two complete runs in a row."
    return line + "."


def goals_block(
    g: Goals, docs: list[MilestoneDoc], data: Parity, tasks: Tasks, last: dict[str, Any] | None
) -> str:
    """The `goals` block; `last` is the last complete history row before this run."""
    exited = ", ".join(data.milestones_exited) or "none"
    return f"""## Goals

**{md_text(g.phase1_line)}**

**{md_text(g.verdict_line)}** {previous_checkpoint(last, data.phase1.v15_met)}

Milestones whose exit row in the report is met: {exited}.

### Milestones

One row per milestone of [docs/milestones.md](milestones.md), in milestone order; the phase
table there gives the order they are worked in. "Exit criteria checked" counts the `- [x]` items
of the milestone's exit criteria ("prose" when they are not a checklist). Catalog coverage, the
exit measures (decision 0085, read at visible parity by decision 0114), the frozen probe lists
(decision 0043) and the probe milestones (decision 0051) are the eval report's rows; a probe row
such as M0{EN}M2 is shown on each milestone it covers. Tasks are `scripts/tasks.py stats --json`.

{milestone_table(g, docs, tasks)}"""


def lifts(g: Goals, f: FunctionParity) -> bool:
    """Whether fixing `f`'s visibly-off type, error and text cells alone brings its counted cells
    (the Phase 1 rate's: attributed, or every Excel cell below the floor) to the rate."""
    counted = f.attributed if f.attribution == "sole" else f.excel
    fixed = min(counted["visible_other"], f.rate_visible)
    return g.meets_rate(f.rate_within15 + fixed, f.rate_cells)


def campaign_block(g: Goals, data: Parity, tasks: Tasks) -> str:
    """The `campaign` block: a row per stage and the parked row."""
    # A function whose tier is under the bar but which meets the rate had its only off cells left
    # out of the rate (decision 0134), so no fix lifts it.
    below = [f for f in data.functions if f.tier in BELOW_BAR and not f.meets_rate]
    other = sum(f.attributed["visible_other"] for f in below)
    numeric = sum(f.attributed["visible_numeric"] for f in below)
    lifted = sorted(f.name for f in below if lifts(g, f))
    bar = f"{GE} {g.rate}"
    off = {
        "semantics": f"{other:,} type, error or text",
        "accuracy": f"{numeric:,} numbers, accuracy and visible together",
        "visible": "in the accuracy row",
        PARKED: EN,
    }
    rows = []
    for stage in (*STAGES, PARKED):
        st = tasks.by_stage[stage]
        rows.append(
            [
                STAGE_LABELS[stage],
                STAGE_FIXES[stage],
                f"{st['open']:,}",
                f"{tasks.ready[stage]:,}",
                f"{st['blocked']:,}",
                f"{st['done']:,}",
                off[stage],
                f"{len(lifted):,} of {len(below):,}" if stage == "semantics" else EN,
            ]
        )
    header = [
        "Stage",
        "Fixes",
        "Open",
        "Ready",
        "Blocked",
        "Done",
        f"Visibly-off cells (functions under {g.rate})",
    ]
    tiers = " and ".join(TIER_LABELS[t] for t in BELOW_BAR)
    names = ", ".join(lifted) if lifted else "none"
    return f"""## Campaign stages

Decision 0114 works the visibly-off cells in stages, picked in this order: semantics, then
accuracy, then visible. **{len(below):,}** in-scope functions are below the Phase 1 rate
({bar} of their counted cells within 15 digits; tiers {tiers}). Their attributed Excel cells hold
**{other:,}** visibly-off type, error or text cells and **{numeric:,}** visibly-off numbers.
Fixing only the type, error and text cells would lift **{len(lifted):,}** of them to {bar}.

{table([*header, f"Functions lifted to {bar}"], rows, 2)}

- **Tasks** are `scripts/tasks.py stats --json`. Open is todo, in progress or blocked. Ready is
  the tasks ready on `main`, ignoring branch claims, so a claimed task still counts. A task with a
  stage and `defer: phase-1b` counts in its stage row and in the parked row (decision 0119), so the
  rows do not add up to a total.
- **Visibly-off cells** are the attributed cells of the functions below the rate. Numbers are
  shown for the accuracy and visible stages together: telling them apart needs a high-precision
  reference and per-family judgment, which a daily run cannot repeat (SPEC-0022, "Alternatives
  considered").
- **Lifted by the semantic fixes alone:** {names}."""


METHOD = f"""- **Goals.** The Phase 1 and checkpoint lines are report.json `v15_checkpoint.phase1_line`
  and `verdict_line`, as the eval wrote them; the previous run's checkpoint is the last complete
  row of docs/progress/history.json. The milestone rows read {MILESTONES_MD} for the phase table
  and the exit checklists, report.json `catalog_coverage`, `milestone_exit`, `milestone_lists`
  and `probe_milestones`, and tasks.json `by_milestone`.
- **Campaign stages.** Task counts are tasks.json `by_campaign` and `deferred` (decision 0119).
  The functions below the rate are those whose report tier is under `rate999` and whose
  `meets_rate` is false (decision 0134 leaves listed cells out of the rate, not the tier); a function is
  lifted when its counted cells (`v15_checkpoint.functions[].rate_cells`) reach the report's rate
  (`rate_num / rate_den`) once its visibly-off type, error and text cells are fixed."""
