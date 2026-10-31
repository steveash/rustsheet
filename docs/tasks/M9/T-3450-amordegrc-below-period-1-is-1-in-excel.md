---
id: T-3450
title: "AMORDEGRC below period 1 is 1 in Excel at gen-m9/d912671baa1364aa A76 although no AMORDEGRC or AMORLINC in the workbook leaves 1"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
Where the left-over value of gen-m9/d912671baa1364aa A76 comes from is explained with probe evidence, and the engine
matches it, or the cell is recorded as unreachable under a decision.

## Context
Gap triage 2026-10-31e (`make eval` on main 8a14b8b). `scripts/exit_owners.py --unattributed` lists this cell with no
open owner.
- gen-m9/d912671baa1364aa.case.json (1): Calc!A76
- A76 `AMORDEGRC(98882,37287,38647,SUMPRODUCT((1e-15*Data!A10)),TIMEVALUE("1:30 PM"),0.1,1)`: the period is
  0.5625, so by financial.md "AMORDEGRC below period 1" the result is the value the last AMORDEGRC or AMORLINC
  evaluated before it left behind, 0 when none did. Excel 1, ours 0 (`NumericMismatch`).
- The same formula alone in a fresh workbook is 0 in Excel (probes/amordegrc-fraction-alone-d,
  `AMORDEGRC(98882,37287,38647,1E-13,0.5625,0.1,1)`).
- Every other AMOR call in the case is an error or a charge that does not round to 1, so none would leave 1 under
  the documented rule: A27 `AMORDEGRC(..,9,10,4)` `#NUM!`, A47 AMORLINC `#NUM!`, A96 `AMORLINC(92598,36663,39379,343,6,1E-300,3)`
  9.2598E-296 (rounds to 0), and A125, A145, A174, A194, A223, A243, A272, A292, A321, A341, A370, A390 are all errors.
  So either a call that leaves a value without the rule saying so (an error past some check, a period-0 call, a
  rounding of the AMORLINC charge other than to the nearest whole number), the calculation order, or state carried
  from outside the workbook (another workbook recalculated in the same oracle session) explains the 1.
- Related: T-3393 (left-over state around dynamic-array formulas, probes -ag, -ah), T-1895, T-2844. Rule and the
  probe family: financial.md "AMORDEGRC below period 1"; code: functions/financial/amortization.rs (`amor_left_over`).

## Acceptance criteria
- [ ] Probes (excel-graph, milestone M9) rebuild the case's AMOR calls in a fresh workbook, then each candidate above
      alone in front of a fraction, and tell which explanation holds; their cells are listed here before any engine
      change.
- [ ] If a rule explains it: financial.md's entry states it citing the probe cells, the engine follows it, and the A76
      line leaves `eval/baseline/gen-m9.txt` with no new known-failure line in any baseline.
- [ ] If the 1 comes from outside the workbook: a decision record (owner ruling) says how such cells are treated,
      and financial.md says why the cell cannot be reproduced.
- [ ] `make check` green; reviewers approve.

## Out of scope
- probes/amordegrc-fraction-state-ag and -ah (T-3393).

## Visible-parity stage
Decisions 0083 and 0114 (`semantics`): done when A76 has Excel's value, or the decision record above rules it
unreachable.
