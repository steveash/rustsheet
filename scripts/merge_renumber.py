#!/usr/bin/env python3
"""Clear a PR the merge train ejected for an id collision (decision 0224).

  scripts/merge_renumber.py --pr N   # exit 0: pushed; 1: refused (nothing pushed); 2: error

In this checkout, which must have no local changes, it does what `make catch-up` does
(scripts/merge_catchup.py), from a renumbered PR head:
- finds the task and decision files the PR adds since its merge base with `main` whose id `main`
  also has (`id_renumber.colliding`); refuses when there are none, or when the merge with `main`
  conflicts outside the eval-gated baseline lists;
- reserves a fresh id for each in the id registry (decision 0215): a task id in the task's own
  milestone (`tasks.reserve_id`), a decision number (`id_registry.allocate`). It never picks one
  by hand, and refuses when the registry cannot reserve one;
- renumbers the PR head (`id_renumber.renumbered_commit`: the files move to the new ids, and the
  old id becomes the new one only in the files the PR adds and the lines it adds);
- merges `main` into that without committing, resolves conflicts in the eval-gated lists as
  catch-up does and re-records them only then;
- commits one merge commit whose parents are the PR head and `main`, pushes it to the PR branch
  (never forced), and restores the checkout.

When CI is green on the pushed head, `make mark-ready PR=N CARRY=1` re-marks the PR: the carry rule
accepts exactly this shape (`merge_ready.Repo.carry_problem`, `id_renumber.renumbered_merge`).
"""

from __future__ import annotations

import sys
from collections.abc import Callable, Sequence
from pathlib import Path

import id_registry
import id_renumber
import merge_catchup
import merge_ready
import new_decision
import tasks
from merge_catchup import Refused
from merge_lock import git

# (the checkout, old id, the task's milestone or "" for a decision, ids already taken) -> a
# reserved fresh id.
Allocate = Callable[[Path, str, str, set[str]], str]


def reserve(root: Path, old: str, milestone: str, taken: set[str]) -> str:
    """A fresh id of `old`'s kind, reserved in the id registry by the rule `make new-task` (in
    `milestone`) or `make new-decision` uses; Refused when the registry cannot reserve it. Those
    rules read the task files and branches of the checkout this script belongs to, so `root`
    must be that checkout."""
    if root.resolve() != tasks.ROOT.resolve():
        raise Refused(f"ids are reserved from the checkout of {tasks.ROOT}, not {root}")
    if id_renumber.TASK_ID.fullmatch(old):
        known = tasks.load()
        used = taken | {str(t["id"]) for t in known} | tasks.git_task_ids(fetch=True)
        chosen, note = tasks.reserve_id(
            lambda claimed: tasks.next_id(milestone, known, used | claimed), fetch=True
        )
        reserved = note == id_registry.note(True)
    else:
        used = taken | new_decision.numbers_in_tree(new_decision.check_decisions.DECISIONS)
        used |= tasks.git_branch_names("docs/decisions", new_decision.check_decisions.NAME, fetch=True)
        chosen, reserved = id_registry.allocate(
            lambda claimed: new_decision.next_number(used | claimed), id_registry.DECISION, fetch=True
        )
    if not reserved:
        raise Refused(f"the id registry could not reserve a new id for {old}")
    return chosen


def milestone_of(root: Path, head: str, path: str) -> str:
    """The `milestone:` of task file `path` in `head`, or its directory's name; "" for a decision."""
    if not id_renumber.TASK_PATH.match(path):
        return ""
    text = git("show", f"{head}:{path}", root=root).stdout
    try:
        milestone = str(tasks.front_matter(text, Path(path)).get("milestone", ""))
    except ValueError:
        milestone = ""
    if not tasks.MILESTONE.fullmatch(milestone):
        milestone = path.split("/")[2] if path.count("/") >= 3 else ""
    if not tasks.MILESTONE.fullmatch(milestone):
        raise Refused(f"{path} names no milestone")
    return milestone


def renumbering(allocate: Allocate) -> merge_catchup.Rewrite:
    """The `merge_catchup.Rewrite` that renumbers the PR head's colliding ids first."""

    def rewrite(root: Path, head: str, main: str) -> tuple[str, str]:
        try:
            base = id_renumber.merge_base(root, head, main)
            found = id_renumber.colliding(root, base, head, main)
            if not found:
                raise Refused(f"the PR adds no task or decision file whose id {merge_ready.BASE_BRANCH} has")
            conflicts = merge_catchup.conflicts_outside(merge_ready.Repo(root), head, main)
            if conflicts is None or conflicts:
                raise Refused("the merge with main conflicts outside the eval-gated baseline lists")
            # Before any id is reserved: an old id in a line the PR added outside docs/ is the worker's.
            outside = id_renumber.uses_outside_docs(root, base, head, {old: old for old in found})
            if outside:
                raise Refused(
                    f"lines the PR added outside {id_renumber.DOCS} name a colliding id "
                    f"({', '.join(outside[:3])}): only the PR's worker may change them"
                )
            taken = id_renumber.ids_in(root, main) | id_renumber.ids_in(root, head)
            mapping: dict[str, str] = {}
            for old, path in sorted(found.items()):
                mapping[old] = allocate(root, old, milestone_of(root, head, path), taken | set(mapping.values()))
            id_renumber.check_mapping(root, base, head, main, mapping)
            commit = id_renumber.renumbered_commit(root, base, head, mapping)
        except id_renumber.RenumberError as error:
            raise Refused(str(error)) from error
        moves = ", ".join(f"{old} -> {new}" for old, new in sorted(mapping.items()))
        return commit, f"Merge {merge_ready.BASE_BRANCH}; renumber {moves} (make renumber, decision 0224)"

    return rewrite


def renumber(
    root: Path,
    branch: str,
    allocate: Allocate = reserve,
    record: Callable[[Path, Path], int] = merge_catchup.rerecord,
    before_push: Callable[[str], None] | None = None,
) -> str:
    """The module docstring's steps on `branch`; returns the pushed head. Raises Refused (nothing
    pushed). `before_push` gets the merge commit's sha before the push (the janitor's note)."""
    return merge_catchup.catch_up(
        root, branch, record, renumbering(allocate), record_clean=False, before_push=before_push
    )


def main(argv: Sequence[str] | None = None) -> int:
    return merge_catchup.run_cli(argv, __doc__ or "", "merge_renumber", "renumbering", renumber)


if __name__ == "__main__":
    sys.exit(main())
