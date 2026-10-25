"""Generate the daily progress page, docs/progress.md, from the eval report (SPEC-0022).

Reads `eval/runs/latest/report.json` (written by `make eval`), `eval/catalog/scope.json` and the
existing page under `--out-dir` (the repository's docs directory; `make progress-report` passes
it). Writes `progress.md`, keeping the steward's judgment blocks byte for byte, and, until T-0081
replaces it, the per-function parity page (`parity.md`, `parity/`). Every input is checked before
anything is written, so a failed run leaves the pages as they were.

With `--check` it writes nothing: it checks the judgment blocks of the existing page (check.py,
T-0074) and exits 1, listing every offender, when one fails.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from rustsheet_tools import repo_root
from rustsheet_tools.progress_report import check, page, parity
from rustsheet_tools.progress_report.model import BUCKETS, ProgressReportError, compute, load_object, run_of

PROG = "rustsheet-progress-report"


def main(argv: list[str] | None = None) -> None:
    repo = repo_root()
    ap = argparse.ArgumentParser(prog=PROG, description=(__doc__ or "").splitlines()[0])
    ap.add_argument("--report", type=Path, default=repo / "eval" / "runs" / "latest" / "report.json")
    ap.add_argument("--scope", type=Path, default=repo / "eval" / "catalog" / "scope.json")
    ap.add_argument(
        "--out-dir", type=Path, help="the docs directory to write into (or, with --check, to read)"
    )
    ap.add_argument("--check", action="store_true", help="check the judgment blocks of the existing page")
    args = ap.parse_args(argv)
    if args.check:
        # The page's own path (a CODE_READ_PATHS entry, decision 0049) names the default docs dir.
        run_check(args.out_dir or (repo / "docs/progress.md").parent)
        return
    if args.out_dir is None:
        ap.error("--out-dir is required unless --check is given")
    try:
        report = load_object(args.report)
        parity_data = compute(report, load_object(args.scope))
        run = run_of(report)
        progress = page.render(run, page.read_existing(args.out_dir))
    except (KeyError, TypeError, ProgressReportError) as e:
        sys.exit(f"{PROG}: {e}")
    written = [*parity.write(parity_data, args.out_dir, run.date, run.commit or "unknown")]
    written.append(page.write(args.out_dir, progress))
    for path in written:
        print(f"wrote {path}")
    fb, cb = parity_data.functions_per_bucket(), parity_data.cells_per_bucket()
    print(f"{len(parity_data.functions)} functions: " + ", ".join(f"{b} {fb[b]}" for b in BUCKETS))
    print("cells: " + ", ".join(f"{b} {cb[b]:,}" for b in BUCKETS))


def run_check(docs: Path) -> None:
    """`--check`: print `progress check ok`, or every offender and exit 1 (SPEC-0022, T-0074)."""
    try:
        existing = page.read_existing(docs)
        if existing is None:
            msg = f"{docs / page.PAGE}: no page to check"
            raise ProgressReportError(msg)
        found = check.offenders(existing, page.GENERATED_BLOCKS)
    except ProgressReportError as e:
        sys.exit(f"{PROG} --check: {e}")
    if found:
        sys.exit("\n".join([f"{PROG} --check: {len(found)} offender(s) in {docs / page.PAGE}:", *found]))
    print(check.OK)


if __name__ == "__main__":
    main()
