#!/usr/bin/env python3
"""Milestone exit cells that no open task of the milestone owns (decision 0096).

  scripts/exit_owners.py --milestone M5 [--report PATH] [--tasks DIR] [--strict]
  scripts/exit_owners.py --unattributed [--report PATH] [--tasks DIR] [--strict]
  scripts/exit_owners.py --all-off [--function NAME] [--report PATH] [--tasks DIR] [--strict]

Reads `eval/runs/latest/report.json` (written by `make eval`) and lists every visibly-off cell of
the milestone's exit criteria (decision 0085, read at visible parity by decisions 0114 and 0135:
the `gen-m<k>` suite and any other required suite of the exit row (decision 0146: M1's smoke,
M11's corpus-bench), the ironcalc-calc row of "Milestone function scope" less its exit
exceptions when the exit row requires it (not M7, decision 0140), and the probes of the
milestone's row of "Probe milestones", `M0-M2` for M2, or for M1 the frozen `m1-probes` list), plus each probe case that fails with no
cell, that no open task of that milestone names, one per line as
`criterion<TAB>case!Sheet!A1<TAB>formula`, then a count line. `--strict` exits 1 when the count is
not 0.

A task owns a cell when its front matter has the milestone and an open status, and its text names
the case (`case_names`): by its file stem (the last path segment of the case id less `.case.json`:
a gen hash, an xlsx file name, a probe name; with or without that suffix) when no other case of the
report has that stem, else only by a longer suffix of the case id that is unique in the report
(`pv26_011/golden.xlsx`, `t2436/<hash>`, `gen-m9/<hash>`) or by its directory below the suite when
no other case of the report is in a directory of that name (`pv26_011`; decision 0138), a
unique stem also by its directory path when no other case is under it (`corpus-bench/53062`;
a name may end a sentence with `.`; decision 0139), and
either the cell's A1 name as a whole word, or a colon span (`A5:D5`) holding it on a line that
names the case, or the case on a line with no A1-shaped token (a task that names only the case
owns all of it). Hyphen spans (`D49-D65`) name their ends only (decision 0136). The ironcalc-calc
exceptions are the row's `excepted` cells, which `make eval` takes from
eval/milestones/exit-exceptions.tsv.

`--unattributed` lists instead the report's `unattributed_off` cells (decision 0118 item 6,
decision 0132, T-0082: visibly-off Excel cells attributed to no function; a `DEVIATION` cell is
never among them) that no open task of any milestone owns, by the same rule, as
`case!Sheet!A1<TAB>formula`, then a count line. It gates nothing (`--strict` aside).

`--all-off` lists every visibly-off Excel cell no open task owns (T-0086): the report's
`attributed_off` cells (decision 0118; the eval applies the visibly-off rule of closeness.rs and
names the sole call, so no rule is repeated here) grouped by function, then `unattributed_off`,
then the cells of the probe-milestone rows (every milestone) that neither list holds, as
`group<TAB>case!Sheet!A1<TAB>formula<TAB>expected<TAB>actual`, then a count per group and a
total. A cell owned only by a `defer:` task is unowned and shows `deferred` after its group
(`GAMMA (deferred)`); `--function NAME` keeps one group.
"""

from __future__ import annotations

import argparse
import functools
import json
import re
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, TypeVar

sys.path.insert(0, str(Path(__file__).resolve().parent))
# tools/ is a uv project, but sheetop is stdlib-only, so this script stays stdlib-only too.
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "tools" / "src"))
import tasks  # noqa: E402 (the sys.path lines above must run first)
from rustsheet_tools.sheetop import column_number  # noqa: E402

T = TypeVar("T")
ROOT = Path(__file__).resolve().parent.parent
REPORT = ROOT / "eval" / "runs" / "latest" / "report.json"
CALC_SUITE = "ironcalc-calc"
PROBES_SUITE = "probes"
# A failing item of a case that is not a cell (`*`, `!Panic`, ...): the whole case.
WHOLE_CASE = "*"
PASSING = ("pass", "deviation")
# Spec cases are `<name>.case.json` files; tasks name them without the suffix.
SPEC_SUFFIX = ".case.json"


@dataclass(frozen=True)
class ExitCell:
    """One failing cell of an exit criterion."""

    criterion: str
    case: str
    # `Sheet!A1`, or WHOLE_CASE.
    key: str
    formula: str

    @property
    def a1(self) -> str | None:
        return None if self.key == WHOLE_CASE else self.key.rsplit("!", 1)[-1]


