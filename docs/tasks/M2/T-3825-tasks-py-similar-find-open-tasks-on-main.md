---
id: T-3825
title: "tasks.py similar: find open tasks on main and pushed branches that a new task would duplicate, run by new-task and the planner agents"
milestone: M2
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Before a planner or gap-triager files a task, one command lists the open tasks that look like the same work. It
covers tasks on main and on every pushed `origin/*` branch, including unmerged task-only planning branches.
`make new-task` prints these matches when it writes a file. A planner run then sees that another run already filed the
work, and does not file it a second time.

## Context
- Observed 2026-11-02. Two unmerged planning branches each filed the same Gate A classification work for the M9
  functions that have no table or exemption:
  - `agent/research/tasks-plan-opus-m2-2026-11-01` adds T-3786 ("Gate A: classify the M9 statistical functions with
    no table or exemption, ...") and T-3787 ("... the M9 financial, engineering and database functions ...").
  - `agent/planning/opus-refill-m2` adds T-3797 ("... the 51 M9 financial functions ..."), T-3798 (distribution and
    quantile), T-3799 ("... the 74 M9 sample-statistics functions ...") and T-3800 (engineering and database).
  - Both sets cover the same `missing_tables` entries of `rustsheet-eval accuracy` (eval/runs/accuracy.json). One
    of the two sets is a duplicate and needs dropping before it is worked.
- Today the only automatic duplicate check is the cell-overlap rule of `tasks.py validate` (T-3229, decision 0182).
  It sees only cell lists, and only against the branch's merge base. These tasks list no cells. The planner agent
  (.claude/agents/planner.md) says "Never duplicate an existing task", but gives no tool for it, so each run greps by
  hand, and the hand search misses tasks still on unmerged branches.
- `git_task_ids` and `branch_claims` in scripts/tasks.py already read task files on `origin/*` branches. Reuse that
  reading; do not write a second branch scanner.
- Matching has to tell real duplicates from siblings. The Gate A slices T-3669 ("... ASIN, ACOS, ATAN, ...") and
  T-3670 ("... SINH, COSH, TANH, ...") share most of their title words, but they are different work. A plain word
  overlap flags them. A measure that weights rare words (for example TF-IDF cosine over title and Goal, across all
  open tasks), plus the overlap of function names and case stems, separates the two kinds. The implementer picks the
  measure and the threshold, and the fixtures below pin them.
- `.claude/` is a protected path, so the planner and gap-triager agent edits need a decision record
  (`make new-decision`).

## Acceptance criteria
- [ ] `python3 scripts/tasks.py similar --title "<title>" [--goal "<text>"] [--no-fetch]` prints the open tasks above
      the threshold, at most 5, best first. Each line gives the id, the score, the title, and `main` or the branch it
      was found on. It exits 0 either way.
- [ ] Tests in scripts/test_tasks.py, with fixture task files copied from the titles and Goals above:
      - T-3786 flags T-3799, and T-3787 flags T-3797.
      - T-3669 does not flag T-3670.
      - A task never flags itself, and done or dropped tasks are never listed.
- [ ] `make new-task` prints `similar open tasks:` with the matches after it writes the file (it still writes it), or
      nothing when there are none.
- [ ] .claude/agents/planner.md and .claude/agents/gap-triager.md tell the agent to run `tasks.py similar` before
      each `make new-task`, and to fold the work into a match or name it in the new task.
- [ ] A decision record covers the agent-file change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Dropping one of the duplicate sets above. The planner or steward that lands those branches does that, and this
  task's PR may note which set it found.
- Making `validate` fail on similar titles. A follow-up can decide this once the false-positive rate is known.
- Duplicate task ids (the id registry, decisions 0016 and 0215).
