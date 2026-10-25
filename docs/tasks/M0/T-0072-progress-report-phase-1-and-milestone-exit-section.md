---
id: T-0072
title: "Progress report: Phase 1 and milestone exit section and campaign-stage table with task counts and off cells"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-0070, T-0071]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
The dashboard opens with progress toward the goals:
- the Phase 1 exit (decision 0114) and the fifteen-digit checkpoint (decision 0094);
- a row per milestone with its exit status and task counts;
- a table of the decision-0114 campaign stages with their open, ready, blocked and done tasks,
  their off cells, and the functions the semantics stage would lift.

## Context
- SPEC-0022, the `goals` and `campaign` rows of "Page layout", and "Alternatives considered" for
  why accuracy and visible off cells are shown together.
- Inputs:
  - report.json `v15_checkpoint` (`phase1_line`, `verdict_line`, `at_rate`);
  - `milestone_exit` rows (decision 0085, T-2177);
  - `milestone_lists` (decision 0043);
  - `probe_milestones` (decision 0051);
  - `catalog_coverage`;
  - the T-0068 `attributed` counts;
  - tasks.json (T-0069);
  - docs/milestones.md (count `- [x]` and `- [ ]` under each `## M<n>` heading's
    `**Exit criteria:**`, and the phase table).
- The assessment's plan table: semantics 1,688 off cells and 45 functions lifted, 57 tasks
  ready; accuracy about 3,000 cells and 35 functions, 15 tasks; Excel's algorithm about 2,500
  cells and 47 functions, about 32 tasks; parked 34 tasks.

## Acceptance criteria
- [ ] Module `progress_report/goals.py` writes the `goals` block:
  - the report's Phase 1 line and checkpoint line verbatim;
  - with T-0071's history, whether the previous complete run was also met;
  - one row per milestone M0 to M14: phase, exit criteria checked/total in docs/milestones.md,
    the `milestone_exit` measures (met or not, with counts) where the report has a row, the
    frozen-list and probe-milestone failures where they apply, and tasks per status.
- [ ] It writes the `campaign` block, one row each for semantics, accuracy, visible and parked
      (`defer: phase-1b`):
  - open, ready, blocked and done tasks;
  - visibly-off attributed cells of the functions below 99.9%: `visible_other` on the semantics
    row, `visible_numeric` on the accuracy and visible rows together;
  - on the semantics row, the number of below-bar functions whose `attributed` share reaches
    99.9% once their `visible_other` cells are fixed.
  Python only does arithmetic on the report's counts; the 99.9% threshold comes from the report
  (`v15_checkpoint` or the T-0068 tier), never from a Python constant.
- [ ] The milestones.md parse fails loudly on a milestone heading with no `**Exit criteria:**`
      list (test).
- [ ] Tests cover the lift count on a synthetic report, and the stage and parked rows from a
      synthetic tasks.json.
- [ ] The regenerated page is committed. The Outcome compares the campaign table with the
      assessment's plan table and explains any difference.
- [ ] `make py-check` green; `make check` green; reviewers approve.

## Out of scope
- A mechanical split of numeric off cells between the accuracy and visible stages (SPEC-0022,
  "Alternatives considered").
- Changing the milestone exit measures.