def case_names(cases: list[str]) -> dict[str, tuple[str, ...]]:
    """Each case id → the names a task may call it by (decision 0138). A stem that one case has
    names that case. A stem that several cases share (every calipers `golden.xlsx`, a gen-m9
    hash rerun as `probes/t2436/<hash>`) names none of them: such a case is named by each suffix
    of its id, of two segments or more, that no other case has, and by its directory (below the
    suite, the id's first segment) when no other case is in a directory of that name. A case with
    its own stem that is below the suite is also named by its directory path (the id less its
    last segment, suite included) when no other case of the report is under that path."""
    ids = sorted(set(cases))
    parts = {c: c.removesuffix(SPEC_SUFFIX).split("/") for c in ids}
    stems = Counter(p[-1] for p in parts.values())
    suffixes = Counter("/".join(p[k:]) for p in parts.values() for k in range(len(p) - 1))
    dirs = Counter(p[-2] for p in parts.values() if len(p) > 2)
    # Cases under each directory path below the suite, and the ids a path may collide with.
    under = Counter("/".join(p[:k]) for p in parts.values() for k in range(2, len(p)))
    whole = {"/".join(p) for p in parts.values()}
    out: dict[str, tuple[str, ...]] = {}
    for c, p in parts.items():
        if stems[p[-1]] == 1:
            # `corpus-bench/53062` for `corpus-bench/53062/1_53062_input.xlsx` (decision 0139).
            folder = "/".join(p[:-1])
            alone = len(p) > 2 and under[folder] == 1 and folder not in whole
            out[c] = (p[-1], folder) if alone else (p[-1],)
            continue
        found = ["/".join(p[k:]) for k in range(len(p) - 1) if suffixes["/".join(p[k:])] == 1]
        if len(p) > 2 and dirs[p[-2]] == 1:
            found.append(p[-2])
        out[c] = tuple(found)
    return out


def failing_cells(report: dict[str, Any]) -> dict[str, list[dict[str, Any]]]:
    """Case id → its failing cells, from the report's `cases` (it keeps only failing cells)."""
    out: dict[str, list[dict[str, Any]]] = {}
    for case in report.get("cases") or []:
        cells = [c for c in case.get("cells") or [] if c.get("verdict") not in PASSING]
        out.setdefault(str(case["id"]), []).extend(cells)
    return out


def formula_of(cell: dict[str, Any]) -> str:
    return str(cell.get("formula") or cell.get("anchor_formula") or "")


def exit_cells(report: dict[str, Any], milestone: str) -> list[ExitCell]:
    """Every visibly-off cell of `milestone`'s exit criteria, in report order per criterion. The
    visibly-off lists are the report's (decision 0135): the exit row's `gen_visibly_off_cells`,
    the function-scope row's `visibly_off` and the probe row's `visibly_off_cells`."""
    exit_row = next((r for r in report.get("milestone_exit") or [] if r.get("milestone") == milestone), None)
    if exit_row is None:
        raise ValueError(f"report.json has no milestone_exit row for {milestone}")
    by_case = failing_cells(report)
    formulas = {
        (case, f"{c['sheet']}!{c['cell']}"): formula_of(c) for case, cells in by_case.items() for c in cells
    }
    out: list[ExitCell] = []

    # Every required suite (decision 0146): the row's first (`gen_suite`, `gen-m<k>` before it
    # existed) and `more_suites`, each with its own visibly-off list.
    suites = [(exit_row.get("gen_suite") or f"gen-{milestone.lower()}", exit_row.get("gen_visibly_off_cells"))]
    suites += [(str(s.get("name")), s.get("visibly_off_cells")) for s in exit_row.get("more_suites") or []]
    for suite, cells_off in suites:
        off = set(cells_off or [])
        for case in report.get("cases") or []:
            if case.get("suite") == suite:
                for c in by_case.get(str(case["id"]), []):
                    key = f"{c['sheet']}!{c['cell']}"
                    if f"{case['id']}!{key}" in off:
                        out.append(ExitCell(suite, str(case["id"]), key, formula_of(c)))

    # A function-scope row the exit row shows but does not require (counts with a null
    # `calc_met`, M7) is no exit criterion (decision 0140).
    shown_only = exit_row.get("calc_failing") is not None and exit_row.get("calc_met") is None
    scope = {} if shown_only else report.get("function_scope") or {}
    for row in scope.get("rows") or []:
        if row.get("milestone") == milestone:
            excepted = set(row.get("excepted") or [])
            visible = set(row.get("visibly_off") or [])
            for name in row.get("failing") or []:
                if name not in excepted and name in visible:
                    case, key = split_cell_name(str(name))
                    out.append(ExitCell(CALC_SUITE, case, key, formulas.get((case, key), "")))

    probe_row = exit_row.get("probe_row") or milestone
    # M1's probe measure is the frozen `m1-probes` list, not a probe row (decision 0146 item 1).
    for row in report.get("milestone_lists") or []:
        if row.get("name") == probe_row:
            for name in row.get("visibly_off_cells") or []:
                case, key = split_cell_name(str(name))
                out.append(ExitCell(probe_row, case, key, formulas.get((case, key), "")))
    for row in report.get("probe_milestones") or []:
        if row.get("milestone") == probe_row:
            probe_off = set(row.get("visibly_off_cells") or [])
            for case, _count in row.get("failing_cases") or []:
                cells = by_case.get(case, [])
                if not cells:
                    out.append(ExitCell(PROBES_SUITE, case, WHOLE_CASE, ""))
                for c in cells:
                    key = f"{c['sheet']}!{c['cell']}"
                    if f"{case}!{key}" in probe_off:
                        out.append(ExitCell(PROBES_SUITE, case, key, formula_of(c)))
    return out


