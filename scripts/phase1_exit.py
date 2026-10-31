#!/usr/bin/env python3
"""One verdict per Phase 1 exit criterion of docs/milestones.md (decision 0187).

  scripts/phase1_exit.py [--report PATH] [--tasks DIR] [--catalog DIR] [--json]

Reads `eval/runs/latest/report.json` (written by `make eval`) and the task tree, and prints one
line per criterion, `met`, `not met` or `manual`, with the numbers behind it, then an overall
line. The criteria and their measures are decision 0187's:
- `run`: the run skipped no suite and was not dirty (otherwise nothing below can be trusted);
- `catalog`: eval/catalog/excel-functions.tsv and functions.yaml name the same functions, each
  with a parity scope (T-0040);
- `function rate`: `v15_checkpoint.phase1_met`, with no stale or unchecked exception line;
- `owners`: `exit_owners.unowned_off` and `exit_owners.unowned` for every Phase 1 milestone list
  no cell (a cell owned only by a `defer:` task is unowned);
- `milestone M<k>`: the report's `milestone_exit` row (an owner count rides beside a met row);
- `manual`: the prose items decision 0146 checks outside the rows.
Verdicts come from report.json and exit_owners.py; nothing is recomputed here. Exit 1 when any
measured line is `not met`, 2 when the report cannot be read, else 0.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))
import exit_owners  # noqa: E402 (the sys.path line above must run first)
import tasks  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
CATALOG = ROOT / "eval" / "catalog"
MET, NOT_MET, MANUAL = "met", "not met", "manual"
# The Phase 1 milestones whose exit criteria the Phase 1 exit requires (docs/milestones.md).
PHASE1_MILESTONES = ("M1", "M2", "M5", "M6", "M7", "M8", "M9", "M10", "M11")
# Decision 0146: exit criteria measured outside the report's rows, with where they are written.
MANUAL_ITEMS = (
    ("M1 parity bugs and API v0", "docs/milestones.md, M1 exit criteria (decision 0146 item 1)"),
    ("M2 one walker, one matcher", "docs/milestones.md, M2 exit criteria (decision 0137)"),
)
# scripts/ is stdlib-only (no YAML reader), so the catalog is read by these line patterns; they
# follow functions.yaml's two entry shapes, and test_phase1_exit.py runs them on the real files so
# a layout change fails there (tools/tests/test_catalog.py checks the same sets through PyYAML).
# A functions.yaml entry, block style (`  - name: SUM`, `  - name: "N"`) or flow style
# (`  - { name: NOW, ..., scope: in, ... }`), and a block entry's `scope:` line.
CATALOG_NAME = re.compile(r'^  - (?:\{ )?name: "?([^",\s}]+)"?')
CATALOG_FLOW_SCOPE = re.compile(r"[{,] scope: (\w+)")
CATALOG_SCOPE = re.compile(r"^    scope: (\S+)\s*$")


@dataclass(frozen=True)
class Line:
    """One criterion's verdict and the numbers behind it."""

    criterion: str
    verdict: str
    detail: str


def run_line(report: dict[str, Any]) -> Line:
    run = report.get("run") or {}
    skipped = [str(s.get("name")) for s in run.get("skipped_suites") or []]
    dirty = bool(run.get("dirty"))
    problems = ([f"skipped suites: {', '.join(skipped)}"] if skipped else []) + (
        ["dirty tree"] if dirty else []
    )
    commit = run.get("commit", "?")
    if problems:
        return Line("run", NOT_MET, f"at {commit}: {'; '.join(problems)}")
    return Line("run", MET, f"at {commit}: every suite ran, clean tree")


def excel_function_names(path: Path) -> set[str]:
    """The `name` column of excel-functions.tsv (`#` lines are provenance, the first other line the header)."""
    rows = [ln for ln in path.read_text(encoding="utf-8").splitlines() if ln and not ln.startswith("#")]
    return {ln.split("\t", 1)[0] for ln in rows[1:]}


def catalog_scopes(path: Path) -> dict[str, str]:
    """Each functions.yaml entry's scope (`in` when the entry has no `scope:` key)."""
    scopes: dict[str, str] = {}
    current = None
    for line in path.read_text(encoding="utf-8").splitlines():
        if m := CATALOG_NAME.match(line):
            current = m.group(1)
            flow = CATALOG_FLOW_SCOPE.search(line)
            scopes[current] = flow.group(1) if flow else "in"
        elif current and (m := CATALOG_SCOPE.match(line)):
            scopes[current] = m.group(1)
    return scopes


