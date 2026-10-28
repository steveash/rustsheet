---
id: T-2676
title: "Sweep gen-m9 round 2: ACCRINT, CONVERT, GEOMEAN, VAR.P and VAR.S to 200 generated outer-call cells"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
Each of the 5 M9 functions below has at least 200 Excel-backed gen-m9 cells with the function as the outer call
(decision 0094 checkpoint item 1). They have more than 400 Excel cells each, so the T-2255 to T-2258 sweeps
(functions under 400 cells) left them out, and they are the only uncovered M9 functions.

## Context
- Source: "Fifteen-digit parity checkpoint" (`report.json` `v15_checkpoint`, `covered: false`), gap triage
  2026-10-25 on origin/main. Function (Excel cells; generated outer-call cells; visibly off in the report): ACCRINT
  (1,062; 0; 29), CONVERT (926; 0; 9), GEOMEAN (660; 0; 4), VAR.P (825; 22; 13), VAR.S (914; 1; 13).
- gen-m9 today: 261 cases, 95,261 scored cells, 5,269 F lines in eval/baseline/gen-m9.txt.
- Template: T-2257 (financial) and T-2256 (statistical) Outcomes. Generate with
  `cd tools && uv run rustsheet-gen --milestone M9 --focus --functions ACCRINT,CONVERT,GEOMEAN,VAR.P,VAR.S --count <n> --formulas 400 --seed 2676`
  so that each function is the outer call of at least 220 new formulas. Then
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9` and `make eval-baseline`.
- Open families new failures may join: ACCRINT T-2562 (calendar start), T-2639; CONVERT T-2047 (last digit to a
  non-base unit, defer) and T-2046 (its sweep); GEOMEAN T-1099 and T-1284 (blocked, last ulp); VAR.P/VAR.S T-0951,
  T-1749 (accumulation order).
- Excel for the web changed 1904-workbook financial results after 2026-10-12 (T-2337). ACCRINT cells in 1904 cases
  need that task's re-fill rule, or should be left out of this sweep.

## Acceptance criteria
- [x] In `make eval`'s "Fifteen-digit parity checkpoint", the 5 functions have at least 200 generated outer-call cells
      each and show as covered.
- [x] Every new cell is Excel-backed (excel-graph sidecars). The fill reports 0 failures, or the Outcome explains each
      refused case. The suite stays gated.
- [x] `make eval-baseline` reports 0 regressions, and the decision 0088 lock moves only by new coverage.
- [x] The gap-triager traces every new gen-m9 known failure to its innermost diverging sub-expression. Each family
      gets a task, new or extended (`campaign: visible` if it is visibly off; `defer: phase-1b` if every cell is
      within 15 digits). The Outcome lists each one with its cell count.
- [x] Quirks found are recorded in docs/excel-semantics/ in this PR.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing any function: the triaged tasks do that.
- New edge pools (note gaps in the Outcome).
- The other uncovered functions (T-2673 M2, T-2674 M6, T-2675 M7).

## Visible-parity stage
The coverage half of decision 0094's checkpoint for M9's last 5 uncovered functions. The task is done when they meet
the 200 outer-call-cell bar and every new visibly-off family has a task.

## Outcome
gen-m9 grew by 3 focused cases (seed 2676, 3 x 400 formulas, 1,200 cells), all excel-graph-backed (fill: 3 sidecars,
0 failures, 0 skipped). Command: `cd tools && uv run rustsheet-gen --milestone M9 --focus --functions
ACCRINT,CONVERT,GEOMEAN,VAR.P,VAR.S --count 3 --formulas 400 --seed 2676`; no top-up batch was needed. No case uses the
1904 date system, so the T-2337 re-fill rule does not apply.

"Fifteen-digit parity checkpoint": all 5 are covered, generated outer-call cells ACCRINT 240, CONVERT 240, GEOMEAN 240,
VAR.P 262, VAR.S 241 (498 of 506 in-scope functions covered). `make eval-baseline`: gen-m9 known failures
3,897 -> 3,956 (59 new-coverage failures, 0 regressions, 0 fixed); visible parity 3,792 in-scope cells visibly off (31
in the new cases), lock 4,448 unchanged.

The gap-triager traced all 59 lines to their innermost diverging sub-expression (33 visibly off, 26 ulp-level). Campaign
stages follow docs/process/task-format.md (`semantics` for a wrong error, `accuracy` for a non-faithful result):

| Task | New/extended | Family | Cells (visible/ulp) |
|---|---|---|---|
| T-2792 | new | ACCRINT check order: calc_method first, each date range-checked as read, FALSE par `#VALUE!` | 21 (21/0) |
| T-2793 | new | CONVERT with a boolean unit cell is `#VALUE!` (ours `#N/A`) | 3 (3/0) |
| T-2794 | new | GEOMEAN in-range `exp(ln P/n)` not faithful for products far from 1 | 16 (2/14) |
| T-1284 | extended | GEOMEAN last-ulp root rounding | 12 (0/12) |
| T-2425 | extended | nested FORECAST.ETS* (`#N/A` in Excel for the web, ours `#N/IMPL!`) | 4 (4/0) |
| T-2562 | extended | ACCRINT at the calendar's start | 1 (1/0) |
| T-2765 | extended | FTEST last digits, nested as ACCRINT's rate | 1 (0/1) |
| T-2047 | extended | CONVERT to a non-base unit, last digit | 1 (0/1) |

Quirks recorded: financial.md "ACCRINT check order: calc_method first, dates as read (T-2676)" (plus FALSE par evidence
and the calendar-start bullet), engineering.md "CONVERT: a boolean unit is #VALUE! (T-2676)", numbers.md "GEOMEAN
in-range root" (products far from 1). No new edge pools were added.