def open_tasks(tasks_dir: Path) -> list[tuple[dict[str, object], str]]:
    """The front matter and text of every open task in `tasks_dir`, in path order."""
    out = []
    for path in sorted(tasks_dir.glob("*/T-*.md")):
        text = path.read_text()
        meta = tasks.front_matter(text, path)
        if meta.get("status") in tasks.OPEN:
            out.append((meta, text))
    return out


def open_task_texts(tasks_dir: Path, milestone: str | None, deferred: bool | None = None) -> list[str]:
    """The text of every open task of `milestone` (any milestone when `None`) in `tasks_dir`;
    with `deferred` set, only the tasks that do (`True`) or do not (`False`) carry `defer:`."""
    return [
        text
        for meta, text in open_tasks(tasks_dir)
        if milestone in (None, meta.get("milestone")) and deferred in (None, "defer" in meta)
    ]


def names(text: str, token: str, word: str) -> bool:
    """Whether `text` contains `token` with no `word` character on either side."""
    return re.search(rf"(?<!{word}){re.escape(token)}(?!{word})", text) is not None


# A case stem is bounded like a file name; an A1 name like a cell reference (`Calc!A107` names
# A107, `A10` does not name A1). A name may end a sentence: a `.` that no file-name character
# follows does not extend it (`corpus-bench/55049.`; decision 0139).
STEM_WORD = r"[\w.-]"
STEM_END = r"(?![\w-]|\.[\w.-])"
A1_WORD = r"[\w$]"
A1_PART = r"\$?([A-Z]{1,3})\$?([0-9]{1,7})"
A1_TOKEN = re.compile(rf"(?<!{A1_WORD}){A1_PART}(?!\w)")
# `A5:D5` beside a stem names its whole rectangle, as an A1 reference does (decision 0136);
# `D49-D65` does not.
A1_SPAN = re.compile(rf"(?<!{A1_WORD}){A1_PART}:{A1_PART}(?!\w)")
A1_SPLIT = re.compile(A1_PART)


@functools.lru_cache(maxsize=None)  # `--all-off` asks the same text and case for every cell of the case
def names_case(text: str, names: tuple[str, ...]) -> bool:
    """Whether `text` holds one of a case's `names` (`case_names`), bare or followed by
    `.case.json` (decision 0136), bounded like a file name."""
    return any(
        re.search(rf"(?<!{STEM_WORD}){re.escape(n)}(?:{re.escape(SPEC_SUFFIX)})?{STEM_END}", text)
        for n in names
    )


def names_in_span(text: str, names: tuple[str, ...], a1: str) -> bool:
    """Whether a colon span on a line of `text` that names the case holds the cell `a1`. Per line,
    unlike a single A1 name, because tasks quote formulas whose ranges (`Data!B6:C10`) are not
    cells they own."""
    m = A1_SPLIT.fullmatch(a1)
    if m is None:
        return False
    col, row = column_number(m[1]), int(m[2])
    for line in text.splitlines():
        if not names_case(line, names):
            continue
        for s in A1_SPAN.finditer(line):
            c1, c2 = sorted((column_number(s[1]), column_number(s[3])))
            r1, r2 = sorted((int(s[2]), int(s[4])))
            if c1 <= col <= c2 and r1 <= row <= r2:
                return True
    return False


