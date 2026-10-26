---
id: T-2675
title: "Sweep gen-m7 round 2: the 8 M7 functions under 200 generated outer-call cells (LAMBDA, LET, GROUPBY, PIVOTBY, HSTACK, SEQUENCE, REGEXTEST, REGEXREPLACE)"
milestone: M7
lane: research
status: in-progress
priority: P2
size: M
tier: opus
campaign: visible
depends_on: [T-2677]
---

## Goal
Each of the 8 M7 functions below has at least 200 Excel-backed generated cells with the function as the outer call
(decision 0094 checkpoint item 1). They have more than 400 Excel cells each, so T-2254 (functions under 400 cells)
left them out.

## Context
- Source: "Fifteen-digit parity checkpoint" (`report.json` `v15_checkpoint`, `covered: false`), gap triage
  2026-10-25 on origin/main. Function (generated outer-call cells now; visibly-off cells in the report): GROUPBY (79;
  28), HSTACK (85; 28), LAMBDA (0; 124), LET (99; 0), PIVOTBY (75; 30), REGEXREPLACE (84; 0), REGEXTEST (82; 3),
  SEQUENCE (83; 25). The visibly-off cells mostly come from formulas where the function is not the outer call.
- LAMBDA's 0 is a counting gap, not a generator gap: the generator writes `LAMBDA(..)(args)` (catalog `invoke: true`)
  and the harness does not count it as a LAMBDA outer call. T-2677 fixes the count, hence the dependency.
- gen-m7 today: 52 cases, 31,845 scored cells, all bit-exact (no F lines in eval/baseline/gen-m7.txt).
- Template: T-2648 / T-2254 (their Outcomes have the commands). Generate with
  `cd tools && uv run rustsheet-gen --milestone M7 --focus --spill --functions <the 8> --count <n> --formulas 400 --seed 2675`
  (`--spill` because GROUPBY, PIVOTBY, HSTACK and SEQUENCE spill), with a top-up batch for any function still short.
  Then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m7` and `make eval-baseline`.
- T-2286's M6/M7 edge pools (LAMBDA bodies, GROUPBY/PIVOTBY options) apply. Open GROUPBY/PIVOTBY families that new
  failures may join: T-2460 (a blank key stays blank), T-2606, T-2451. REGEX families: T-1972, T-2356.

## Acceptance criteria
- [ ] In `make eval`'s "Fifteen-digit parity checkpoint", every function listed above has at least 200 generated
      outer-call cells and shows as covered.
- [ ] Every new cell is Excel-backed (excel-graph sidecars). The fill reports 0 failures, or the Outcome explains each
      refused case. The suite stays gated.
- [ ] `make eval-baseline` reports 0 regressions, and the decision 0088 lock moves only by new coverage.
- [ ] The gap-triager traces every new gen-m7 known failure to its innermost diverging sub-expression. Each family
      gets a task, new or extended (`campaign: visible` if it is visibly off), with milestone M7 when the diverging
      function is an M7 function. The Outcome lists each one with its cell count.
- [ ] Quirks found are recorded in docs/excel-semantics/ in this PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any function: the triaged tasks do that.
- New edge pools (note gaps in the Outcome).
- The other uncovered functions (T-2673 M2, T-2674 M6, T-2676 M9) and RANDARRAY (exempt, volatile).

## Visible-parity stage
The coverage half of decision 0094's checkpoint for M7. The task is done when the 8 functions meet the 200
outer-call-cell bar and every new visibly-off family has a task.
