---
id: T-0038
title: "make new-decision allocates decision numbers"
milestone: M0
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Two PRs written in parallel never take the same decision number, so no duplicate number reaches
`main`.

## Context
Agents number decision records by hand ("highest on main + 1"). On 2026-10-04, T-0032 (#65,
20:00) and T-0033 (#66, 20:08) both took 0021; T-0036 renumbered one to 0024 and added
`scripts/check_decisions.py` (decision 0023). That check is detection only: CI tests each PR merged
with the main of its last push, so two PRs that are both green against the same old main can both
merge. The duplicate then fails `make hygiene` on main and on every later PR until someone
renumbers. Decisions are now frequent (12 added on 2026-10-04), because every protected-path change
needs one.

Task ids had the same problem; `make new-task` solved it (decision 0016) with
`scripts/tasks.py::git_task_ids`, which fetches origin and reads `docs/tasks/` on every
`origin/*` branch. Reuse that pattern: generalize the helper to take the directory and filename
pattern, or share its branch-listing part, rather than copying it (CLAUDE.md directive 3;
`scripts/check_duplication.py`).

## Acceptance criteria
- [ ] `make new-decision TITLE="..."` writes `docs/decisions/NNNN-<slug>.md` from a template
  (Status / Context / Decision / Consequences, as in 0023). NNNN is one above the highest number
  found on the working tree and on every `origin/*` branch, read after a fetch. Without git or a
  network it warns and uses the working tree, as `make new-task` does.
- [ ] Unit tests in `scripts/` cover: a number taken only on a pushed branch is skipped; the
  working-tree fallback works; the slug matches the `check_decisions.py` name pattern.
- [ ] One definition: the branch scan is shared with `git_task_ids` (no copied loop), and the
  name regex is shared with `scripts/check_decisions.py`.
- [ ] The work-loop skill, `docs/process/agent-workflow.md` and `docs/decisions/README.md` say
  to create records with `make new-decision` and to push the record with the first commit on the
  branch, so other sessions see the number. A decision record notes the change.
- [ ] `make check` green; reviewers approve.

Note (decision 0028): allocation alone cannot prevent duplicates, because a session can allocate
before it pushes. The guarantee is the merge-time up-to-date check, and this task only makes
collisions rarer. Share the allocator with `make new-task` so task ids and decision numbers follow
one rule.

## Out of scope
- A merge queue or other branch-protection changes (owner settings).
- Renumbering existing records.
