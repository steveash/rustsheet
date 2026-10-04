#!/usr/bin/env python3
"""Task board for autonomous agents. Tasks are markdown files with front matter in docs/tasks/.

  scripts/tasks.py list [--lane L] [--status S] [--milestone M]
  scripts/tasks.py next --lane L [--tier T] [--no-fetch]  # best ready, unclaimed task; handoffs
                                         # (also on pushed agent/* branches) count as opus, come first
  scripts/tasks.py validate              # schema + dependency checks (CI)
  scripts/tasks.py board                 # write docs/tasks/BOARD.md (local view, git-ignored)
  scripts/tasks.py new --milestone M1 --lane engine --title "..." [--priority P1] [--size S]
                       [--tier sonnet|opus] [--no-fetch]
                                         # write a task file with the next free id (decision 0016)

Front matter keys: id, title, milestone, lane, status, priority, size, tier, depends_on, spec.
See docs/process/task-format.md.
"""

from __future__ import annotations

import re
import subprocess
import sys
import time
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TASKS = ROOT / "docs" / "tasks"
STATUSES = ["todo", "in-progress", "blocked", "done", "dropped"]
LANES = ["harness", "engine", "functions", "api", "perf", "quality", "research"]
PRIORITIES = ["P0", "P1", "P2", "P3"]
SIZES = ["S", "M", "L"]
TIERS = ["sonnet", "opus"]  # model tier that works the task (decision 0018)
OPEN = ("todo", "in-progress", "blocked")  # not done or dropped; the first two are "active"
MILESTONE = re.compile(r"^M\d+$")
TASK_FILE = re.compile(r"(?:^|/)(T-\d{4})-[^/]*\.md$")


def parse(path: Path) -> dict[str, object]:
    text = path.read_text()
    m = re.match(r"^---\n(.*?)\n---\n", text, re.S)
    if not m:
        raise ValueError(f"{path}: missing front matter")
    meta: dict[str, object] = {"path": path}
    for line in m.group(1).splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        key, _, value = line.partition(":")
        value = value.strip()
        if value.startswith("[") and value.endswith("]"):
            meta[key.strip()] = [v.strip() for v in value[1:-1].split(",") if v.strip()]
        else:
            meta[key.strip()] = value.strip('"')
    return meta


def path_of(t: dict[str, object]) -> Path:
    """The task file a parsed task came from (`parse` stores it under "path")."""
    return Path(str(t["path"]))


def load() -> list[dict[str, object]]:
    return [parse(p) for p in sorted(TASKS.glob("*/*.md")) if p.name != "README.md"]


def validate(tasks: list[dict[str, object]]) -> list[str]:
    errors = []
    ids = {}
    for t in tasks:
        tid = str(t.get("id", ""))
        where = t["path"].relative_to(ROOT)  # type: ignore[union-attr]
        if not re.match(r"^T-\d{4}$", tid):
            errors.append(f"{where}: bad id {tid!r}")
        if tid in ids:
            first = path_of(ids[tid]).relative_to(ROOT)
            errors.append(
                f"{where}: duplicate id {tid} (also {first}); renumber the newer task with "
                "`make new-task` (decision 0016)"
            )
        ids[tid] = t
        for key, allowed in (("status", STATUSES), ("lane", LANES), ("priority", PRIORITIES), ("size", SIZES)):
            if t.get(key) not in allowed:
                errors.append(f"{where}: {key}={t.get(key)!r} not in {allowed}")
        if not MILESTONE.match(str(t.get("milestone", ""))):
            errors.append(f"{where}: bad milestone {t.get('milestone')!r}")
        if not t.get("title"):
            errors.append(f"{where}: missing title")
        if t.get("status") in OPEN and t.get("tier") not in TIERS:
            errors.append(f"{where}: tier={t.get('tier')!r} not in {TIERS} (open tasks need a tier)")
        if "TODO(fill)" in path_of(t).read_text():
            errors.append(f"{where}: template placeholders `TODO(fill)` left in the task file")
        if t.get("size") == "L" and t.get("status") in OPEN[:2]:
            errors.append(f"{where}: size L tasks must be split before work starts")
    for t in tasks:
        for dep in t.get("depends_on", []) or []:  # type: ignore[union-attr]
            if dep not in ids:
                errors.append(f"{t['id']}: unknown dependency {dep}")
    return errors


def ready(t: dict[str, object], by_id: dict[str, dict[str, object]]) -> bool:
    deps = t.get("depends_on", []) or []
    return t.get("status") == "todo" and all(by_id.get(d, {}).get("status") == "done" for d in deps)  # type: ignore[union-attr]


