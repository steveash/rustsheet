"""`rustsheet-accuracy`: Gate A reference tables (SPEC-0029).

  rustsheet-accuracy list                                  functions with a definition
  rustsheet-accuracy generate T.INV [--batch s1]           write a batch's case and math sidecar
  rustsheet-accuracy generate RATE --batch e-T-1234 --count 40 --stratum tail
  rustsheet-accuracy verify T.INV                          recompute committed math at dps 80
  rustsheet-accuracy math --suite probes-numeric           math sidecars for an eval suite's cells
  rustsheet-accuracy math --check --suite probes            fail when a suite's sidecars are missing or stale

Then `uv run rustsheet-oracle fill -b excel-graph --suite accuracy` fills Excel's values, and
`rustsheet-eval accuracy` (run by `make check`) checks the engine against both.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from rustsheet_tools import repo_root
from rustsheet_tools.accuracy import defs
from rustsheet_tools.accuracy.eval_math import check_suite, write_suite
from rustsheet_tools.accuracy.model import REGISTRY, Definition
from rustsheet_tools.accuracy.sampler import STRATA
from rustsheet_tools.accuracy.tables import draw_points, verify_batch, write_batch

_ = defs  # importing the package registers the definitions


def _definition(name: str) -> Definition:
    found = REGISTRY.get(name.upper())
    if found is None:
        msg = f"no Gate A definition for {name} (have: {', '.join(sorted(REGISTRY))})"
        raise SystemExit(msg)
    return found


def _math(suites: list[str], *, check: bool, workers: int | None) -> int:
    """`math`: write one suite's sidecars, or (`--check`) report the suites' missing or stale ones."""
    if check:
        stale = [line for suite in suites for line in check_suite(suite)]
        for line in stale:
            print(line)
        print(f"math sidecars: {', '.join(suites)}: {'stale' if stale else 'current'}")
        return 1 if stale else 0
    if len(suites) != 1:
        raise SystemExit("math writes one suite at a time")
    (suite,) = suites
    results = write_suite(suite, workers)
    skipped: dict[str, int] = {}
    for math in results:
        for reason, n in math.skipped.items():
            skipped[reason] = skipped.get(reason, 0) + n
    cases = sum(1 for m in results if m.written)
    print(f"{suite}: {sum(m.written for m in results)} math cells in {cases} cases")
    for reason, n in sorted(skipped.items()):
        print(f"  skipped {n}: {reason}")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="rustsheet-accuracy", description=__doc__.split("\n")[0])
    parser.add_argument("--tables", type=Path, help="tables directory (default eval/accuracy/tables)")
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("list", help="functions with a definition")
    gen = sub.add_parser("generate", help="write one batch of a function's table")
    gen.add_argument("function")
    gen.add_argument("--batch", default="s1", help="batch name: s1, r4-T-XXXX, e-T-XXXX (default s1)")
    gen.add_argument("--count", type=int, help="points (default 120/200/300 by arity)")
    gen.add_argument("--stratum", choices=STRATA, help="draw every point from one stratum")
    ver = sub.add_parser("verify", help="recompute a function's committed math values")
    ver.add_argument("function")
    mth = sub.add_parser("math", help="write math sidecars for an eval suite's one-call cells (T-3380)")
    mth.add_argument("--suite", required=True, action="append", help="suite (repeatable with --check)")
    mth.add_argument("--check", action="store_true", help="check the sidecars without mpmath (T-3588)")
    mth.add_argument("--workers", type=int, help="processes (default: one per CPU)")
    args = parser.parse_args(argv)
    tables: Path = args.tables or repo_root() / "eval" / "accuracy" / "tables"

    if args.command == "list":
        for name in sorted(REGISTRY):
            print(name)
        return 0
    if args.command == "math":
        return _math(args.suite, check=args.check, workers=args.workers)
    definition = _definition(args.function)
    if args.command == "generate":
        if args.count is not None and args.count < 1:
            raise SystemExit("--count must be at least 1")
        points = draw_points(definition, args.batch, args.count, stratum=args.stratum)
        case_path, math_path = write_batch(tables, definition, args.batch, points)
        domain = sum(1 for p in points if p.math.value is None)
        print(f"{definition.function}/{args.batch}: {len(points)} points ({domain} domain errors)")
        print(f"  {case_path}\n  {math_path}")
        return 0
    folder = tables / definition.function
    batches = sorted(folder.glob("*.math.json"))
    if not batches:
        print(f"no batches under {folder}", file=sys.stderr)
        return 1
    problems: list[str] = []
    for math_path in batches:
        case_path = math_path.with_name(math_path.name.removesuffix(".math.json") + ".case.json")
        problems += verify_batch(definition, case_path, math_path)
    for line in problems:
        print(line)
    print(f"{definition.function}: {len(batches)} batches, {len(problems)} differences")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