def catalog_line(catalog_dir: Path, report: dict[str, Any] | None = None) -> Line:
    """The catalog against Microsoft's list and, given a report, its out-of-scope set against the
    report's `out_of_scope` list (the functions the run left out)."""
    excel = excel_function_names(catalog_dir / "excel-functions.tsv")
    scopes = catalog_scopes(catalog_dir / "functions.yaml")
    out_names = {n for n, sc in scopes.items() if sc == "out"}
    reported = None if report is None else {str(o.get("name")) for o in report.get("out_of_scope") or []}
    missing = sorted(excel - set(scopes))
    extra = sorted(set(scopes) - excel)
    bad_scope = sorted(n for n, s in scopes.items() if s not in ("in", "out"))
    problems = (
        ([f"not in the catalog: {', '.join(missing)}"] if missing else [])
        + ([f"not an Excel function: {', '.join(extra)}"] if extra else [])
        + ([f"no parity scope: {', '.join(bad_scope)}"] if bad_scope else [])
        + (
            [f"report out_of_scope differs: {', '.join(sorted(reported ^ out_names))}"]
            if reported is not None and reported != out_names
            else []
        )
    )
    if problems:
        return Line("catalog", NOT_MET, "; ".join(problems))
    out = len(out_names)
    return Line("catalog", MET, f"{len(excel)} Excel functions, {len(excel) - out} in scope, {out} out")


def rate_line(report: dict[str, Any]) -> Line:
    v15 = report.get("v15_checkpoint") or {}
    stale = list(v15.get("stale_exceptions") or [])
    unchecked = list(v15.get("unchecked_exceptions") or [])
    met = v15.get("phase1_met") is True and not stale and not unchecked
    detail = f"{v15.get('phase1_line') or 'no phase1_line in the report'}"
    if stale:
        detail += f"; {len(stale)} stale exception lines"
    if unchecked:
        detail += f"; {len(unchecked)} unchecked exception lines"
    return Line("function rate", MET if met else NOT_MET, detail)


def owner_counts(
    report: dict[str, Any], tasks_dir: Path, milestones: list[str]
) -> tuple[int, int, dict[str, int]]:
    """(unowned `--all-off` cells, of them owned only by a deferred task, unowned exit cells per milestone)."""
    off = exit_owners.unowned_off(report, tasks_dir)
    per = {m: len(exit_owners.unowned(report, m, tasks_dir)) for m in milestones}
    return len(off), sum(1 for _, deferred in off if deferred), per


def owners_line(off: int, deferred: int, per: dict[str, int]) -> Line:
    exits = sum(per.values())
    detail = f"{off} visibly-off cells with no open owner ({deferred} owned only by a deferred task)"
    detail += f"; {exits} milestone exit cells with no owner of their milestone"
    if exits:
        detail += " (" + ", ".join(f"{m} {n}" for m, n in per.items() if n) + ")"
    return Line("owners", MET if off == 0 and exits == 0 else NOT_MET, detail)


def milestone_lines(report: dict[str, Any], per: dict[str, int]) -> list[Line]:
    rows = {str(r.get("milestone")): r for r in report.get("milestone_exit") or []}
    lines = []
    for m in PHASE1_MILESTONES:
        row = rows.get(m)
        if row is None:
            lines.append(Line(f"milestone {m}", NOT_MET, "no milestone_exit row in the report"))
            continue
        met = row.get("exit") is True
        detail = "exit row met" if met else "exit row not met"
        if per.get(m):
            detail += f"; {per[m]} exit cells have no {m} owner (see owners)"
        lines.append(Line(f"milestone {m}", MET if met else NOT_MET, detail))
    return lines


def verdicts(report: dict[str, Any], tasks_dir: Path, catalog_dir: Path) -> list[Line]:
    rows = [str(r.get("milestone")) for r in report.get("milestone_exit") or []]
    milestones = [m for m in PHASE1_MILESTONES if m in rows]
    off, deferred, per = owner_counts(report, tasks_dir, milestones)
    return [
        run_line(report),
        catalog_line(catalog_dir, report),
        rate_line(report),
        owners_line(off, deferred, per),
        *milestone_lines(report, per),
        *(Line(f"manual: {item}", MANUAL, where) for item, where in MANUAL_ITEMS),
    ]


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--report", type=Path, default=exit_owners.REPORT)
    ap.add_argument("--tasks", type=Path, default=tasks.TASKS)
    ap.add_argument("--catalog", type=Path, default=CATALOG)
    ap.add_argument("--json", action="store_true", help="print the lines and the overall verdict as JSON")
    args = ap.parse_args(argv)
    lines = exit_owners.with_report(args.report, lambda r: verdicts(r, args.tasks, args.catalog))
    if lines is None:
        return 2
    failing = [ln.criterion for ln in lines if ln.verdict == NOT_MET]
    overall = NOT_MET if failing else MET
    if args.json:
        print(
            json.dumps(
                {"overall": overall, "failing": failing, "lines": [asdict(ln) for ln in lines]}, indent=2
            )
        )
    else:
        for ln in lines:
            print(f"{ln.criterion}\t{ln.verdict}\t{ln.detail}")
        tail = f": {', '.join(failing)}" if failing else " (check the manual lines by hand)"
        print(f"Phase 1 exit: {overall}{tail}")
    return 1 if failing else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
