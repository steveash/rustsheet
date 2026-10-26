---
id: T-2676
title: "Sweep gen-m9 round 2: ACCRINT, CONVERT, GEOMEAN, VAR.P and VAR.S to 200 generated outer-call cells"
milestone: M9
lane: research
status: in-progress
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
- [ ] In `make eval`'s "Fifteen-digit parity checkpoint", the 5 functions have at least 200 generated outer-call cells
      each and show as covered.
- [ ] Every new cell is Excel-backed (excel-graph sidecars). The fill reports 0 failures, or the Outcome explains each
      refused case. The suite stays gated.
- [ ] `make eval-baseline` reports 0 regressions, and the decision 0088 lock moves only by new coverage.
- [ ] The gap-triager traces every new gen-m9 known failure to its innermost diverging sub-expression. Each family
      gets a task, new or extended (`campaign: visible` if it is visibly off; `defer: phase-1b` if every cell is
      within 15 digits). The Outcome lists each one with its cell count.
- [ ] Quirks found are recorded in docs/excel-semantics/ in this PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any function: the triaged tasks do that.
- New edge pools (note gaps in the Outcome).
- The other uncovered functions (T-2673 M2, T-2674 M6, T-2675 M7).

## Visible-parity stage
The coverage half of decision 0094's checkpoint for M9's last 5 uncovered functions. The task is done when they meet
the 200 outer-call-cell bar and every new visibly-off family has a task.
