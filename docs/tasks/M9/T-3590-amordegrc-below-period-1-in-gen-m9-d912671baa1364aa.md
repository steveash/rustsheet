---
id: T-3590
title: "AMORDEGRC below period 1 in gen-m9 d912671baa1364aa A76: find which cell leaves Excel's 1 (bisect the workbook with probes)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
gen-m9 d912671baa1364aa Calc!A76 gives Excel's value. If the probes cannot explain Excel's 1, financial.md records
what they ruled out, and the cell is listed there as UNEXPLAINED with the probe ids.

## Context
- Gap triage 2026-11-01 (`make eval` on origin/main 408444f20): the only gen-m9 visibly-off cell that no function
  attribution and no open task owns (`make unowned-off`). Excel 1, ours 0.
  - Formula: `=AMORDEGRC(98882,37287,38647,SUMPRODUCT((1e-15*Data!A10)),TIMEVALUE("1:30 PM"),0.1,1)`.
  - Data!A10 is 100, so the salvage is 1E-13 and the period is 0.5625.
- financial.md "AMORDEGRC below period 1" (T-1895, T-2825, T-2844) says that below period 1 Excel returns a value
  left over from another AMORDEGRC/AMORLINC call, taken in Excel's fresh-load order. The engine models that through
  `OrderState::amor_left_over` (evaluation.rs), `Model::amortization_call` (functions/financial/amortization.rs) and
  `excel_order.rs` (SPEC-0017). The same entry says A76 is UNEXPLAINED:
  - the next AMOR* call in that order, A96 `AMORLINC(92598,36663,39379,343,6,1E-300,3)`, leaves 0 (probe
    amordegrc-fraction-state-r);
  - the case's other AMORDEGRC and AMORLINC cells are errors: A27, A47, A125, A145, A194, A223, A243, A272, A321
    `#NUM!`; A174, A390 `#N/A`; A292, A341, A370 `#VALUE!`.
  - So no cell visible in the table leaves 1.
- Hypotheses to separate:
  1. A call that errors still leaves a state: an error raised after the walk started, for example in a later
     argument check.
  2. The value is not left over: some argument pattern gives 1. The salvage here is a SUMPRODUCT of a tiny product,
     while the probed ones were literals.
  3. The order reaches a cell on Pivot or Db first (4 sheets; the probes used one or two).
- Method: excel-graph probes (`excel-probe` skill) on copies of the case:
  - drop one sheet, then halves of the AMOR* cells, then the non-AMOR cells, until the 1 goes;
  - then a minimal workbook that reproduces it.
  Name each probe in the Outcome.
- Related: T-3393 (dynamic-array formulas in the order, probes -ag and -ah).

## Visible-parity stage
Decision 0083: the cell is visibly off (1 against 0). The target is Excel's value. If no rule is found, the target
is an UNEXPLAINED entry naming the probes.

## Acceptance criteria
- [ ] Probe cases (probes/, tag M9, excel-graph) that bisect the workbook, each listed in the Outcome with its
      result.
- [ ] gen-m9/d912671baa1364aa.case.json (1): Calc!A76 passes and its line leaves eval/baseline/gen-m9.txt, or
      financial.md states the rule ruled in or out and the cell stays a known failure.
- [ ] Every amordegrc-* and amor-* probe still passes. `make eval-baseline` records no regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Dynamic-array formulas in the order (T-3393).
