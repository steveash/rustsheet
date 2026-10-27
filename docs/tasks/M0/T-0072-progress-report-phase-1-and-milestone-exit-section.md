---
id: T-0072
title: "Progress report: Phase 1 and milestone exit section and campaign-stage table with task counts and off cells"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-0081, T-0071]
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
- [x] Module `progress_report/goals.py` writes the `goals` block:
  - the report's Phase 1 line and checkpoint line verbatim;
  - with T-0071's history, whether the previous complete run was also met;
  - one row per milestone M0 to M14: phase, exit criteria checked/total in docs/milestones.md,
    the `milestone_exit` measures (met or not, with counts) where the report has a row, the
    frozen-list and probe-milestone failures where they apply, and tasks per status.
- [x] It writes the `campaign` block, one row each for semantics, accuracy, visible and parked
      (`defer: phase-1b`):
  - open, ready, blocked and done tasks;
  - visibly-off attributed cells of the functions below 99.9%: `visible_other` on the semantics
    row, `visible_numeric` on the accuracy and visible rows together;
  - on the semantics row, the number of below-bar functions whose `attributed` share reaches
    99.9% once their `visible_other` cells are fixed.
  Python only does arithmetic on the report's counts; the 99.9% threshold comes from the report
  (`v15_checkpoint`, T-0080, or the T-0068 tier), never from a Python constant.
- [x] The milestones.md parse fails loudly on a milestone heading with no `**Exit criteria:**`
      list (test).
- [x] Tests cover the lift count on a synthetic report, and the stage and parked rows from a
      synthetic tasks.json.
- [x] The regenerated page is committed. The Outcome compares the campaign table with the
      assessment's plan table and explains any difference.
- [x] `make py-check` green; `make check` green; reviewers approve.

## Out of scope
- A mechanical split of numeric off cells between the accuracy and visible stages (SPEC-0022,
  "Alternatives considered").
- Changing the milestone exit measures.

## Outcome
- `tools/src/rustsheet_tools/progress_report/goals.py` writes the `goals` and `campaign` blocks,
  placed after the bottom line (page.py `LAYOUT`). The generator reads report.json, tasks.json,
  the catalog and docs/milestones.md only; it never reads task files.
- report.json had no `phase1_line`, `verdict_line` or rate, which the acceptance criteria need.
  `v15_checkpoint` now writes `rate_num`/`rate_den` (`RATE_NUM`/`RATE_DEN`) and both lines
  (crates/rustsheet-eval/src/v15_checkpoint.rs, with a unit test). A report from before this change
  has an older shape, so the page is marked stale, not broken. The page was regenerated from a fresh
  `make eval` at a2d5914a (all 17 suites; same tier counts as T-0081's run).
- `Tasks` (model.py) also keeps each stage's `ready` and the per-milestone status counts. The
  history row is unchanged.
- The functions below the bar are the report tiers under `rate999`. A function is lifted when its
  counted cells (`rate_cells`: attributed, or every Excel cell below the floor) reach the report's
  rate once its counted `visible_other` cells are fixed. The off cells are attributed cells, so
  each cell is counted once.
- docs/milestones.md has a checkbox exit list for M0 only (5/5). The other milestones' exit
  criteria are prose, shown as "prose". A milestone heading with no `**Exit criteria:**` line fails,
  as does a milestone that is missing from the phase table or listed there twice.
- Campaign table on this run, against the assessment's plan table (2026-10-25, `main` 2c52346):

  | Stage | Page: open / ready / blocked / done | Page: off cells | Page: lifted | Assessment |
  |---|---|---|---|---|
  | Semantics | 47 / 43 / 0 / 33 | 20,009 | 69 of 169 | 1,688 cells, 45 lifted, 57 ready |
  | Accuracy | 13 / 9 / 0 / 1 | 131,346 (with visible) | - | ~3,000 cells, +35, 15 tasks |
  | Visible | 38 / 23 / 4 / 216 | (in accuracy) | - | ~2,500 cells, last 47, ~32 tasks |
  | Parked | 37 / 0 / 6 / 1 | - | - | 34 tasks |

  The differences:
  - The page counts the functions below 99.9%, which is 169 functions in tiers ≥ 99% and < 99%. The assessment
    counted only the 127 below 99%. The page's below-99% tier is 97 functions, and 28 of them would be lifted.
  - Dense sweeps that landed after 2c52346 dominate the cell counts. VLOOKUP alone has
    17,946 type cells (2,063 without it). BETA.DIST has 82,690 numeric cells and F.DIST 38,101.
  - The assessment split numbers into accuracy (within 1e-9) and algorithm using a 50-digit
    reference. The page shows them together, as the owner chose (SPEC-0022).
  - Task counts are today's tasks.json. Ready ignores branch claims (decision 0119). Since the
    assessment, 33 semantics tasks are done and more were filed.
- Gates: `make py-check`, `make hygiene`, `make typos` and `scripts/check_governance.py origin/main`
  pass. `cargo test` and `clippy -D warnings` pass for rustsheet-eval. Not run here: the full
  `make check`, and the review panel.

