---
id: T-2673
title: "Sweep gen-m2: the 23 M2 functions under 200 generated outer-call cells"
milestone: M2
lane: research
status: done
priority: P2
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
Each of the 23 M2 functions below has at least 200 Excel-backed generated cells with the function as the outer call
(decision 0094 checkpoint item 1). They already have more than 400 Excel cells each, so only the generated half of the
coverage rule is missing. T-2648's Outcome left 37 uncovered in-scope functions "outside this task's list" and filed
no follow-up; this task and T-2674, T-2675 and T-2676 cover them by milestone.

## Context
- Source: "Fifteen-digit parity checkpoint" in eval/runs/latest/report.md (`report.json` key `v15_checkpoint`, rows
  with `covered: false`), gap triage 2026-10-25 on origin/main. Function (generated outer-call cells now), 23 in all:
  ABS (78), AND (138), CONCATENATE (99), COUNTIF (163), COUNTIFS (157), DAY (84), LEFT (104), LEN (126), LOWER (69),
  MATCH (181), MID (84), MONTH (93), NOT (101), OR (135), RIGHT (118), ROUND (169), ROUNDDOWN (80), ROUNDUP (72),
  SUMIF (184), SUMIFS (102), TRIM (108), UPPER (64), YEAR (62).
- Together they have 516 visibly-off cells in the report (ROUND 72, COUNTIF 52, YEAR 50, MATCH 48, SUMIF 47, ...).
  Nearly all of them come from gen-m9 and probes formulas where the function is not the outer call, so they are not
  this task's target. They are why outer-call cells are worth having.
- gen-m2 today: 30 cases, 12,000 scored cells, all bit-exact (no F lines in eval/baseline/gen-m2.txt).
- Template: T-2648 (its Outcome has the exact commands). Generate with
  `cd tools && uv run rustsheet-gen --milestone M2 --focus --functions <the 23> --count <n> --formulas 400 --seed 2673`
  so that each function is the outer call of at least 140 new formulas, with a top-up batch for any function still
  short. Then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m2` and `make eval-baseline`.
- Criteria functions (COUNTIF, COUNTIFS, SUMIF, SUMIFS) and MATCH read criteria and lookup pools; check their pools
  cover wildcards, comparison operators and text-number criteria (the generator's M2 pools). Note any missing pool in
  the Outcome instead of making up values.

## Acceptance criteria
- [ ] In `make eval`'s "Fifteen-digit parity checkpoint", every function listed above has at least 200 generated
      outer-call cells and shows as covered.
- [ ] Every new cell is Excel-backed (excel-graph sidecars). The fill reports 0 failures, or the Outcome explains each
      refused case. The suite stays gated.
- [ ] `make eval-baseline` reports 0 regressions, and the decision 0088 lock moves only by new coverage.
- [ ] The gap-triager traces every new gen-m2 known failure to its innermost diverging sub-expression, as T-2648 did.
      Each family gets a task, new or extended (`campaign: visible` if it is visibly off), with milestone M2 when the
      diverging function is an M2 function. The Outcome lists each one with its cell count.
- [ ] Quirks found are recorded in docs/excel-semantics/ in this PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any function: the triaged tasks do that.
- New edge pools (note gaps in the Outcome).
- The other 14 uncovered functions (T-2674 M6, T-2675 M7, T-2676 M9) and the exempt ones (NOW, TODAY).

## Visible-parity stage
The coverage half of decision 0094's checkpoint for M2. The task is done when the 23 functions meet the 200
outer-call-cell bar and every new visibly-off family has a task.

## Outcome
gen-m2 grew from 30 to 39 cases and from 12,000 to 15,600 scored cells (9 cases x 400), all excel-graph-backed (fill: 9
sidecars, 0 failures, 0 skipped, no case refused). Command:
`cd tools && uv run rustsheet-gen --milestone M2 --focus --functions <the 23> --count 9 --formulas 400 --seed 2673`
(each function the outer call of 155-157 new formulas, so no top-up batch), then
`uv run rustsheet-oracle fill -b excel-graph --suite gen-m2` and `make eval-baseline`.

All 23 functions are now covered in the "Fifteen-digit parity checkpoint" (generated outer-call cells: minimum YEAR 217,
then UPPER 223, LOWER 226, ROUNDUP 230; maximum SUMIF 341). 14 in-scope functions remain uncovered, all owned by T-2674,
T-2675 and T-2676. gen-m2 stays 100% bit-exact: known failures 0 -> 0 (0 new-coverage, 0 regressions); the visible-parity
lock 6,039 is unchanged. With no new gen-m2 known failures there is nothing for the gap-triager to trace and no family task
to file.

Pools: the 782 COUNTIF/COUNTIFS/SUMIF/SUMIFS/MATCH formulas use wildcard criteria (176), comparison operators (345) and
text-number criteria (165), so no criteria-pool gap was found. No quirks surfaced, so docs/excel-semantics/ is unchanged.
