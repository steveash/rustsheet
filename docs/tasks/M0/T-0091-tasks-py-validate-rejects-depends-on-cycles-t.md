---
id: T-0091
title: "tasks.py validate rejects depends_on cycles (T-0963 and T-0965 depend on each other, so neither is ever ready)"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`scripts/tasks.py validate` (and so `make check` and CI) fails on a cycle in `depends_on`, naming the tasks of the cycle,
and main has no cycle: today T-0963 and T-0965 depend on each other, so `ready` can never return either of them.

## Context
Found by the gap triager, 2026-10-28 (main 81569726). docs/tasks/M9/T-0963-isref-of-let-lambda-variables-bound-to-a.md
(`tier: opus`, functions-logic) has `depends_on: [T-0965]`, and docs/tasks/M9/T-0965-add-upstream-isref-xlsx-to-calc-tests-once.md
(`tier: sonnet`) has `depends_on: [T-0223, T-0963, T-0964]`. `ready()` requires every dependency to be `done`, so both wait
forever, yet `python3 scripts/tasks.py validate` reports "2410 tasks, 0 errors": `validate` (scripts/tasks.py, the loop after
the per-task checks) only checks that each dependency id exists. T-0963's Context says T-0223 already landed the code and
what is left is confirming ironcalc-calc/INFORMATION/ISREF.xlsx Sheet1!A19, A20, A23, A24, A26 once T-0965 adds the
workbook, so the intended order is T-0965 (which adds the workbook) depending on nothing open, and T-0963 after it, or
the two merged. Pick the order that matches their acceptance criteria and record it in both task files.
scripts/ is a protected path (scripts/check_governance.py), so the change needs a decision record (`make new-decision`)
or a citation of an existing one that covers task-tool checks (agent-workflow.md step 6).

## Acceptance criteria
- [ ] `validate` reports each `depends_on` cycle among open tasks once, e.g. `T-0963 -> T-0965 -> T-0963: depends_on cycle`;
      a cycle through done or dropped tasks is reported too (it is still malformed), and a self-dependency counts as a cycle.
- [ ] scripts/test_tasks.py has tests for a two-task cycle, a three-task cycle, a self-dependency and an acyclic diamond
      (no error).
- [ ] The T-0963/T-0965 cycle is broken in the task files (one `depends_on` edited, with a one-line reason in that task's
      Context), and `python3 scripts/tasks.py validate` on main reports 0 errors.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Working T-0963 or T-0965 themselves.
