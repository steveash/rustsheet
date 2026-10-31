---
id: T-3420
title: "AMORDEGRC left-over value: explain why gen-m9 d912671baa1364aa A76 (period 0.5625) is 1 in Excel, 0 ours"
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
gen-m9 d912671baa1364aa Calc!A76 `=AMORDEGRC(98882,37287,38647,SUMPRODUCT((1e-15*Data!A10)),TIMEVALUE("1:30 PM"),0.1,1)`
gives Excel's 1. Excel's rule for the value a fractional-period AMORDEGRC returns is extended, from new probes, to
cover this cell.

## Context
- Gap triage 2026-10-31c (`make eval` on b1bdc1c2d, `make unowned-off`): the only visibly-off cell attributed to no
  function with no open owner apart from COMPLEXs K25 (T-3422). It is an `F` line of eval/baseline/gen-m9.txt.
- Cell (excel-graph fill of 2026-10-28, `make eval-show CASE=gen-m9/d912671baa1364aa.case.json`):
  - gen-m9/d912671baa1364aa.case.json (1): Calc!A76, Excel 1, ours 0. Salvage is `1E-15*Data!A10` = 1E-13 (Data!A10
    is 100), period `TIMEVALUE("1:30 PM")` = 0.5625, rate 0.1, basis 1.
- What is known (docs/excel-semantics/financial.md "AMORDEGRC below period 1", decision 0131): below period 1
  AMORDEGRC returns the value the last AMORDEGRC or AMORLINC Excel evaluated before it left (calculation-order
  state, `OrderState::amor_left_over`, set and read by `Model::amortization_call` in
  crates/rustsheet-engine/src/functions/financial/amortization.rs; the order is `excel_order.rs`, SPEC-0017). Every
  other probed cell fits that table, but A76 does not: the AMORDEGRC and AMORLINC calls in its workbook are errors
  (A27, A47, A125, A145, A174, A194, A223, A243, A272, A292, A321, A341, A370, A390) except A96
  `AMORLINC(92598,36663,39379,343,6,1E-300,3)` (Excel 9.2598E-296), which leaves 0 (fraction-state probe -r).
  So the 1 comes from somewhere the table does not name. T-2825 and T-2844 both left it UNEXPLAINED.
- Candidate readings to tell apart with probes (none is evidence yet):
  1. A76 alone (a fresh workbook with Data!A10 = 100, or the literal salvage 1E-13) is already 1: then the value is not
     a left-over at all, but a property of these arguments (the tiny salvage, the period from TIMEVALUE, basis 1).
  2. A76 alone is 0 as the table predicts: then some other cell of the workbook (another sheet, a lifted or spilling
     formula, a function not yet known to touch the state) sets it. Bisect the workbook (remove halves of Calc, then
     the Data, Db and Pivot sheets) with probes that keep A76's formula and position.
  3. The value depends on the type of the period argument (a TIMEVALUE result against the literal 0.5625).
- Use the `excel-probe` skill; probes follow the eval/suites/probes/cases/amordegrc-fraction-state-* naming with
  the M9 tag.
- Related, not this task: T-3393 (left-over state around dynamic-array formulas, probes -ag and -ah).

## Acceptance criteria
- [ ] New excel-graph probes (listed with their values in this file and in financial.md) decide which reading holds,
      or name the one still open.
- [ ] financial.md "AMORDEGRC below period 1" states the rule for A76 from that evidence, replacing "UNEXPLAINED"
      (or keeps it with what the probes ruled out).
- [ ] gen-m9/d912671baa1364aa Calc!A76 passes: its known-failure line disappears from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure line in any baseline, or the Outcome says why the rule cannot
      be reproduced yet and files the follow-up.
- [ ] The new probes pass, or are known failures named in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The fraction-state probes -ag and -ah (T-3393).
- AMORLINC's own left-over value, except where a probe here needs it.

## Visible-parity stage
Decision 0083 and 0114 (`semantics`): done when Calc!A76 has Excel's value, or the rule is recorded and the
time-boxed miss is named in the Outcome with a follow-up.
