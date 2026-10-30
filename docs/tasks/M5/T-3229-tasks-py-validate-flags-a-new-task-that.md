---
id: T-3229
title: "tasks.py validate flags a new task that lists cells an open task already owns"
milestone: M5
lane: quality
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
A PR cannot add a task whose eval cells an open task already lists unless one of the two tasks names the other.
That rule catches duplicate tasks at `make hygiene` and in CI, not in review.

## Context
On 2026-10-30, the quality reviewer found that 5 of the 15 tasks in four idle refill PRs duplicated open tasks on main:
- T-2915 duplicated T-2928 (#2037, closed);
- T-3159 duplicated T-1749 (#2338);
- T-3160 duplicated T-3070, T-2561, T-3013 and T-2638 (#2338);
- T-3194 duplicated T-3217 (#2379);
- T-2859 duplicated T-2724 (#2005).

Parallel gap triages each read the eval report, and none of them searched the open tasks for the cells they were
filing. Each duplicate cost a review round, and a duplicate that gets merged splits one fix across two tasks.

## Acceptance criteria
- [x] `scripts/tasks.py validate` computes the tasks the branch adds: task files in the working tree that are absent
      on the merge base of origin/main and HEAD. When one of those tasks shares cells with another open task and
      neither file names the other's id, validate fails and names both tasks, the cell count and a sample. With no
      origin/main ref, validate prints a warning and skips the check.
- [x] Cells are read from cell-list lines (`- <suite>/<case>[.case.json|.xlsx] (N): S!A1, A2-A5`). They are also read
      from inline citations (`gen-m9/<case> Calc!A200`, `` `probes/x.case.json` P!A12 ``, `P!A12-A15 and A22`). A
      sheet carries over to the cells after it, and `Calc` counts as the default sheet.
- [x] Run on the historical heads, the check flags T-2915 against T-2928 and T-2859 against T-2724. It is silent on
      the current heads of #2338 and #2379, after their duplicates were dropped.
- [x] Unit tests in scripts/test_tasks.py (`CellOverlaps`). docs/process/task-format.md documents the rule.
- [x] `make hygiene` green; reviewers approve.

## Out of scope
- Overlaps between tasks already on main: only added tasks are checked, so existing recorded splits stay valid.
- Judging whether a split is real. Naming the other task is accepted as a recorded split, and the reviewers judge it.

## Outcome
The check runs in `validate`. It is not part of `stats`, which stays git-free apart from `ls-files`.

Run against the old branch heads:
- It catches T-2915 vs T-2928 (7 cells), and T-2859 vs T-2724 (50 cells) plus three more #2005 overlaps. Those are
  now resolved on that branch.
- It does not catch T-3159 or T-3160: each named its owners in passing ("may share", "neighbours"), and the check
  accepts any mention as a recorded split.
- It catches T-3194 only on the PR's merge commit, which CI checks out, because the branch was cut before T-3217
  merged.

A mention that hands a cell away ("A372 to T-0967") still counts as a listed cell. Naming the other task, which that
note already does, clears the error.
