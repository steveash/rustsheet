---
id: T-2253
title: "Sweep gen-m6: the 28 M6 functions under 400 Excel cells to at least 500, with edge pools"
milestone: M6
lane: research
status: done
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-0063, T-2286]
---

## Goal
The 28 M6 functions under 400 Excel cells (dates and lookup) reach at least 500. Each function gets enough Excel-checked cells, with edge values, that a zero
visibly-off count means something (decision 0094).

## Context
- Functions and their current Excel cells (eval of 2026-10-21, `main` 7dab5361), 28 in all:
  ADDRESS (67), AREAS (90), CHOOSECOLS (24), CHOOSEROWS (36), COLUMN (92), DATEDIF (104), DAYS (112), DAYS360 (148), EDATE (264), EOMONTH (141), GETPIVOTDATA (54), HOUR (101), HYPERLINK (30), ISOWEEKNUM (43), LOOKUP (236), MINUTE (98), NETWORKDAYS (134), NETWORKDAYS.INTL (205), ROW (188), SECOND (107), TIME (132), TRANSPOSE (52), TRIMRANGE (198), WEEKDAY (84), WEEKNUM (261), WORKDAY (159), WORKDAY.INTL (211), YEARFRAC (282).
- Template: T-2114. It swept gen-m5 to 22,400 Excel cells with `--focus` and edge pools, filled
  them through excel-graph, gated the suite, and triaged 260 new failures into tasks.
- Generate with `cd tools && uv run rustsheet-gen --milestone M6 --focus --functions <the list> --count <n>
  --formulas 400 --seed <new>`. `--functions` and `--category` come from T-0063. Pick `--count`
  so that each function is the outer call of about 250 formulas.
- Then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m6` and `make eval-baseline`.
- gen-m6 has 1,000 cells today, and GETPIVOTDATA has no generated cell (it needs a pivot source; T-1708). Exempting it is not allowed; record what blocks it.
## Acceptance criteria
- [ ] In `make eval`'s "Excel evidence per function", every function listed above has at least
      400 Excel-backed cells, and at least 200 of them are new generated formulas with the
      function as the outer call. Aim for 500. Count from T-0062's checkpoint section once it has
      landed, and from the suite's cases until then.
- [ ] Every new cell is Excel-backed (excel-graph sidecars; the fill reports 0 failures, or the
      Outcome explains each refused case), and the suite is gated.
- [ ] `make eval-baseline` records the new failures as new coverage only: 0 regressions, and the
      decision 0088 lock moves only by new coverage.
- [ ] The gap-triager traces the new failures (as T-2114 did). Every visibly-off family gets a
      `campaign: visible` task, new or extended, listed in the Outcome with its cell count.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any function: the triaged tasks do that.
- New edge pools, which go to the pool tasks. If a parameter has no pool, note it in the Outcome
  rather than inventing values ad hoc.

## Visible-parity stage
Coverage half of decision 0094's checkpoint: this task is done when its functions meet the
400-cell bar and their visibly-off cells all have tasks.

## Outcome
The coverage bar was already met when this task was picked up. T-2180 (merged after the 2026-10-21 eval this task's counts come from)
added 55 focus cases of 400 formulas (seed 2). By `make eval`'s fifteen-digit checkpoint (main 74babade), 27 of the 28 functions have
656 to 1,211 Excel-backed cells and 575 to 638 generated ones (minimum: CHOOSECOLS 656 cells, NETWORKDAYS.INTL 575 generated), all
excel-graph.
What T-2180 lacked was T-2286's lookup and array pools (this task's dependency), so this PR adds a top-up sweep of the eight listed
functions that draw from them: `rustsheet-gen --milestone M6 --focus --functions CHOOSECOLS,CHOOSEROWS,LOOKUP,TRANSPOSE,NETWORKDAYS,
NETWORKDAYS.INTL,WORKDAY,WORKDAY.INTL --count 5 --formulas 400 --seed 22`. That is 5 cases and 2,000 cells, about 250 outer calls each.
`rustsheet-oracle fill -b excel-graph --suite gen-m6` wrote 5 sidecars (0 failures, 0 refused, 0 conflicts). gen-m6 now has 85 cases and
25,000 cells, 99.78% bit-exact, and stays gated.
Eval delta: +2,000 gen-m6 cells, 8 new-coverage known failures, 0 regressions.
Triage of the 8 (all visibly off):
- T-2395 (new): a holidays array holding blanks from `HSTACK(Data!Z1:Z2,{1;2})` is accepted (6 cells).
- T-2271 (extended): a blank weekend cell and a FALSE weekend cell are `#NUM!` (2 cells).
dates.md records both.
GETPIVOTDATA stays at 54 cells with no generated cell. It needs a pivot source the generator cannot build (T-1708, SPEC-0015), so it is
not exempted and does not meet the bar here. Params with no pool among the 28: ADDRESS, AREAS, COLUMN, ROW, HYPERLINK, TIME and
TRIMRANGE (noted for the pool tasks, per Out of scope).
