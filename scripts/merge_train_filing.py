#!/usr/bin/env python3
"""The merge train's response to a mismatch and its task filing (SPEC-0027 section 4, step 7; T-3156):
the revert PR, the P0 and P1 tasks and the scratch worktrees they are built in. Split out of
merge_train.py, which imports it; this module imports nothing from the train."""

from __future__ import annotations

import re
import subprocess
import sys
import tempfile
from collections.abc import Iterator, Sequence
from contextlib import contextmanager
from pathlib import Path

import gh_client
import merge_ready
from gh_client import Client, GhError
from merge_lock import GitError, git

REMOTE = "origin"
MAIN_REF = "refs/heads/main"
ID_CHECK_TIMEOUT = 300
IDENTITY = ("-c", "user.name=merge-train", "-c", "user.email=merge-train@invalid")

REVERT_PREFIX = "merge-train/revert-"
TASKS_PREFIX = "merge-train/tasks-"
# The tasks a train files are the merge train's own (SPEC-0027 is M2 harness work).
TASK_MILESTONE, TASK_LANE = "M2", "harness"
TASK_BODY = """## Goal
{title}: find the cause, and fix it or file the fix.

## Context
Filed by the merge train (SPEC-0027 section 4) from what it saw; every value below is the script's own.
{context}

## Acceptance criteria
- [ ] The cause is found and explained here, and fixed or filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Turning the merge train off: anyone may open the flip to `lock` at once (decision 0157).
"""


class Stop(Exception):
    """The train stops this hold with nothing more merged. `p0` when `main` differs from the
    tested train at PR `pr` (step 7's mismatch response): the run files a P0 task."""

    def __init__(self, reason: str, *, p0: bool = False, pr: int = 0) -> None:
        super().__init__(reason)
        self.p0, self.pr = p0, pr


class Landed(Stop):
    """Step 7's mismatch after a squash: our squash `sha` of PR `pr` is on `main` with a parent or
    tree the train did not test, so the run opens its revert PR (and never merges it)."""

    def __init__(self, pr: int, sha: str, parent: str, expected_parent: str, tree: str, expected_tree: str) -> None:
        super().__init__(f"main differs from the tested train after squashing #{pr} ({sha})", p0=True, pr=pr)
        self.sha, self.parent, self.expected_parent = sha, parent, expected_parent
        self.tree, self.expected_tree = tree, expected_tree


class CutOff(Stop):
    """A stop after which the known-green prefix still merges and the search state is recorded
    (SPEC-0027 section 6, "Budget cut-off"): the step 6 gate failed, or the infrastructure failed
    twice."""


def fetch_main(root: Path) -> str:
    """`main`'s sha on origin, freshly fetched into `refs/remotes/origin/main`."""
    git("fetch", "-q", "--no-tags", REMOTE, f"+{MAIN_REF}:refs/remotes/origin/main", root=root)
    return git("rev-parse", "refs/remotes/origin/main", root=root).stdout.strip()


@contextmanager
def worktrees(root: Path, base: str, names: Sequence[str] = ("train", "m0")) -> Iterator[tuple[Path, ...]]:
    """Scratch worktrees at `base` outside the checkout, one per name; by default the train's and,
    read-only, M0's (its `scripts` run the id checks). All are removed on the way out."""
    with tempfile.TemporaryDirectory(prefix="merge-train-") as scratch:
        trees = tuple(Path(scratch) / name for name in names)
        try:
            for tree in trees:
                git("worktree", "add", "-q", "--detach", str(tree), base, root=root)
            yield trees
        finally:
            for tree in trees:
                git("worktree", "remove", "--force", str(tree), root=root, check=False)


def respond(root: Path, client: Client, log: list[str], tasks: list[tuple[str, str, str]], lease: str,
            stop: Stop) -> None:
    """Step 7's response to a mismatch: the revert PR when our squash landed with an untested
    tree (opened, never merged here), then the P0 task naming it, filed with the hold's other
    tasks by `file_tasks`."""
    lines = [f"- {stop}", f"- train: {lease or 'none'}"]
    if isinstance(stop, Landed):
        lines += [f"- squash: {stop.sha}", f"- expected tree: {stop.expected_tree} (the tested train's)",
                  f"- actual tree: {stop.tree}", f"- expected parent: {stop.expected_parent}",
                  f"- actual parent: {stop.parent or 'none'}"]
        try:
            revert = open_revert(root, client, log, stop)
            lines.append(f"- revert PR: #{revert}; review and merge it like any PR, with P0 priority "
                         "(the merger never merges it)")
        except (GhError, GitError) as error:
            log.append(f"P0: the revert PR of {stop.sha} was not opened: {error}")
            # The task body is the script's own text, so git's or gh's stderr stays in the log.
            lines.append(f"- revert PR: none; it was not opened ({error.__class__.__name__}); "
                         f"revert {stop.sha} by hand")
    else:
        lines.append("- nothing of the train's landed with an untested tree, so there is no revert PR")
    title = f"merge train: main differs from the tested train at #{stop.pr}" if stop.pr else "merge train: P0"
    tasks.insert(0, ("P0", title, "\n".join(lines)))


