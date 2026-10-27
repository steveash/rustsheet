"""Generate the daily progress page, docs/progress.md, from the eval report (SPEC-0022).

Reads `eval/runs/latest/report.json` (written by `make eval`), `eval/catalog/scope.json`, the
eval baselines, `eval/runs/latest/tasks.json` (`scripts/tasks.py stats --json`),
docs/milestones.md, git, and the existing page and run history under `--out-dir` (the
repository's docs directory; `make progress-report` passes it). A complete run writes
`progress.md`, keeping the steward's judgment blocks byte for byte, and `progress/` (the
all-functions page, the charts and history.json). An incomplete run (no report, an older report
shape, or a skipped suite) rewrites only the page's status block as stale, adds a
`complete: false` history row and exits 0. Every input is checked and every file rendered before
anything is written, so a failed run leaves the pages as they were.

With `--check` it writes nothing: it checks the judgment blocks of the existing page (check.py,
T-0074) and exits 1, listing every offender, when one fails.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from rustsheet_tools import repo_root
from rustsheet_tools.progress_report import arguments, check, codebase, evidence, goals, history, page, parity
from rustsheet_tools.progress_report.model import (
    Parity,
    ProgressReportError,
    ReportShapeError,
    Run,
    compute,
    load_object,
    run_of,
    tasks_of,
)

PROG = "rustsheet-progress-report"


@dataclass(frozen=True)
class Stale:
    """Why a run is incomplete, and its run record when the report has a valid one."""

    reason: str
    run: Run | None


def read_report(path: Path) -> dict[str, Any] | Stale:
    """The parsed report, or why there is none. The reason is printed on the committed page, so it
    names the file but not where it lives."""
    try:
        text = path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return Stale(f"{path.name} is missing: the eval failed or produced no report", None)
    except (OSError, UnicodeDecodeError) as e:
        return Stale(f"{path.name} could not be read ({type(e).__name__}): the eval failed", None)
    try:
        report = json.loads(text)
    except ValueError:
        return Stale(f"{path.name} is not valid JSON: the eval failed or was cut short", None)
    if not isinstance(report, dict):
        return Stale(f"{path.name} is not a JSON object", None)
    return report


def _skipped_reason(run: Run) -> str:
    return "suites not run: " + ", ".join(f"{s.name} ({s.reason})" for s in run.skipped)


def main(argv: list[str] | None = None, git: history.Git | None = None) -> None:
    repo = repo_root()
    ap = argparse.ArgumentParser(prog=PROG, description=(__doc__ or "").splitlines()[0])
    ap.add_argument("--report", type=Path, default=repo / "eval" / "runs" / "latest" / "report.json")
    ap.add_argument("--scope", type=Path, default=repo / "eval" / "catalog" / "scope.json")
    ap.add_argument("--catalog", type=Path, default=repo / "eval" / "catalog" / "functions.yaml")
    ap.add_argument("--baselines", type=Path, default=repo / "eval" / "baseline", help="eval/baseline/")
    ap.add_argument("--tasks", type=Path, default=repo / "eval" / "runs" / "latest" / "tasks.json")
    ap.add_argument("--milestones", type=Path, default=repo / goals.MILESTONES_MD)
    ap.add_argument(
        "--out-dir", type=Path, help="the docs directory to write into (or, with --check, to read)"
    )
    ap.add_argument("--check", action="store_true", help="check the judgment blocks of the existing page")
    ap.add_argument(
        "--today",
        default=dt.datetime.now(dt.UTC).date().isoformat(),
        help="the UTC date of an incomplete run whose report has no run record (default: today)",
    )
    args = ap.parse_args(argv)
    if args.check:
        # The default docs dir is named by the page's own path because "docs/progress.md" is in
        # CODE_READ_PATHS (scripts/ci_changes.py, decision 0049) and a bare "docs" is not: keep it so,
        # or scripts/check_docs_readers.py fails `make hygiene`.
        run_check(args.out_dir or (repo / "docs/progress.md").parent)
        return
    if args.out_dir is None:
        ap.error("--out-dir is required unless --check is given")
    git = git or history.RepoGit(repo)
    history_path = args.out_dir / parity.ASSETS / history.HISTORY
    try:
        rows = history.load(history_path)
        previous = page.read_existing(args.out_dir)
        report = read_report(args.report)
        outcome = report if isinstance(report, Stale) else _evaluate(report, args)
        result = (
            _stale_outcome(outcome, rows, previous, git.head(), args.today)
            if isinstance(outcome, Stale)
            else _complete_outcome(
                *outcome[:3],
                rows,
                previous,
                coverage=outcome[3],
                git=git,
                repo=repo,
                tasks=load_object(args.tasks),
                milestones=goals.parse_milestones(_read_text(args.milestones)),
            )
        )
    except ProgressReportError as e:
        sys.exit(f"{PROG}: {e}")
    written = [*parity.write(result.files, args.out_dir), page.write(args.out_dir, result.page)]
    written.append(history_path)
    history_path.parent.mkdir(parents=True, exist_ok=True)
    history_path.write_bytes(history.dump(result.rows).encode("utf-8"))
    for line in [*(f"wrote {path}" for path in written), *result.summary]:
        print(line)


@dataclass(frozen=True)
class Result:
    """What a run writes: the page, the files under progress/, the history rows, and the lines it
    prints."""

    page: str
    files: dict[str, str]
    rows: list[history.Row]
    summary: list[str]


def _stale_outcome(
    stale: Stale, rows: list[history.Row], previous: str | None, head: str | None, today: str
) -> Result:
    """An incomplete run: only the status block is rewritten, and a `complete: false` row added."""
    run = stale.run
    # Without a run record, the row is dated today and names the generating commit.
    date, commit = (run.date, run.commit or head) if run else (today, head)
    summary = [f"stale: {stale.reason}; only the status block was rewritten"]
    if rows and date < rows[-1]["date"]:
        # An old report left behind (its run predates the last row): the page is still marked
        # stale, but the row is not recorded, since older rows are never rewritten.
        summary.append(
            f"not recorded in history: the run's date {date} is before the last row, {rows[-1]['date']}"
        )
    else:
        rows = history.add(rows, history.incomplete_row(date, commit, run.skipped if run else ()))
    # The page keeps the numbers of the latest complete row, which may be today's own.
    last = history.latest_complete(rows)
    status = page.stale_status(stale.reason, date, commit, last, run, head=head)
    return Result(page.render_stale(previous, status), {}, rows, summary)


def _complete_outcome(
    run: Run,
    data: Parity,
    goal: goals.Goals,
    rows: list[history.Row],
    previous: str | None,
    *,
    coverage: arguments.Coverage,
    git: history.Git,
    repo: Path,
    tasks: dict[str, Any],
    milestones: list[goals.MilestoneDoc],
) -> Result:
    """A complete run: every generated block, the charts and functions.md, and its history row."""
    counts = tasks_of(tasks)
    # HEAD is read only when the report has no commit (see history.complete_row).
    counts_now = codebase.measure(repo)
    row = history.complete_row(run, data, counts, None if run.commit else git.head(), counts_now)
    rows = history.add(rows, row)
    last = history.last_complete(rows, row["date"])
    commits = (
        git.commits_between(last["commit"], row["commit"])
        if last and last["commit"] and row["commit"]
        else None
    )
    sections = {
        "goals": goals.goals_block(goal, milestones, data, counts, last),
        "campaign": goals.campaign_block(goal, data, counts),
        "since-last-run": history.since_block(row, last, commits),
        "trends": history.trends_block(parity.ASSETS),
        "arguments": arguments.block(coverage),
        "codebase": codebase.block(counts_now, last),
    }
    summary = [
        f"{len(data.functions)} functions: " + ", ".join(f"{t.tier} {t.functions}" for t in data.tiers),
        parity.phase1_lines(data).splitlines()[0],
    ]
    files = {**parity.files(data, run), **evidence.files(data), **history.charts(rows)}
    return Result(page.render(run, data, previous, sections), files, rows, summary)


def _evaluate(
    report: dict[str, Any], args: argparse.Namespace
) -> Stale | tuple[Run, Parity, goals.Goals, arguments.Coverage]:
    """The run, its parity data and its goals fields, or why the run is incomplete: an older
    report shape, or a suite it skipped. A malformed report or a catalog that is not the run's
    still fails."""
    try:
        run = run_of(report)
    except ReportShapeError as e:
        return Stale(f"an older report shape ({e})", None)
    try:
        data = compute(report, load_object(args.scope), args.baselines, evidence.load_priors(args.catalog))
        goal = goals.load(report)
        coverage = arguments.load(report)
    except ReportShapeError as e:
        return Stale(f"an older report shape ({e})", run)
    if run.skipped:
        return Stale(_skipped_reason(run), run)
    return run, data, goal, coverage


def _read_text(path: Path) -> str:
    """`path` as UTF-8 text; a missing or unreadable file is a `ProgressReportError`."""
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as e:
        msg = f"{path}: {e}"
        raise ProgressReportError(msg) from e


def run_check(docs: Path) -> None:
    """`--check`: print `progress check ok`, or every offender and exit 1 (SPEC-0022, T-0074)."""
    try:
        existing = page.read_existing(docs)
        if existing is None:
            msg = f"{docs / page.PAGE}: no page to check"
            raise ProgressReportError(msg)
        found = check.offenders(existing)
    except ProgressReportError as e:
        sys.exit(f"{PROG} --check: {e}")
    if found:
        sys.exit("\n".join([f"{PROG} --check: {len(found)} offender(s) in {docs / page.PAGE}:", *found]))
    print(check.OK)


if __name__ == "__main__":
    main()
