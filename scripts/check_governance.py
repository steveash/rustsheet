#!/usr/bin/env python3
"""Governance gate for autonomous development.

Some paths define the rules agents work under (CI, lint policy, baselines, agent
instructions). A PR that changes them must also add or update a decision record in
docs/decisions/ explaining why, so rules are never silently weakened. Additionally,
`--allow-regressions` baseline changes and test deletions are flagged, and a newly added probes
case must carry exactly one milestone tag, M2 or later (decisions 0043, 0051, 0057).

Usage: scripts/check_governance.py [BASE_REF]   (default: origin/main)
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from collections.abc import Callable, Iterable

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


def main() -> int:
    base = sys.argv[1] if len(sys.argv) > 1 else "origin/main"
    try:
        merge_base = git("merge-base", base, "HEAD").strip()
    except subprocess.CalledProcessError:
        print(f"note: base {base} not found; skipping governance check")
        return 0
    changed = git("diff", "--name-status", merge_base, "HEAD").splitlines()
    paths = [line.split("\t")[-1] for line in changed]
    protected = [p for p in paths if is_protected(p)]
    decisions = [p for p in paths if p.startswith("docs/decisions/") and p.endswith(".md")]
    failed = False
    if protected and not decisions:
        failed = True
        print("FAIL governance: protected paths changed without a decision record in docs/decisions/:")
        for p in protected:
            print(f"  {p}")
    deleted_tests = [
        line.split("\t")[-1]
        for line in changed
        if line.startswith("D") and ("/tests/" in line or "/test/" in line or line.endswith("_test.py"))
    ]
    if deleted_tests:
        failed = True
        print("FAIL governance: test files deleted (requires a decision record and reviewer sign-off):")
        for p in deleted_tests:
            print(f"  {p}")
        if decisions:
            failed = False
            print("  (decision record present — allowed; reviewers must confirm)")
    probe_problems = probe_tag_problems(changed, case_tags)
    if probe_problems:
        failed = True
        print(
            "FAIL governance: new probe cases need exactly one milestone tag, M2 or later, the "
            "milestone of the function or feature they probe (decision 0057):"
        )
        for problem in probe_problems:
            print(f"  {problem}")
    if not failed:
        print(f"governance ok ({len(paths)} files changed, {len(protected)} protected)")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
