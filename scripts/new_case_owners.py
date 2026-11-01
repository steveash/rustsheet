#!/usr/bin/env python3
"""Gate: every visibly-off Excel cell of an eval case the branch adds or refills has an open owner.

T-3793, decision 0245. A case the branch adds, or whose `.case.json` / workbook / `.oracle.json`
sidecar it changes (a refill counts), is new. Each new case is run alone
(`rustsheet-eval run --suite S --filter F --out <tmp>`, under a second), and its visibly-off cells
(decision 0188 R1(c): `exit_owners.all_off_cells`) must each be owned by an open task by
`exit_owners`' rule; a cell owned only by a `defer:` task passes. A branch with no new case runs
nothing, so `main` always passes.

Usage: new_case_owners.py [--base origin/main] [--eval target/release/rustsheet-eval]
Exit 0 when every cell has an owner, 1 when one has none (each listed as
`case!Sheet!A1<TAB>formula<TAB>expected<TAB>actual`), 2 when the check cannot run.
Standard library only.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import exit_owners  # noqa: E402 (the sys.path line above must run first)
import tasks  # noqa: E402

ROOT = exit_owners.ROOT
SUITES = "eval/suites"
# The case files a suite may hold (`rustsheet-eval`'s `SuiteKind`) and their oracle sidecar.
CASE_SUFFIXES = (".case.json", ".xlsx", ".xlsm")
SIDECAR = ".oracle.json"


class GateError(Exception):
    """The check cannot run (no merge base, a failed git or eval command)."""


def git(root: Path, *args: str) -> str:
    try:
        return subprocess.run(["git", *args], cwd=root, check=True, capture_output=True, text=True).stdout
    except (OSError, subprocess.CalledProcessError) as e:
        raise GateError(f"git {' '.join(args)} failed: {e}") from e


def changed_paths(root: Path, base: str) -> list[str]:
    """Paths under eval/suites the branch adds or modifies against its merge base with `base`,
    the working tree and untracked files included (`make check` runs before the commit)."""
    merge_base = git(root, "merge-base", base, "HEAD").strip()
    diff = git(
        root, "diff", "--no-renames", "--diff-filter=AM", "--name-only", "-z", merge_base, "--", SUITES
    )
    untracked = git(root, "ls-files", "--others", "--exclude-standard", "-z", "--", SUITES)
    return sorted({p for p in (diff + untracked).split("\0") if p})


def suite_root(root: Path, suite: str) -> str | None:
    """The suite's case root below its directory (`suite.toml`'s `root`), or None for a vendored
    suite (`fetch`: its cases are not in the repository) or a directory that is no suite."""
    manifest = root / SUITES / suite / "suite.toml"
    if not manifest.is_file():
        return None
    meta = tomllib.loads(manifest.read_text())
    if "fetch" in meta:
        return None
    return str(meta.get("root", "."))


def case_file(path: Path) -> Path | None:
    """The case a changed file belongs to: itself, or the case beside an oracle sidecar."""
    name = path.name
    if name.endswith(CASE_SUFFIXES):
        return path
    if name.endswith(SIDECAR):
        stem = name.removesuffix(SIDECAR)
        for suffix in CASE_SUFFIXES:
            case = path.with_name(stem + suffix)
            if case.is_file():
                return case
    return None


def new_cases(root: Path, paths: list[str]) -> dict[str, set[str]]:
    """Suite → the ids (`suite/relative/path`) of the cases `paths` add or refill."""
    out: dict[str, set[str]] = {}
    for p in paths:
        parts = Path(p).parts
        if len(parts) < 4 or parts[:2] != ("eval", "suites"):
            continue
        suite = parts[2]
        case_root = suite_root(root, suite)
        if case_root is None:
            continue
        base = (root / SUITES / suite / case_root).resolve()
        case = case_file(root / p)
        if case is None or not case.is_file():
            continue
        try:
            rel = case.resolve().relative_to(base)
        except ValueError:
            continue
        out.setdefault(suite, set()).add(f"{suite}/{rel.as_posix()}")
    return out


def all_case_ids(root: Path) -> list[str]:
    """Every case id of the repository's own suites, so a one-case run names its case the way the
    full report does (`exit_owners.case_names` depends on the other cases)."""
    ids = []
    for manifest in sorted((root / SUITES).glob("*/suite.toml")):
        suite = manifest.parent.name
        case_root = suite_root(root, suite)
        if case_root is None:
            continue
        base = manifest.parent / case_root
        for f in base.rglob("*"):
            if f.name.endswith(CASE_SUFFIXES) and f.is_file():
                ids.append(f"{suite}/{f.relative_to(base).as_posix()}")
    return ids


def run_case(eval_bin: Path, root: Path, case_id: str, out: Path) -> dict[str, object]:
    """The report of a run of `case_id` alone."""
    suite, _, rel = case_id.partition("/")
    cmd = [str(eval_bin), "run", "--suite", suite, "--filter", rel, "--out", str(out)]
    try:
        subprocess.run(cmd, cwd=root, check=True, capture_output=True, text=True)
        report: dict[str, object] = json.loads((out / "report.json").read_text())
    except (OSError, ValueError, subprocess.CalledProcessError) as e:
        raise GateError(f"{' '.join(cmd)} failed: {e}") from e
    return report


def unowned_new(
    report: dict[str, object], tasks_dir: Path, new: set[str], names: dict[str, tuple[str, ...]]
) -> list[exit_owners.OffCell]:
    """The visibly-off cells of the `new` cases in `report` that no open task owns (a cell owned
    by a deferred task only is owned)."""
    return [
        o
        for o, deferred in exit_owners.unowned_off(report, tasks_dir, names)
        if o.cell.case in new and not deferred
    ]


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--base", default="origin/main")
    ap.add_argument("--eval", type=Path, default=ROOT / "target" / "release" / "rustsheet-eval")
    ap.add_argument("--tasks", type=Path, default=tasks.TASKS)
    args = ap.parse_args(argv)
    try:
        by_suite = new_cases(ROOT, changed_paths(ROOT, args.base))
        if not by_suite:
            print("new-case owners: no eval case added or refilled")
            return 0
        names = exit_owners.case_names(all_case_ids(ROOT))
        found: list[exit_owners.OffCell] = []
        with tempfile.TemporaryDirectory() as tmp:
            ids = sorted(case for cases in by_suite.values() for case in cases)
            for n, case_id in enumerate(ids):
                report = run_case(args.eval, ROOT, case_id, Path(tmp) / str(n))
                found += unowned_new(report, args.tasks, {case_id}, names)
    except GateError as e:
        print(f"error: {e}", file=sys.stderr)
        return 2
    count = sum(len(v) for v in by_suite.values())
    for o in found:
        print(f"{o.cell.case}!{o.cell.key}\t{o.cell.formula}\t{o.expected}\t{o.actual}")
    if found:
        print(
            f"new-case owners: {len(found)} visibly-off cells of the {count} new or refilled cases have no open"
            " owner; file or amend a task that names them (decision 0245)"
        )
        return 1
    print(f"new-case owners: every visibly-off cell of the {count} new or refilled cases has an owner")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
