#!/usr/bin/env python3
"""Governance gate for autonomous development.

Some paths define the rules agents work under (CI, lint policy, baselines, agent
instructions). A PR that changes them must also add or update a decision record in
docs/decisions/ explaining why, so rules are never silently weakened. Additionally,
`--allow-regressions` baseline changes and test deletions are flagged, and a newly added probes
case must carry exactly one milestone tag, M2 or later (decisions 0043, 0051, 0057). A diff that
adds or changes a task filed from a GitHub issue (`issue:` in its front matter) may not touch a
protected path at all, decision record or not, nor a dependency manifest, lockfile or build
script, nor add `allow(` or `unsafe` to Rust code (decision 0158).

Usage: scripts/check_governance.py [BASE_REF]   (default: origin/main)
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from collections.abc import Callable, Iterable
from pathlib import Path

import merge_flip
import tasks

PROTECTED = (
    ".github/",
    ".claude/",
    "CLAUDE.md",
    "Cargo.toml",  # workspace lint policy
    "clippy.toml",
    "deny.toml",
    "rustfmt.toml",
    "Makefile",
    "scripts/",
    "quality/",
    "docs/process/",
    "eval/milestones/",  # frozen milestone probe lists (decisions 0043, 0044)
)


def is_deviations_file(path: str) -> bool:
    """A suite's accepted-deviations list (decision 0036): each line hides a cell from bit-exact."""
    return path.startswith("eval/suites/") and path.endswith("/deviations")


def is_protected(path: str) -> bool:
    """Whether changing `path` needs a decision record in the same PR."""
    return path.startswith(PROTECTED) or path in PROTECTED or is_deviations_file(path)


PROBE_CASES = "eval/suites/probes/cases/"
# A milestone tag, as `parse_milestone` in crates/rustsheet-eval/src/probe_milestones.rs reads one.
MILESTONE_TAG = re.compile(r"^M\d+$")


def probe_tag_problems(
    name_status_lines: Iterable[str], read_tags: Callable[[str], list[str]]
) -> list[str]:
    """Probe cases the diff adds whose milestone tags break decision 0057: one per offender.

    A case added under eval/suites/probes/cases/ (outside catalog/, whose milestones decision 0051
    assigns per cell) needs exactly one milestone tag, and not M0 or M1: a post-freeze probe of an
    M0-M2 subject counts toward M2 anyway (decisions 0043, 0051), so a stale M0/M1 tag only hides
    a later-milestone subject on M2's exit row. Modified, renamed and deleted cases are not checked
    (their tags are never edited, decision 0043). `name_status_lines` are `git diff --name-status`
    lines; `read_tags(path)` gives the `tags` of the added case.
    """
    problems = []
    for line in name_status_lines:
        fields = line.split("\t")
        if len(fields) != 2 or fields[0] != "A":
            continue
        path = fields[1]
        if not (path.startswith(PROBE_CASES) and path.endswith(".case.json")):
            continue
        if path.startswith(PROBE_CASES + "catalog/"):
            continue
        milestones = [tag for tag in read_tags(path) if MILESTONE_TAG.match(tag)]
        if len(milestones) != 1 or milestones[0] in ("M0", "M1"):
            shown = ", ".join(milestones) if milestones else "none"
            problems.append(f"{path} (milestone tags: {shown})")
    return problems


def git(*args: str) -> str:
    return subprocess.run(["git", *args], capture_output=True, text=True, check=True).stdout


def case_tags(path: str) -> list[str]:
    """The `tags` of the case at `path` as committed on HEAD; none if it is not valid JSON."""
    try:
        tags = json.loads(git("show", f"HEAD:{path}")).get("tags", [])
    except (subprocess.CalledProcessError, json.JSONDecodeError, AttributeError):
        return []
    return [tag for tag in tags if isinstance(tag, str)] if isinstance(tags, list) else []


def task_has_issue(path: str) -> bool:
    """Whether the task file at `path`, as committed on HEAD, has `issue:` in its front matter."""
    try:
        return "issue" in tasks.front_matter(git("show", f"HEAD:{path}"), Path(path))
    except (subprocess.CalledProcessError, ValueError):
        return False


def issue_task_files(changed: list[str], has_issue: Callable[[str], bool]) -> list[str]:
    """Task files a diff's `--name-status` lines add or change (not delete) that came from a GitHub
    issue (decision 0158)."""
    return [
        path
        for line in changed
        if not line.startswith("D")
        and (path := line.split("\t")[-1]).startswith("docs/tasks/")
        and tasks.TASK_FILE.search(path)
        and has_issue(path)
    ]


# Supply-chain and lint surface an issue task may not touch (decision 0158), on top of
# `is_protected`. Ordinary PRs are unaffected: these apply only to a diff with an `issue:` task.
ISSUE_FORBIDDEN_NAMES = ("Cargo.lock", "uv.lock", "Cargo.toml", "build.rs", "pyproject.toml")
ISSUE_FORBIDDEN_DIRS = (".cargo/",)
# Added Rust lines an issue task may not contain: lint allows (`#[allow(`, `#![allow(`) and `unsafe`.
ISSUE_FORBIDDEN_RUST = re.compile(r"allow\(|\bunsafe\b")


def issue_forbidden_path(path: str) -> bool:
    """A dependency manifest, lockfile, build script or cargo config, at any depth."""
    return path.rsplit("/", 1)[-1] in ISSUE_FORBIDDEN_NAMES or path.startswith(ISSUE_FORBIDDEN_DIRS)


