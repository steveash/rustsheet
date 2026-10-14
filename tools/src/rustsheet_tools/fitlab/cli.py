"""`rustsheet-fitlab`: rank candidate kernels against an oracle sweep (docs/architecture/fit-lab.md).

  rustsheet-fitlab --list                          the shipped fits
  rustsheet-fitlab erfc                            run a shipped fit (or `package.module:FIT`)
  rustsheet-fitlab erfc-tail --top 10 --worst 5    more or fewer rows and worst cells
  rustsheet-fitlab --case probes-numeric/t1040-erfc --pattern '^=ERFC\\((.*)\\)$' \\
      --candidate math:erfc                        ad hoc: any callables on any sweep

The JSON (every candidate's counts and worst cells, per-family unions) goes to
eval/runs/fitlab/<fit>.json unless --json names another path.
"""

from __future__ import annotations

import argparse
import importlib
import json
import sys
from pathlib import Path
from typing import Any

from rustsheet_tools import repo_root
from rustsheet_tools.fitlab.fit import ENGINE, Fit, FitResult, run
from rustsheet_tools.fitlab.kernels import registry
from rustsheet_tools.fitlab.score import (
    Candidate,
    candidate,
    families,
    family_table,
    rank,
    table,
    worst_text,
)


def _import(spec: str) -> Any:
    """`package.module:attr` → the attribute."""
    module, _, attr = spec.partition(":")
    if not attr:
        msg = f"{spec!r} is not of the form package.module:attr"
        raise SystemExit(msg)
    return getattr(importlib.import_module(module), attr)


def _fit(args: argparse.Namespace) -> Fit:
    if args.fit:
        fits = registry()
        if args.fit in fits:
            return fits[args.fit]
        found = _import(args.fit)
        if not isinstance(found, Fit):
            msg = f"{args.fit} is not a Fit"
            raise SystemExit(msg)
        return found
    if not (args.case and args.pattern and args.candidate):
        msg = "give a fit name, or --case, --pattern and at least one --candidate"
        raise SystemExit(msg)
    adhoc: list[Candidate] = [candidate(spec, _import(spec)) for spec in args.candidate]
    return Fit("adhoc", "ad hoc fit", args.case, args.pattern, lambda: adhoc)


def report(result: FitResult, top: int, worst: int) -> str:
    """The text report: sweep summary, ranked table, families, worst cells."""
    sweep, fit = result.sweep, result.fit
    lines = [
        f"fit {fit.name}: {fit.description}",
        f"{len(sweep.cells)} cells from {', '.join(fit.cases)}; {sweep.excluded} excluded by the fit's "
        f"filter, {sweep.non_numeric} with a non-numeric Excel value, {len(sweep.unparsed)} unparsed",
        "",
        table(result.scores, top),
    ]
    if len({s.family for s in result.scores}) < len(result.scores):
        lines += ["", family_table(families(result.scores), len(sweep.cells))]
    ranked = rank(result.scores)
    shown = [s for s in ranked if s.name == ENGINE] + [s for s in ranked if s.name != ENGINE][:3]
    lines += ["", *(worst_text(s, worst) for s in shown if s.exact < s.total)]
    if sweep.unparsed:
        lines += ["", "unparsed:", *(f"  {u}" for u in sweep.unparsed[:10])]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="rustsheet-fitlab", description=__doc__.split("\n")[0])
    parser.add_argument("fit", nargs="?", help="shipped fit name, or package.module:FIT")
    parser.add_argument("--list", action="store_true", help="list the shipped fits")
    parser.add_argument("--case", action="append", help="ad hoc: suite/case (repeatable)")
    parser.add_argument("--pattern", help="ad hoc: formula regex, group 1 = the argument list")
    parser.add_argument("--candidate", action="append", help="ad hoc: package.module:callable (repeatable)")
    parser.add_argument("--only", help="score only candidates whose name matches this regex")
    parser.add_argument("--no-engine", action="store_true", help="skip the engine pseudo-candidate")
    parser.add_argument("--top", type=int, default=20, help="rows of the ranked table (default 20)")
    parser.add_argument("--worst", type=int, default=5, help="worst cells per candidate (default 5)")
    parser.add_argument("--json", type=Path, help="JSON output (default eval/runs/fitlab/<fit>.json)")
    args = parser.parse_args(argv)
    if args.list:
        for name, fit in registry().items():
            print(f"{name:<16} {fit.description}")
        return 0
    fit = _fit(args)
    result = run(fit, engine=not args.no_engine, only=args.only, keep=max(args.worst, 1))
    if not result.sweep.cells:
        print(f"fit {fit.name}: no cell matched {fit.pattern!r}", file=sys.stderr)
        return 1
    print(report(result, args.top, args.worst))
    out = args.json or repo_root() / "eval" / "runs" / "fitlab" / f"{fit.name}.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result.to_json(args.worst), indent=1) + "\n", encoding="utf-8")
    print(f"\nJSON: {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