def open_revert(root: Path, client: Client, log: list[str], landed: Landed) -> int:
    """Reverts `landed.sha` on the current `main` in a scratch worktree, pushes the result to
    `merge-train/revert-<sha>` and opens its PR, whose body the script writes in full."""
    branch = f"{REVERT_PREFIX}{landed.sha}"
    with worktrees(root, fetch_main(root), ("revert",)) as (tree,):
        git(*IDENTITY, "revert", "--no-edit", landed.sha, root=tree)
        git("push", "-q", REMOTE, f"HEAD:refs/heads/{branch}", root=tree)
    body = "\n".join([
        f"The merge train squashed #{landed.pr} as {landed.sha}, and `main` then differed from the tree the "
        "train tested (SPEC-0027 section 4, step 7, \"Response to a mismatch\"). This PR reverts that squash.",
        "",
        f"- squash: {landed.sha}",
        f"- expected tree: {landed.expected_tree}",
        f"- actual tree: {landed.tree}",
        f"- expected parent: {landed.expected_parent}",
        f"- actual parent: {landed.parent or 'none'}",
        "",
        "Review and merge it like any PR, with P0 priority, under the current mode's flow. The merger never "
        "merges it.",
    ])
    pull = client.create_pull(f"revert: merge train mismatch at #{landed.pr}", branch, body)
    pr = gh_client.number(merge_ready.int_of(pull, "number"))
    log.append(f"opened revert PR #{pr} for {landed.sha}")
    return pr


def file_tasks(root: Path, client: Client, log: list[str], tasks: list[tuple[str, str, str]]) -> None:
    """Files the hold's tasks (a P0 of a mismatch, a P1 of a red train that did not reproduce)
    as task files in one task-only PR from `merge-train/tasks-<first id>`, cut from the current
    `main`. A failure is logged with the tasks it did not file: the hold itself is over."""
    if not tasks:
        return
    try:
        with worktrees(root, fetch_main(root), ("tasks",)) as (tree,):
            ids = [write_task(tree, *task) for task in tasks]
            listed = ", ".join(ids)
            git("add", "-A", "docs/tasks", root=tree)
            git(*IDENTITY, "commit", "-q", "-m", f"tasks: merge train {listed}", root=tree)
            branch = f"{TASKS_PREFIX}{ids[0]}"
            git("push", "-q", REMOTE, f"HEAD:refs/heads/{branch}", root=tree)
        body = "\n".join([
            "Task-only PR filed by the merge train (SPEC-0027 section 4).", "",
            *(f"- {tid} ({priority}): {title}" for tid, (priority, title, _) in zip(ids, tasks, strict=True)),
        ])
        pull = client.create_pull(f"tasks: merge train {listed}", branch, body)
        pr = gh_client.number(merge_ready.int_of(pull, "number"))
        log.append(f"filed {listed} in #{pr}")
    except (GhError, GitError) as error:
        log.append(f"the tasks were not filed: {error}")
        log += [f"unfiled {priority}: {title}" for priority, title, _ in tasks]


def write_task(tree: Path, priority: str, title: str, context: str) -> str:
    """A task file in `tree` with a collision-free id from `main`'s `tasks.py new`; returns the id."""
    argv = [sys.executable, "-I", str(tree / "scripts" / "tasks.py"), "new", "--milestone", TASK_MILESTONE,
            "--lane", TASK_LANE, "--title", title, "--priority", priority, "--size", "S", "--tier", "opus"]
    try:
        run = subprocess.run(argv, cwd=tree, stdin=subprocess.DEVNULL, capture_output=True, text=True,
                             timeout=ID_CHECK_TIMEOUT, check=False)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise GitError(f"tasks.py new: {error.__class__.__name__}") from error
    wrote = re.search(r"^wrote (docs/tasks/M\d+/T-\d{4}-[a-z0-9-]+\.md): ", run.stdout, re.M)
    if run.returncode != 0 or wrote is None:
        raise GitError(f"tasks.py new: exit {run.returncode}")
    path = tree / wrote[1]
    front = path.read_text().split("\n---\n", 1)[0]
    tid = re.search(r"^id: (T-\d{4})$", front, re.M)
    if tid is None:
        raise GitError(f"tasks.py new wrote no id in {wrote[1]}")
    path.write_text(f"{front}\n---\n\n" + TASK_BODY.format(title=title, context=context))
    return tid[1]