def added_rust_lines(unified_diff: str) -> list[tuple[str, str]]:
    """(path, text) of each line a `git diff -U0` adds to a `.rs` file."""
    added, path = [], ""
    for line in unified_diff.splitlines():
        if line.startswith("+++ "):
            target = line[4:]
            path = target[2:] if target.startswith("b/") else ""
        elif line.startswith("+") and path.endswith(".rs"):
            added.append((path, line[1:]))
    return added


def issue_task_problems(changed: list[str], added: Callable[[], list[tuple[str, str]]]) -> list[str]:
    """What an issue task's diff may not do beyond `is_protected` paths (decision 0158): touch a
    manifest, lockfile or build script (both sides of a rename count), or add a lint allow or
    `unsafe` to a `.rs` file. `added()` gives the diff's added Rust lines."""
    paths = {p for line in changed for p in line.split("\t")[1:]}
    problems = [f"  {p} (manifest, lockfile or build script)" for p in sorted(paths) if issue_forbidden_path(p)]
    problems += [
        f"  {path}: +{text.strip()} (lint allow or unsafe)"
        for path, text in added()
        if ISSUE_FORBIDDEN_RUST.search(text)
    ]
    return problems


def deleted_test_files(changed: list[str]) -> list[str]:
    """The test files a diff's `--name-status` lines delete."""
    return [
        line.split("\t")[-1]
        for line in changed
        if line.startswith("D") and ("/tests/" in line or "/test/" in line or line.endswith("_test.py"))
    ]


def failures(
    changed: list[str],
    read_tags: Callable[[str], list[str]],
    flip: bool,
    has_issue: Callable[[str], bool] = task_has_issue,
    added: Callable[[], list[tuple[str, str]]] = list,
) -> list[list[str]]:
    """The failed rules for a diff's `--name-status` lines, each as its report lines. `flip` says
    the diff is a lock flip (decision 0157): it waives the decision-record rule and nothing else.
    `has_issue(path)` says whether a task file on HEAD came from a GitHub issue, and `added()`
    gives the diff's added Rust lines, read only for such a diff (decision 0158)."""
    paths = [line.split("\t")[-1] for line in changed]
    protected = [p for p in paths if is_protected(p)]
    decisions = [p for p in paths if p.startswith("docs/decisions/") and p.endswith(".md")]
    found = []
    if protected and not decisions and not flip:
        found.append(
            [
                "FAIL governance: protected paths changed without a decision record in docs/decisions/:",
                *(f"  {p}" for p in protected),
            ]
        )
    from_issue = issue_task_files(changed, has_issue)
    if from_issue and protected:
        found.append(
            [
                "FAIL governance: a task filed from a GitHub issue may not change protected paths, "
                "even with a decision record (decision 0158). Label the issue `needs-owner` and leave "
                "the protected change to an owner-directed task. Issue tasks in this diff:",
                *(f"  {p}" for p in from_issue),
                "Protected paths changed:",
                *(f"  {p}" for p in protected),
            ]
        )
    supply = issue_task_problems(changed, added) if from_issue else []
    if supply:
        found.append(
            [
                "FAIL governance: a task filed from a GitHub issue may not touch dependency manifests, "
                "lockfiles or build scripts, or add lint allows or `unsafe` (decision 0158). Label the "
                "issue `needs-owner`. Issue tasks in this diff:",
                *(f"  {p}" for p in from_issue),
                "Not allowed:",
                *supply,
            ]
        )
    deleted_tests = deleted_test_files(changed)
    if deleted_tests and not decisions:
        found.append(
            [
                "FAIL governance: test files deleted (requires a decision record and reviewer sign-off):",
                *(f"  {p}" for p in deleted_tests),
            ]
        )
    probe_problems = probe_tag_problems(changed, read_tags)
    if probe_problems:
        found.append(
            [
                "FAIL governance: new probe cases need exactly one milestone tag, M2 or later, the "
                "milestone of the function or feature they probe (decision 0057):",
                *(f"  {problem}" for problem in probe_problems),
            ]
        )
    return found


def main() -> int:
    base = sys.argv[1] if len(sys.argv) > 1 else "origin/main"
    try:
        merge_base = git("merge-base", base, "HEAD").strip()
    except subprocess.CalledProcessError:
        print(f"note: base {base} not found; skipping governance check")
        return 0
    changed = git("diff", "--name-status", merge_base, "HEAD").splitlines()
    flip = not merge_flip.local_problems(merge_flip.ROOT, merge_base, "HEAD")
    if flip:
        print(f"governance: a lock flip of {merge_flip.SWITCH} (decision 0157); no decision record needed")
    found = failures(
        changed, case_tags, flip,
        added=lambda: added_rust_lines(git("diff", "-U0", "--no-color", merge_base, "HEAD", "--", "*.rs")),
    )
    if deleted_test_files(changed) and any(line.split("\t")[-1].startswith("docs/decisions/") for line in changed):
        print("note: test files deleted with a decision record present: allowed; reviewers must confirm")
    for lines in found:
        print("\n".join(lines))
    if not found:
        protected = sum(is_protected(line.split("\t")[-1]) for line in changed)
        print(f"governance ok ({len(changed)} files changed, {protected} protected)")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
