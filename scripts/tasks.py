#!/usr/bin/env python3
"""Task board for autonomous agents. Tasks are markdown files with front matter in docs/tasks/.

  scripts/tasks.py list [--lane L] [--status S] [--milestone M]
  scripts/tasks.py next --lane L [--tier T] [--no-fetch]  # best ready, unclaimed task in a lane
                                         # that is not busy; handoffs (also on pushed agent/*
                                         # branches) count as opus, come first; --lane functions
                                         # means any functions-* lane (decision 0022)
  scripts/tasks.py ready [--lane L] [--tier T] [--no-fetch]  # every task `next` could pick, in its
                                         # order, then "N ready" (queue health, decision 0021)
  scripts/tasks.py validate              # schema + dependency checks, and every TODO(T-XXXX) in
                                         # code names an open task (CI)
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
# The functions lane is split by function family so several workers can fix functions at once
# (decision 0022). Plain "functions" remains valid only on done/dropped tasks, and as the `next`
# / `ready` lane filter meaning "any functions-* lane".
FUNCTION_LANES = [
    "functions-math",
    "functions-stat",
    "functions-criteria",
    "functions-financial",
    "functions-engineering",
    "functions-text",
    "functions-lookup",
    "functions-date",
    "functions-logic",
]
LANES = ["harness", "engine", "api", "perf", "quality", "research", *FUNCTION_LANES, "functions"]
BUSY_SECONDS = 2 * 3600  # a lane whose pushed agent branch moved this recently is busy
PRIORITIES = ["P0", "P1", "P2", "P3"]
SIZES = ["S", "M", "L"]
TIERS = ["sonnet", "opus"]  # model tier that works the task (decision 0018)
OPEN = ("todo", "in-progress", "blocked")  # not done or dropped
ACTIVE = ("todo", "in-progress")
MILESTONE = re.compile(r"^M\d+$")
TASK_FILE = re.compile(r"(?:^|/)(T-\d{4})-[^/]*\.md$")
TODO_OWNER = re.compile(r"TODO\((T-\d{4})\)")
# Code whose TODOs must name an open task; docs/ quotes old TODOs as history.
TODO_DIRS = ("crates", "bindings", "tools", "scripts")


def parse(path: Path) -> dict[str, object]:
    return front_matter(path.read_text(), path)


def front_matter(text: str, path: Path) -> dict[str, object]:
    """The front matter of task file `text` (read from `path`, on disk or on a branch)."""
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
        if t.get("status") in OPEN and t.get("lane") == "functions":
            errors.append(
                f"{where}: lane='functions' is retired for open tasks; use a functions-* lane "
                "(table in docs/process/agent-workflow.md; decision 0022)"
            )
        if t.get("status") in OPEN and t.get("tier") not in TIERS:
            errors.append(f"{where}: tier={t.get('tier')!r} not in {TIERS} (open tasks need a tier)")
        if "TODO(fill)" in path_of(t).read_text():
            errors.append(f"{where}: template placeholders `TODO(fill)` left in the task file")
        if t.get("size") == "L" and t.get("status") in ACTIVE:
            errors.append(f"{where}: size L tasks must be split before work starts")
    for t in tasks:
        for dep in t.get("depends_on", []) or []:  # type: ignore[union-attr]
            if dep not in ids:
                errors.append(f"{t['id']}: unknown dependency {dep}")
    return errors


def stale_todos(tasks: list[dict[str, object]], files: dict[str, str]) -> list[str]:
    """Every `TODO(T-XXXX)` in `files` (path -> text) whose task is not open: done, dropped or
    unknown. A TODO's owner must be a task someone will still pick up.
    """
    status = {str(t.get("id")): str(t.get("status")) for t in tasks}
    errors = []
    for path, text in sorted(files.items()):
        for n, line in enumerate(text.splitlines(), 1):
            for tid in TODO_OWNER.findall(line):
                if status.get(tid) in OPEN:
                    continue
                state = f"is {status[tid]}" if tid in status else "does not exist"
                errors.append(
                    f"{path}:{n}: TODO({tid}) names a task that {state}; resolve the TODO, or re-point "
                    "it to the open task that owns the work"
                )
    return errors


def code_files() -> dict[str, str]:
    """Tracked files under TODO_DIRS (git ls-files skips build outputs and .venv), path -> text."""
    out = subprocess.run(
        ["git", "-C", str(ROOT), "ls-files", "-z", "--", *TODO_DIRS], capture_output=True, text=True, check=True
    ).stdout
    files = {}
    for name in out.split("\0"):
        path = ROOT / name
        if name and path.is_file():
            files[name] = path.read_text(errors="replace")
    return files


def ready(t: dict[str, object], by_id: dict[str, dict[str, object]]) -> bool:
    deps = t.get("depends_on", []) or []
    return t.get("status") == "todo" and all(by_id.get(d, {}).get("status") == "done" for d in deps)  # type: ignore[union-attr]


def git_branch_names(directory: str, pattern: re.Pattern[str], fetch: bool = True) -> set[str]:
    """`pattern`'s first group for every file name under `directory` on every `origin/*` branch:
    ids that other sessions have pushed (task ids, decision numbers; decisions 0016, 0029).

    Best effort: without git or a network the working tree alone decides, and the validators in
    CI still catch a collision.
    """

    def git(*args: str) -> str:
        return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True, check=True).stdout

    try:
        if fetch:
            git("fetch", "-q", "origin")
        refs = git("for-each-ref", "--format=%(refname)", "refs/remotes/origin").split()
        found: set[str] = set()
        for ref in refs:
            for name in git("ls-tree", "-r", "--name-only", ref, "--", directory).splitlines():
                m = pattern.search(name.rsplit("/", 1)[-1])
                if m:
                    found.add(m.group(1))
        return found
    except (OSError, subprocess.CalledProcessError) as e:
        print(f"warning: could not read origin branches ({e}); ids checked against the working tree only",
              file=sys.stderr)
        return set()


def git_task_ids(fetch: bool = True) -> set[str]:
    """Task ids in docs/tasks/ on every `origin/*` branch."""
    return git_branch_names("docs/tasks", TASK_FILE, fetch)


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


def slug(title: str, default: str = "task") -> str:
    """At most 8 lowercase ASCII words of `title` joined by `-` (matches `[a-z0-9-]+`)."""
    words = re.sub(r"[^a-z0-9]+", " ", title.lower()).split()
    return "-".join(words[:8]) or default


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
    if not (milestone and MILESTONE.fullmatch(milestone) and lane in LANES and lane != "functions" and title) or "\n" in title:
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


AGENT_BRANCH = re.compile(r"^refs/remotes/origin/agent/([^/]+)/(T-\d{4})(?:-|$)")
# An unresumed handoff; the opus run that picks it up renames it "## Handoff (resumed)".
HANDOFF = re.compile(r"^## Handoff\s*$", re.M)
CLAIM_TTL_SECONDS = 24 * 3600  # a pushed agent/* branch younger than this claims its task


def branch_claims(fetch: bool = True) -> tuple[set[str], set[str], set[tuple[str, str]]]:
    """(claimed, handed_off, active) from pushed `agent/<lane>/T-XXXX-*` branches.

    A pushed branch with a commit in the last 24 h is a claim: another worker must not start the
    same task (work-loop step 2). A branch whose task is `status: blocked` on the branch is a claim
    at any age: main still says `todo`, and the blocker is recorded only on the branch (decision 0048).
    A branch whose task file has a `## Handoff` section was escalated to opus (step 5b); main
    still shows `tier: sonnet` until it merges (decision 0018).
    `active` holds (lane, task id) for branches with a commit in the last 2 h that are not
    unresumed handoffs (those wait for the opus worker) and whose task is not `status: blocked`
    on the branch (it cannot progress, but stays claimed); `available` turns them into busy lanes
    (decision 0022). Best effort, like `git_task_ids`.
    """

    def git(*args: str) -> str:
        return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True, check=True).stdout

    try:
        if fetch:
            git("fetch", "-q", "--prune", "origin")
        claimed: set[str] = set()
        handed: set[str] = set()
        blocked: set[str] = set()
        recent: dict[str, str] = {}  # task id -> lane, for branches younger than BUSY_SECONDS
        now = time.time()
        refs = git("for-each-ref", "--format=%(refname) %(committerdate:unix)", "refs/remotes/origin/agent")
        for line in refs.splitlines():
            ref, _, stamp = line.partition(" ")
            m = AGENT_BRANCH.match(ref)
            if not m:
                continue
            lane, tid = m.group(1), m.group(2)
            age = now - int(stamp) if stamp.isdigit() else float("inf")
            # Stale branches (abandoned or merged-but-undeleted) are not claims; handoffs and tasks
            # blocked on the branch are, at any age (decision 0048).
            if age < CLAIM_TTL_SECONDS:
                claimed.add(tid)
            if age < BUSY_SECONDS:
                recent[tid] = lane
            for name in git("ls-tree", "-r", "--name-only", ref, "--", "docs/tasks").splitlines():
                if f"/{tid}-" not in name:
                    continue
                text = git("show", f"{ref}:{name}")
                if HANDOFF.search(text):
                    handed.add(tid)
                try:
                    if front_matter(text, Path(name)).get("status") == "blocked":
                        blocked.add(tid)
                except ValueError:  # malformed front matter on a branch: not known to be blocked
                    pass
        active = {(lane, tid) for tid, lane in recent.items() if tid not in handed | blocked}
        return claimed | blocked, handed, active
    except (OSError, subprocess.CalledProcessError) as e:
        print(f"warning: could not read agent branches ({e}); claims and handoffs on branches not seen",
              file=sys.stderr)
        return set(), set(), set()


def handed_off(t: dict[str, object], on_branches: set[str]) -> bool:
    """A sonnet run escalated this task (work-loop step 5b), on main or on its pushed branch."""
    return str(t["id"]) in on_branches or bool(HANDOFF.search(path_of(t).read_text()))


def effective_tier(t: dict[str, object], on_branches: set[str]) -> object:
    """A handed-off task belongs to opus even while main still says `tier: sonnet`."""
    return "opus" if handed_off(t, on_branches) else t.get("tier")


def in_lane(t: dict[str, object], lane: str | None) -> bool:
    """`lane` matches itself; "functions" matches every functions-* lane (decision 0022)."""
    if lane is None:
        return True
    if lane == "functions":
        return str(t.get("lane")) in (*FUNCTION_LANES, "functions")
    return t.get("lane") == lane


def available(
    tasks: list[dict[str, object]],
    by_id: dict[str, dict[str, object]],
    lane: str | None,
    tier: str | None,
    fetch: bool = True,
) -> list[dict[str, object]]:
    """Ready tasks a worker of `tier` may start in `lane`, best first: P0, then handoffs, then
    milestone and priority (a P0 never waits behind an escalated lower-priority task; decision 0023)."""
    claimed, on_branches, active = branch_claims(fetch=fetch)
    # One task in flight per lane. A branch whose task is already done on main (merged, branch not
    # yet deleted) leaves its lane free, so a looping worker can take the next task at once.
    # The task's own `lane:` decides (the branch name is only a fallback for tasks not on main).
    busy = {
        str(by_id[tid].get("lane")) if tid in by_id else lane
        for lane, tid in active
        if by_id.get(tid, {}).get("status") not in ("done", "dropped")
    }
    cands = [
        t
        for t in tasks
        if ready(t, by_id)
        and in_lane(t, lane)
        and (tier is None or effective_tier(t, on_branches) == tier)
        # Someone else's pushed claim; a handoff is the opus worker's to resume (decision 0018).
        and (str(t["id"]) not in claimed or (handed_off(t, on_branches) and tier != "sonnet"))
        # One task in flight per lane; a handoff is resumed whatever its lane is doing.
        and (t.get("lane") not in busy or (handed_off(t, on_branches) and tier != "sonnet"))
    ]
    cands.sort(
        key=lambda t: (
            str(t.get("priority")) != "P0",
            not handed_off(t, on_branches),
            str(t.get("milestone"))[1:].zfill(3),
            str(t.get("priority")),
            str(t["id"]),
        )
    )
    return cands


def arg(name: str) -> str | None:
    return sys.argv[sys.argv.index(name) + 1] if name in sys.argv else None


def main() -> int:
    cmd = sys.argv[1] if len(sys.argv) > 1 else "list"
    tasks = load()
    by_id = {str(t["id"]): t for t in tasks}
    if cmd == "validate":
        errors = validate(tasks) + stale_todos(tasks, code_files())
        for e in errors:
            print(f"FAIL {e}")
        print(f"{len(tasks)} tasks, {len(errors)} errors")
        return 1 if errors else 0
    if cmd == "new":
        return new_task(tasks, fetch="--no-fetch" not in sys.argv)
    if cmd in ("next", "ready"):
        cands = available(tasks, by_id, arg("--lane"), arg("--tier"), fetch="--no-fetch" not in sys.argv)
        if cmd == "ready":
            for t in cands:
                print(f"{t['id']}  {t.get('lane')!s:<9} {t['title']}")
            print(f"{len(cands)} ready")
            return 0
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
        if not in_lane(t, lane) or (status and t.get("status") != status) or (ms and t.get("milestone") != ms):
            continue
        mark = "*" if ready(t, by_id) else " "
        print(
            f"{mark} {t['id']} [{t.get('status'):<11}] {t.get('milestone'):<4} {t.get('lane'):<9} "
            f"{t.get('priority')} {str(t.get('tier', '-')):<6} {t.get('title')}"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