@functools.lru_cache(maxsize=None)
def names_whole_case(text: str, names: tuple[str, ...]) -> bool:
    """Whether a line of `text` names the case and no cell: a task that lists cells beside a case
    owns only those, even when they have since passed and left the report."""
    return any(names_case(line, names) and not A1_TOKEN.search(line) for line in text.splitlines())


def owned(cell: ExitCell, texts: list[str], called: tuple[str, ...]) -> bool:
    """Whether one of `texts` owns `cell`, a case called by `called`."""
    for text in texts:
        if not names_case(text, called):
            continue
        if cell.a1 is not None and (names(text, cell.a1, A1_WORD) or names_in_span(text, called, cell.a1)):
            return True
        if names_whole_case(text, called):
            return True
    return False


def report_case_names(report: dict[str, Any], cells: list[ExitCell]) -> dict[str, tuple[str, ...]]:
    """`case_names` over every case of the report and of `cells`."""
    ids = [str(c["id"]) for c in report.get("cases") or []]
    return case_names(ids + [c.case for c in cells])


def unowned(report: dict[str, Any], milestone: str, tasks_dir: Path) -> list[ExitCell]:
    cells = exit_cells(report, milestone)
    texts = open_task_texts(tasks_dir, milestone)
    called = report_case_names(report, cells)
    seen: set[tuple[str, str, str]] = set()
    out = []
    for c in cells:
        ident = (c.criterion, c.case, c.key)
        if ident not in seen and not owned(c, texts, called[c.case]):
            seen.add(ident)
            out.append(c)
    return out


UNATTRIBUTED = "unattributed"


def unowned_unattributed(report: dict[str, Any], tasks_dir: Path) -> list[ExitCell]:
    """The report's unattributed visibly-off cells that no open task owns, in report order."""
    texts = open_task_texts(tasks_dir, None)
    cells = [
        ExitCell(UNATTRIBUTED, str(c["case"]), str(c["cell"]), str(c.get("formula") or ""))
        for c in report.get("unattributed_off") or []
    ]
    called = report_case_names(report, cells)
    return [c for c in cells if not owned(c, texts, called[c.case])]


@dataclass(frozen=True)
class OffCell:
    """One visibly-off Excel cell for `--all-off`: its group (a function, `unattributed` or
    `probe M<k>`) and the values."""

    cell: ExitCell
    expected: str
    actual: str


def show_value(value: Any) -> str:
    """A report `Value` (`{"t": "n", "v": 1.5}`) as one printable token."""
    if not isinstance(value, dict):
        return str(value)
    kind, inner = value.get("t"), value.get("v")
    if kind == "empty":
        return "<empty>"
    if kind == "b":
        return "TRUE" if inner else "FALSE"
    return json.dumps(inner) if kind == "s" else str(inner)


def all_off_cells(report: dict[str, Any]) -> list[OffCell]:
    """Every visibly-off Excel cell of the report: attributed (by function, sorted), unattributed,
    then the probe-milestone cells the first two lists lack."""
    by_key = {
        (str(case["id"]), f"{c['sheet']}!{c['cell']}"): c
        for case in report.get("cases") or []
        for c in case.get("cells") or []
    }

    def off(group: str, case: str, key: str, formula: str, got: tuple[str, str] | None = None) -> OffCell:
        c = by_key.get((case, key), {})
        e, a = got or (show_value(c.get("expected")), show_value(c.get("actual")))
        return OffCell(ExitCell(group, case, key, formula), e, a)

    attributed = sorted(
        report.get("attributed_off") or [], key=lambda c: str(c["function"])
    )  # stable: case order within a function
    out = [
        off(str(c["function"]), str(c["case"]), str(c["cell"]), str(c.get("formula") or ""),
            (str(c.get("expected", "")), str(c.get("actual", ""))))
        for c in attributed
    ]
    out += [
        off(UNATTRIBUTED, str(c["case"]), str(c["cell"]), str(c.get("formula") or ""))
        for c in report.get("unattributed_off") or []
    ]
    seen = {(o.cell.case, o.cell.key) for o in out}
    cases_with_cells = {k[0] for k in by_key}
    for row in report.get("probe_milestones") or []:
        group = f"probe {row.get('milestone')}"
        for name in row.get("visibly_off_cells") or []:
            case, key = split_cell_name(str(name))
            if (case, key) not in seen:
                seen.add((case, key))
                out.append(off(group, case, key, formula_of(by_key.get((case, key), {}))))
        for case, _count in row.get("failing_cases") or []:
            if case not in cases_with_cells and (case, WHOLE_CASE) not in seen:
                seen.add((case, WHOLE_CASE))
                out.append(off(group, str(case), WHOLE_CASE, ""))
    return out


