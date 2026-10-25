---
id: T-2648
title: "Sweep gen-m5 round 3: the 41 M5 functions under 200 generated outer-call cells"
milestone: M5
lane: research
status: in-progress
priority: P2
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
Each of the 41 M5 functions below has at least 200 Excel-backed generated cells with the function as the outer call
(decision 0094 checkpoint item 1). They already have more than 400 Excel cells each, so T-2252 (functions under 400
cells) left them out, and they are the only M5 functions besides the exempt ones that miss the coverage half of the
checkpoint.

## Context
- Source: "Fifteen-digit parity checkpoint" in eval/runs/latest/report.md (`report.json` key `v15_checkpoint`, rows
  with `covered: false`), eval of 2026-10-25 (HEAD ce765701). Function (generated outer-call cells now), 41 in all:
  ACOS (170), AGGREGATE (178), ARABIC (166), ARRAYTOTEXT (168), ASIN (172), ATAN (176), ATAN2 (170), CEILING (170),
  CEILING.MATH (169), CEILING.PRECISE (170), CELL (177), CHAR (173), COMBIN (172), ERROR.TYPE (179), FACT (185),
  FACTDOUBLE (168), FIND (175), FLOOR (171), FLOOR.MATH (167), FLOOR.PRECISE (172), GCD (173), IFNA (189), IFS (178),
  INT (175), ISNUMBER (178), ISO.CEILING (165), LCM (166), LOG (171), LOG10 (173), MOD (181), MROUND (177),
  MULTINOMIAL (172), REPLACE (175), REPT (171), ROMAN (174), SEARCH (175), SUBSTITUTE (176), SWITCH (178),
  TEXTJOIN (171), TYPE (169), UNICHAR (171).
- Together they have 1,027 visibly-off cells in the report. Most of those come from formulas where the function is
  not the outer call, so the visibly-off counts are not this task's target. They are why the new outer-call cells are
  worth having.
- Template: T-2252 (its Outcome has the exact commands). Generate with
  `cd tools && uv run rustsheet-gen --milestone M5 --focus --functions <the 41> --count <n> --formulas <m> --seed <new>`
  so that each function is the outer call of at least 60 new formulas, with a top-up batch for any function that is
  still short. Then run `uv run rustsheet-oracle fill -b excel-graph --suite gen-m5` and `make eval-baseline`.
- M5's exit is measured on the gen-m5 row (decision 0085): 38,840 scored cells, 9 known failures, an allowance of 38.
  The sweep must keep the row met (a pass rate of at least 0.999, known failures within `floor(0.001 * scored)`).
  Any new failure gets an open M5 owner, which `make exit-owners MILESTONE=M5` checks.

## Acceptance criteria
- [ ] In `make eval`'s "Fifteen-digit parity checkpoint", every function listed above has at least 200 generated
      outer-call cells and shows as covered.
- [ ] Every new cell is Excel-backed (excel-graph sidecars). The fill reports 0 failures, or the Outcome explains each
      refused case. The suite stays gated.
- [ ] `make eval-baseline` reports 0 regressions, and the decision 0088 lock moves only by new coverage. The gen-m5
      row of "Milestone exit (decision 0085)" is still met.
- [ ] The gap-triager traces every new gen-m5 known failure to its innermost diverging sub-expression, as T-2252 did.
      Each family gets a task, new or extended (`campaign: visible` if it is visibly off), with milestone M5 when the
      diverging function is an M5 function. The Outcome lists each one with its cell count, and
      `make exit-owners MILESTONE=M5` reports 0.
- [ ] Quirks found are recorded in docs/excel-semantics/ in this PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any function: the triaged tasks do that.
- New edge pools. If a parameter has no pool, note it in the Outcome instead of making up values.
- The exempt functions (INFO, RAND, RANDBETWEEN, TRUE, FALSE), and the visibly-off cells where these functions are
  not the outer call.

## Visible-parity stage
The coverage half of decision 0094's checkpoint for M5. The task is done when the 41 functions meet the 200
outer-call-cell bar and every new visibly-off family has a task.
