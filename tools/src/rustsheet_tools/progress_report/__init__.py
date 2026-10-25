"""Generate the daily progress page, docs/progress.md, from the eval report (SPEC-0022).

Reads `eval/runs/latest/report.json` (written by `make eval`), `eval/catalog/scope.json`, the
eval baselines, `eval/runs/latest/tasks.json` (`scripts/tasks.py stats --json`),
docs/milestones.md, git, and the existing page and run history under `--out-dir` (the
repository's docs directory; `make progress-report` passes it). A complete run writes `progress.md`, keeping the steward's judgment
blocks byte for byte, and `progress/` (the all-functions page, the charts and history.json). An
incomplete run (no report, an older report shape, or a skipped suite) rewrites only the page's
status block as stale, adds a `complete: false` history row and exits 0. Every input is checked
and every file rendered before anything is written, so a failed run leaves the pages as they were.
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
from rustsheet_tools.progress_report import goals, history, page, parity
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
    ap.add_argument("--baselines", type=Path, default=repo / "eval" / "baseline", help="eval/baseline/")
    ap.add_argument("--tasks", type=Path, default=repo / "eval" / "runs" / "latest" / "tasks.json")
    ap.add_argument("--milestones", type=Path, default=repo / goals.MILESTONES_MD)
    ap.add_argument("--out-dir", type=Path, required=True, help="the docs directory to write into")
    ap.add_argument(
        "--today",
        default=dt.datetime.now(dt.UTC).date().isoformat(),
        help="the UTC date of an incomplete run whose report has no run record (default: today)",
    )
    args = ap.parse_args(argv)
    git = git or history.RepoGit(repo)
    history_path = args.out_dir / parity.ASSETS / history.HISTORY
    try:
        rows = history.load(history_path)
        previous = page.read_existing(args.out_dir)
        head = git.head()
        report = read_report(args.report)
        outcome: Stale | tuple[Run, Parity, goals.Goals] = (
            report if isinstance(report, Stale) else _evaluate(report, args)
        )
        if isinstance(outcome, Stale):
            run = outcome.run
            date, commit = (run.date, run.commit or head) if run else (args.today, head)
            row = history.incomplete_row(date, commit, run.skipped if run else ())
            rows = history.add(rows, row)
            # The page keeps the numbers of the latest complete row, which may be today's own.
            last = history.latest_complete(rows)
            progress = page.render_stale(previous, page.stale_status(outcome.reason, date, commit, last, run))
            files: dict[str, str] = {}
            summary = [f"stale: {outcome.reason}; only the status block was rewritten"]
        else:
            run, data, goal = outcome
            tasks = tasks_of(load_object(args.tasks))
            milestones = goals.parse_milestones(_read_text(args.milestones))
            row = history.complete_row(run, data, tasks, head)
            rows = history.add(rows, row)
            last = history.last_complete(rows, row["date"])
            commits = git.commits_since(last["commit"]) if last and last["commit"] else None
            sections = {
                "goals": goals.goals_block(goal, milestones, data, tasks, last),
                "campaign": goals.campaign_block(goal, data, tasks),
                "since-last-run": history.since_block(row, last, commits),
                "trends": history.trends_block(parity.ASSETS),
            }
            progress = page.render(run, data, previous, sections, head)
            files = {**parity.files(data, run), **history.charts(rows)}
            summary = [
                f"{len(data.functions)} functions: "
                + ", ".join(f"{t.tier} {t.functions}" for t in data.tiers),
                parity.phase1_lines(data).splitlines()[0],
            ]
    except ProgressReportError as e:
        sys.exit(f"{PROG}: {e}")
    written = [*parity.write(files, args.out_dir), page.write(args.out_dir, progress)]
    written.append(history_path)
    history_path.parent.mkdir(parents=True, exist_ok=True)
    history_path.write_bytes(history.dump(rows).encode("utf-8"))
    for line in [*(f"wrote {path}" for path in written), *summary]:
        print(line)


def _evaluate(report: dict[str, Any], args: argparse.Namespace) -> Stale | tuple[Run, Parity, goals.Goals]:
    """The run, its parity data and its goals fields, or why the run is incomplete: an older
    report shape, or a suite it skipped. A malformed report or a catalog that is not the run's
    still fails."""
    try:
        run = run_of(report)
    except ReportShapeError as e:
        return Stale(f"an older report shape ({e})", None)
    try:
        data = compute(report, load_object(args.scope), args.baselines)
        goal = goals.load(report)
    except ReportShapeError as e:
        return Stale(f"an older report shape ({e})", run)
    if run.skipped:
        return Stale(_skipped_reason(run), run)
    return run, data, goal


def _read_text(path: Path) -> str:
    """`path` as UTF-8 text; a missing or unreadable file is a `ProgressReportError`."""
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as e:
        msg = f"{path}: {e}"
        raise ProgressReportError(msg) from e


if __name__ == "__main__":
    main()
