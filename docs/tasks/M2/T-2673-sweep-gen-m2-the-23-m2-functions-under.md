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
- [x] In `make eval`'s "Fifteen-digit parity checkpoint", every function listed above has at least 200 generated
      outer-call cells and shows as covered.
- [x] Every new cell is Excel-backed (excel-graph sidecars). The fill reports 0 failures, or the Outcome explains each
      refused case. The suite stays gated.
- [x] `make eval-baseline` reports 0 regressions, and the decision 0088 lock moves only by new coverage.
- [x] The gap-triager traces every new gen-m2 known failure to its innermost diverging sub-expression, as T-2648 did.
      Each family gets a task, new or extended (`campaign: visible` if it is visibly off), with milestone M2 when the
      diverging function is an M2 function. The Outcome lists each one with its cell count.
- [x] Quirks found are recorded in docs/excel-semantics/ in this PR.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing any function: the triaged tasks do that.
- New edge pools (note gaps in the Outcome).
- The other 14 uncovered functions (T-2674 M6, T-2675 M7, T-2676 M9) and the exempt ones (NOW, TODAY).

## Visible-parity stage
The coverage half of decision 0094's checkpoint for M2. The task is done when the 23 functions meet the 200
outer-call-cell bar and every new visibly-off family has a task.

## Outcome
gen-m2 grew from 30 to 39 cases and from 12,000 to 15,600 scored cells (9 cases x 400), all excel-graph-backed (fill:
9 sidecars, 0 failures, 0 skipped, no case refused). Generator command:
`cd tools && uv run rustsheet-gen --milestone M2 --focus --functions <the 23> --count 9 --formulas 400 --seed 2673`,
which raised each of the 23 functions by 154-159 generated outer-call cells, so no top-up batch was needed. Then
`uv run rustsheet-oracle fill -b excel-graph --suite gen-m2` and `make eval-baseline`.

After the sweep every one of the 23 functions is covered in the "Fifteen-digit parity checkpoint" (minimum: YEAR, 217
generated outer-call cells; 492 of 506 in-scope functions covered). All 3,600 new cells are bit-exact: gen-m2 known
failures 0 -> 0 (0 regressions, 0 new-coverage failures), so there is no new mismatch family to triage and no task was
filed. Visible parity: 0 visibly-off cells in the new cases; lock 6,039 unchanged. No quirks found.

Pool gap (noted, not filled): the M2 criteria pool for COUNTIF/COUNTIFS/SUMIF/SUMIFS has wildcards (`*c`, `?b?`,
`a*`), comparison operators (`<>1`, `<5`, `>=2`, `>0`, `=a`, `<>`), `""` and numeric literal criteria, but no
text-number criteria such as `"1"` or `"2.5"`. Filed T-2717.

Coverage-quality gap (found in review): in the 9 new cases Excel returns an error for most outer-call cells of SUMIFS
(154/156, all `#VALUE!` from criteria_range/sum_range shape mismatches), COUNTIFS (102/154, all `#VALUE!`) and MATCH
(136/157: 112 `#N/A`, mostly from a 2-D lookup_array). COUNTIF and SUMIF have none. These three count as covered, but
generated evidence for their matching itself is thin (about 2 SUMIFS, 52 COUNTIFS and 21 MATCH non-error cells). The
generator picks each range argument's shape independently and does not treat MATCH's lookup_array as a vector. Filed
T-2719 (shared *IFS shapes, a 1-D MATCH lookup_array, and a top-up batch).