def split_cell_name(name: str) -> tuple[str, str]:
    """`case!Sheet!A1` → (`case`, `Sheet!A1`); a case id holds no `!`."""
    case, _, key = name.partition("!")
    return case, key


def unowned_off(
    report: dict[str, Any], tasks_dir: Path
) -> list[tuple[OffCell, bool]]:
    """`all_off_cells` that no open, non-deferred task owns, each with whether a deferred task
    owns it. Texts are narrowed to the ones naming the case once per case: the naive loop over
    every cell and task is minutes on a full report."""
    cells = all_off_cells(report)
    called = report_case_names(report, [o.cell for o in cells])
    live = open_task_texts(tasks_dir, None, deferred=False)
    parked = open_task_texts(tasks_dir, None, deferred=True)
    naming: dict[str, tuple[list[str], list[str]]] = {}
    out = []
    for o in cells:
        c = o.cell
        if c.case not in naming:
            names_of = called[c.case]
            naming[c.case] = (
                [t for t in live if names_case(t, names_of)],
                [t for t in parked if names_case(t, names_of)],
            )
        mine, theirs = naming[c.case]
        if not owned(c, mine, called[c.case]):
            out.append((o, owned(c, theirs, called[c.case])))
    return out


def with_report(report_path: Path, compute: Callable[[dict[str, Any]], T]) -> T | None:
    """`compute` over the parsed report, or `None` after printing why it could not (exit 2)."""
    if not report_path.is_file():
        print(f"error: {report_path} not found; run `make eval` first", file=sys.stderr)
        return None
    try:
        return compute(json.loads(report_path.read_text()))
    except (ValueError, KeyError, TypeError) as e:
        print(f"error: {report_path}: {e}", file=sys.stderr)
        return None


def main_all_off(report_path: Path, tasks_dir: Path, function: str | None, strict: bool) -> int:
    listed = with_report(report_path, lambda r: unowned_off(r, tasks_dir))
    if listed is None:
        return 2
    cells = [(o, d) for o, d in listed if function in (None, o.cell.criterion)]
    counts: Counter[str] = Counter()
    for o, deferred in cells:
        group = f"{o.cell.criterion} (deferred)" if deferred else o.cell.criterion
        counts[o.cell.criterion] += 1
        print(f"{group}\t{o.cell.case}!{o.cell.key}\t{o.cell.formula}\t{o.expected}\t{o.actual}")
    for group, n in counts.items():
        print(f"{group}: {n}")
    print(f"{len(cells)} visibly-off cells have no open owner")
    return 1 if strict and cells else 0


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    which = ap.add_mutually_exclusive_group(required=True)
    which.add_argument("--milestone", help="M<n>, e.g. M5")
    which.add_argument(
        "--unattributed",
        action="store_true",
        help="unowned cells attributed to no function",
    )
    which.add_argument(
        "--all-off",
        action="store_true",
        help="every unowned visibly-off Excel cell, grouped by function",
    )
    ap.add_argument("--function", help="with --all-off: only this group (a function name, `unattributed` or `probe M<k>`)")
    ap.add_argument("--report", type=Path, default=REPORT)
    ap.add_argument("--tasks", type=Path, default=tasks.TASKS)
    ap.add_argument("--strict", action="store_true", help="exit 1 when any cell is unowned")
    args = ap.parse_args(argv)
    if args.all_off:
        return main_all_off(args.report, args.tasks, args.function, args.strict)
    if args.unattributed:
        return main_unattributed(args.report, args.tasks, args.strict)
    if not tasks.MILESTONE.match(args.milestone):
        print(f"error: --milestone must be M<n>, not {args.milestone!r}", file=sys.stderr)
        return 2
    cells = with_report(args.report, lambda r: unowned(r, args.milestone, args.tasks))
    if cells is None:
        return 2
    for c in cells:
        print(f"{c.criterion}\t{c.case}!{c.key}\t{c.formula}")
    print(f"{len(cells)} failing {args.milestone} exit cells have no open {args.milestone} owner")
    return 1 if args.strict and cells else 0


def main_unattributed(report_path: Path, tasks_dir: Path, strict: bool) -> int:
    cells = with_report(report_path, lambda r: unowned_unattributed(r, tasks_dir))
    if cells is None:
        return 2
    for c in cells:
        print(f"{c.case}!{c.key}\t{c.formula}")
    print(f"{len(cells)} visibly-off cells attributed to no function have no open owner")
    return 1 if strict and cells else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
