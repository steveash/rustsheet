"""Generate the daily progress page, docs/progress.md, from the eval report (SPEC-0022).

Reads `eval/runs/latest/report.json` (written by `make eval`), `eval/catalog/scope.json`, the
eval baselines and the existing page under `--out-dir` (the repository's docs directory; `make
progress-report` passes it). Writes `progress.md`, keeping the steward's judgment blocks byte for
byte, and `progress/` (the all-functions page and the charts). Every input is checked and every
file rendered before anything is written, so a failed run leaves the pages as they were.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from rustsheet_tools import repo_root
from rustsheet_tools.progress_report import evidence, page, parity
from rustsheet_tools.progress_report.model import ProgressReportError, compute, load_object, run_of

PROG = "rustsheet-progress-report"


def main(argv: list[str] | None = None) -> None:
    repo = repo_root()
    ap = argparse.ArgumentParser(prog=PROG, description=(__doc__ or "").splitlines()[0])
    ap.add_argument("--report", type=Path, default=repo / "eval" / "runs" / "latest" / "report.json")
    ap.add_argument("--scope", type=Path, default=repo / "eval" / "catalog" / "scope.json")
    ap.add_argument("--catalog", type=Path, default=repo / "eval" / "catalog" / "functions.yaml")
    ap.add_argument("--baselines", type=Path, default=repo / "eval" / "baseline", help="eval/baseline/")
    ap.add_argument("--out-dir", type=Path, required=True, help="the docs directory to write into")
    args = ap.parse_args(argv)
    try:
        report = load_object(args.report)
        data = compute(report, load_object(args.scope), args.baselines, evidence.load_priors(args.catalog))
        run = run_of(report)
        progress = page.render(run, data, page.read_existing(args.out_dir))
        assets = {**parity.files(data, run), **evidence.files(data)}
    except ProgressReportError as e:
        sys.exit(f"{PROG}: {e}")
    written = [*parity.write(assets, args.out_dir), page.write(args.out_dir, progress)]
    for path in written:
        print(f"wrote {path}")
    print(f"{len(data.functions)} functions: " + ", ".join(f"{t.tier} {t.functions}" for t in data.tiers))
    print(parity.phase1_lines(data).splitlines()[0])


if __name__ == "__main__":
    main()
