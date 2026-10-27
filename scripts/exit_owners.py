#!/usr/bin/env python3
"""Milestone exit cells that no open task of the milestone owns (decision 0096).

  scripts/exit_owners.py --milestone M5 [--report PATH] [--tasks DIR] [--strict]
  scripts/exit_owners.py --unattributed [--report PATH] [--tasks DIR] [--strict]

Reads `eval/runs/latest/report.json` (written by `make eval`) and lists every visibly-off cell of
the milestone's exit criteria (decision 0085, read at visible parity by decisions 0114 and 0135:
the `gen-m<k>` suite, the ironcalc-calc row of "Milestone function scope" less its exit
exceptions, and the probes of the milestone's row of "Probe milestones", `M0-M2` for M2), plus
each probe case that fails with no cell, that no open task of that milestone names, one per line as
`criterion<TAB>case!Sheet!A1<TAB>formula`, then a count line. `--strict` exits 1 when the count is
not 0.

A task owns a cell when its front matter has the milestone and an open status, and its text names
the case's file stem (the last path segment of the case id less `.case.json`: a gen hash, an
xlsx file name, a probe name) and either the cell's A1 name as a whole word, or the stem on a line
with no A1-shaped token (a task that names only the case owns all of it). The ironcalc-calc
exceptions are the row's `excepted` cells, which `make eval` takes from
eval/milestones/exit-exceptions.tsv.

`--unattributed` lists instead the report's `unattributed_off` cells (decision 0118 item 6,
decision 0132, T-0082: visibly-off Excel cells attributed to no function; a `DEVIATION` cell is
never among them) that no open task of any milestone owns, by the same rule, as
`case!Sheet!A1<TAB>formula`, then a count line. It gates nothing (`--strict` aside).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tasks  # noqa: E402 (the sys.path line above must run first)

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
    def stem(self) -> str:
        return self.case.rsplit("/", 1)[-1].removesuffix(SPEC_SUFFIX)

    @property
    def a1(self) -> str | None:
        return None if self.key == WHOLE_CASE else self.key.rsplit("!", 1)[-1]


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

    gen = f"gen-{milestone.lower()}"
    gen_off = set(exit_row.get("gen_visibly_off_cells") or [])
    for case in report.get("cases") or []:
        if case.get("suite") == gen:
            for c in by_case.get(str(case["id"]), []):
                key = f"{c['sheet']}!{c['cell']}"
                if f"{case['id']}!{key}" in gen_off:
                    out.append(ExitCell(gen, str(case["id"]), key, formula_of(c)))

    scope = report.get("function_scope") or {}
    for row in scope.get("rows") or []:
        if row.get("milestone") == milestone:
            excepted = set(row.get("excepted") or [])
            visible = set(row.get("visibly_off") or [])
            for name in row.get("failing") or []:
                if name not in excepted and name in visible:
                    case, _, key = str(name).partition("!")
                    out.append(ExitCell(CALC_SUITE, case, key, formulas.get((case, key), "")))

    probe_row = exit_row.get("probe_row") or milestone
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


def open_task_texts(tasks_dir: Path, milestone: str | None) -> list[str]:
    """The text of every open task of `milestone` (any milestone when `None`) in `tasks_dir`."""
    out = []
    for path in sorted(tasks_dir.glob("*/T-*.md")):
        text = path.read_text()
        meta = tasks.front_matter(text, path)
        if milestone in (None, meta.get("milestone")) and meta.get("status") in tasks.OPEN:
            out.append(text)
    return out


def names(text: str, token: str, word: str) -> bool:
    """Whether `text` contains `token` with no `word` character on either side."""
    return re.search(rf"(?<!{word}){re.escape(token)}(?!{word})", text) is not None


# A case stem is bounded like a file name; an A1 name like a cell reference (`Calc!A107` names
# A107, `A10` does not name A1).
STEM_WORD = r"[\w.-]"
A1_WORD = r"[\w$]"
A1_TOKEN = re.compile(rf"(?<!{A1_WORD})\$?[A-Z]{{1,3}}\$?[0-9]{{1,7}}(?!\w)")


def names_whole_case(text: str, stem: str) -> bool:
    """Whether a line of `text` names `stem` and no cell: a task that lists cells beside a case
    owns only those, even when they have since passed and left the report."""
    return any(names(line, stem, STEM_WORD) and not A1_TOKEN.search(line) for line in text.splitlines())


def owned(cell: ExitCell, texts: list[str]) -> bool:
    """Whether one of `texts` owns `cell`."""
    for text in texts:
        if not names(text, cell.stem, STEM_WORD):
            continue
        if cell.a1 is not None and names(text, cell.a1, A1_WORD):
            return True
        if names_whole_case(text, cell.stem):
            return True
    return False


def unowned(report: dict[str, Any], milestone: str, tasks_dir: Path) -> list[ExitCell]:
    cells = exit_cells(report, milestone)
    texts = open_task_texts(tasks_dir, milestone)
    seen: set[tuple[str, str, str]] = set()
    out = []
    for c in cells:
        ident = (c.criterion, c.case, c.key)
        if ident not in seen and not owned(c, texts):
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
    return [c for c in cells if not owned(c, texts)]


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    which = ap.add_mutually_exclusive_group(required=True)
    which.add_argument("--milestone", help="M<n>, e.g. M5")
    which.add_argument(
        "--unattributed",
        action="store_true",
        help="unowned cells attributed to no function",
    )
    ap.add_argument("--report", type=Path, default=REPORT)
    ap.add_argument("--tasks", type=Path, default=tasks.TASKS)
    ap.add_argument("--strict", action="store_true", help="exit 1 when any cell is unowned")
    args = ap.parse_args(argv)
    if args.unattributed:
        return main_unattributed(args.report, args.tasks, args.strict)
    if not tasks.MILESTONE.match(args.milestone):
        print(f"error: --milestone must be M<n>, not {args.milestone!r}", file=sys.stderr)
        return 2
    if not args.report.is_file():
        print(f"error: {args.report} not found; run `make eval` first", file=sys.stderr)
        return 2
    try:
        report = json.loads(args.report.read_text())
        cells = unowned(report, args.milestone, args.tasks)
    except (ValueError, KeyError, TypeError) as e:
        print(f"error: {args.report}: {e}", file=sys.stderr)
        return 2
    for c in cells:
        print(f"{c.criterion}\t{c.case}!{c.key}\t{c.formula}")
    print(f"{len(cells)} failing {args.milestone} exit cells have no open {args.milestone} owner")
    return 1 if args.strict and cells else 0


def main_unattributed(report_path: Path, tasks_dir: Path, strict: bool) -> int:
    if not report_path.is_file():
        print(f"error: {report_path} not found; run `make eval` first", file=sys.stderr)
        return 2
    try:
        cells = unowned_unattributed(json.loads(report_path.read_text()), tasks_dir)
    except (ValueError, KeyError, TypeError) as e:
        print(f"error: {report_path}: {e}", file=sys.stderr)
        return 2
    for c in cells:
        print(f"{c.case}!{c.key}\t{c.formula}")
    print(f"{len(cells)} visibly-off cells attributed to no function have no open owner")
    return 1 if strict and cells else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