def git_task_ids(fetch: bool = True) -> set[str]:
    """Task ids in docs/tasks/ on every `origin/*` branch: work that other sessions have pushed.

    Best effort: without git or a network the working tree alone decides, and `validate` in CI
    still catches a collision.
    """

    def git(*args: str) -> str:
        return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True, check=True).stdout

    try:
        if fetch:
            git("fetch", "-q", "origin")
        refs = git("for-each-ref", "--format=%(refname)", "refs/remotes/origin").split()
        found: set[str] = set()
        for ref in refs:
            for name in git("ls-tree", "-r", "--name-only", ref, "--", "docs/tasks").splitlines():
                m = TASK_FILE.search(name)
                if m:
                    found.add(m.group(1))
        return found
    except (OSError, subprocess.CalledProcessError) as e:
        print(f"warning: could not read origin branches ({e}); ids checked against the working tree only",
              file=sys.stderr)
        return set()


def next_id(milestone: str, tasks: list[dict[str, object]], taken: set[str]) -> str:
    """Next id after the highest taken one in the hundred-block the milestone uses most
    (M0 -> T-00xx, M1 -> T-01xx). Ids only grow, so gaps are never reused.

    A milestone with no tasks, or whose block is full, gets the next id above every taken one.
    """
    used = {int(i[2:]) for i in taken}
    blocks = Counter(int(str(t["id"])[2:]) // 100 for t in tasks if t.get("milestone") == milestone)
    if blocks:
        block = blocks.most_common(1)[0][0]
        n = max((u for u in used if u // 100 == block), default=block * 100) + 1
        if n < block * 100 + 100:
            return f"T-{n:04d}"
    return f"T-{max(used, default=0) + 1:04d}"


def slug(title: str) -> str:
    words = re.sub(r"[^a-z0-9]+", " ", title.lower()).split()
    return "-".join(words[:8]) or "task"


TEMPLATE = """---
id: {id}
title: "{title}"
milestone: {milestone}
lane: {lane}
status: todo
priority: {priority}
size: {size}
tier: {tier}
depends_on: []
---

## Goal
TODO(fill): the outcome, not the implementation.

## Context
TODO(fill): why; failing case ids; semantics notes; code paths.

## Acceptance criteria
- [ ] TODO(fill): verifiable statements.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
"""


def new_task(tasks: list[dict[str, object]], fetch: bool = True) -> int:
    """Writes a task file with the next free id (`--no-fetch` skips `git fetch`, e.g. offline)."""
    milestone, lane, title = arg("--milestone"), arg("--lane"), arg("--title")
    priority, size, tier = arg("--priority") or "P2", arg("--size") or "S", arg("--tier") or "opus"
    if not (milestone and MILESTONE.fullmatch(milestone) and lane in LANES and title) or "\n" in title:
        print("usage: tasks.py new --milestone M<n> --lane <lane> --title <one-line title> "
              "[--priority P2] [--size S] [--tier opus] [--no-fetch]")
        return 2
    if priority not in PRIORITIES or size not in SIZES or size == "L" or tier not in TIERS:
        print(f"priority must be one of {PRIORITIES}; size one of S, M (split L tasks); tier one of {TIERS}")
        return 2
    taken = {str(t["id"]) for t in tasks} | git_task_ids(fetch)
    tid = next_id(milestone, tasks, taken)
    path = TASKS / milestone / f"{tid}-{slug(title)}.md"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(TEMPLATE.format(id=tid, title=title.replace('"', "'"), milestone=milestone,
                                    lane=lane, priority=priority, size=size, tier=tier))
    print(f"wrote {path.relative_to(ROOT)}: fill in the TODO(fill) lines, then commit and push it now to claim {tid}")
    return 0


AGENT_BRANCH = re.compile(r"^refs/remotes/origin/agent/[^/]+/(T-\d{4})(?:-|$)")
HANDOFF = re.compile(r"^## Handoff\s*$", re.M)
CLAIM_TTL_SECONDS = 24 * 3600  # a pushed agent/* branch younger than this claims its task


def branch_claims(fetch: bool = True) -> tuple[set[str], set[str]]:
    """(claimed, handed_off) task ids from pushed `agent/<lane>/T-XXXX-*` branches.

    A pushed branch with a commit in the last 24 h is a claim: another worker must not start the
    same task (work-loop step 2).
    A branch whose task file has a `## Handoff` section was escalated to opus (step 5b); main
    still shows `tier: sonnet` until it merges (decision 0018). Best effort, like `git_task_ids`.
    """

    def git(*args: str) -> str:
        return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True, check=True).stdout

    try:
        if fetch:
            git("fetch", "-q", "--prune", "origin")
        claimed: set[str] = set()
        handed: set[str] = set()
        now = time.time()
        refs = git("for-each-ref", "--format=%(refname) %(committerdate:unix)", "refs/remotes/origin/agent")
        for line in refs.splitlines():
            ref, _, stamp = line.partition(" ")
            m = AGENT_BRANCH.match(ref)
            if not m:
                continue
            # Stale branches (abandoned or merged-but-undeleted) are not claims; handoffs are, at any age.
            if stamp.isdigit() and now - int(stamp) < CLAIM_TTL_SECONDS:
                claimed.add(m.group(1))
            for name in git("ls-tree", "-r", "--name-only", ref, "--", "docs/tasks").splitlines():
                if f"/{m.group(1)}-" in name and HANDOFF.search(git("show", f"{ref}:{name}")):
                    handed.add(m.group(1))
        return claimed, handed
    except (OSError, subprocess.CalledProcessError) as e:
        print(f"warning: could not read agent branches ({e}); claims and handoffs on branches not seen",
              file=sys.stderr)
        return set(), set()


def handed_off(t: dict[str, object], on_branches: set[str]) -> bool:
    """A sonnet run escalated this task (work-loop step 5b), on main or on its pushed branch."""
    return str(t["id"]) in on_branches or bool(HANDOFF.search(path_of(t).read_text()))


def effective_tier(t: dict[str, object], on_branches: set[str]) -> object:
    """A handed-off task belongs to opus even while main still says `tier: sonnet`."""
    return "opus" if handed_off(t, on_branches) else t.get("tier")


def arg(name: str) -> str | None:
    return sys.argv[sys.argv.index(name) + 1] if name in sys.argv else None


def main() -> int:
    cmd = sys.argv[1] if len(sys.argv) > 1 else "list"
    tasks = load()
    by_id = {str(t["id"]): t for t in tasks}
    if cmd == "validate":
        errors = validate(tasks)
        for e in errors:
            print(f"FAIL {e}")
        print(f"{len(tasks)} tasks, {len(errors)} errors")
        return 1 if errors else 0
    if cmd == "new":
        return new_task(tasks, fetch="--no-fetch" not in sys.argv)
    if cmd == "next":
        lane, tier = arg("--lane"), arg("--tier")
        claimed, on_branches = branch_claims(fetch="--no-fetch" not in sys.argv)
        cands = [
            t
            for t in tasks
            if ready(t, by_id)
            and (lane is None or t.get("lane") == lane)
            and (tier is None or effective_tier(t, on_branches) == tier)
            # Someone else's pushed claim; a handoff is the opus worker's to resume (decision 0018).
            and (str(t["id"]) not in claimed or (handed_off(t, on_branches) and tier != "sonnet"))
        ]
        cands.sort(
            key=lambda t: (
                not handed_off(t, on_branches),
                str(t.get("milestone"))[1:].zfill(3),
                str(t.get("priority")),
                str(t["id"]),
            )
        )
        if not cands:
            print("no ready task")
            return 1
        t = cands[0]
        print(f"{t['id']}  {t['title']}  ({t['path'].relative_to(ROOT)})")  # type: ignore[union-attr]
        return 0
    if cmd == "board":
        lines = ["# Task board", "", "Generated by `make board` (git-ignored; decision 0017). Do not edit or commit it.", ""]
        for status in STATUSES:
            group = [t for t in tasks if t.get("status") == status]
            if not group:
                continue
            lines += [f"## {status} ({len(group)})", "", "| id | milestone | lane | pri | title |", "|---|---|---|---|---|"]
            for t in sorted(group, key=lambda t: str(t["id"])):
                rel = t["path"].relative_to(TASKS)  # type: ignore[union-attr]
                lines.append(f"| [{t['id']}]({rel}) | {t.get('milestone')} | {t.get('lane')} | {t.get('priority')} | {t.get('title')} |")
            lines.append("")
        (TASKS / "BOARD.md").write_text("\n".join(lines))
        print("wrote docs/tasks/BOARD.md")
        return 0
    lane, status, ms = arg("--lane"), arg("--status"), arg("--milestone")
    for t in tasks:
        if (lane and t.get("lane") != lane) or (status and t.get("status") != status) or (ms and t.get("milestone") != ms):
            continue
        mark = "*" if ready(t, by_id) else " "
        print(
            f"{mark} {t['id']} [{t.get('status'):<11}] {t.get('milestone'):<4} {t.get('lane'):<9} "
            f"{t.get('priority')} {str(t.get('tier', '-')):<6} {t.get('title')}"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
