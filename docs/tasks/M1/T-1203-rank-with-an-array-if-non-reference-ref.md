---
id: T-1203
title: "RANK with an array (IF/non-reference) ref argument is #VALUE! spilled over the array"
milestone: M1
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`RANK(5,IF(A1:C1>0,A1:C1))` (an array, not a reference, as the ref argument) matches Excel: `#VALUE!`, spilled over the array's shape.

## Context
From T-1152's probe probes/multiarea-order-stats: Sheet1!H14:J14 are `#VALUE!` in Excel (the result spills 1x3 like the IF array); the engine returns 2 in H14. Semantics: docs/excel-semantics/references.md "Order statistics over unions". Probe RANK.AVG and RANK with other non-reference second arguments (constant arrays, INDEX results, CHOOSE) before fixing.

## Acceptance criteria
- [ ] probes/multiarea-order-stats H14:J14 pass, and a probe covers the other non-reference second arguments of RANK and RANK.AVG.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff
Done on this branch: probes rank-nonref-06..08 and 12..14 (oracle-filled; literal-argument forms are refused by excel-graph, recorded in statistics.md "RANK of a non-reference array"). `numbers_from_rank_ref_arg` (chisq.rs) makes the RANK family return a `#VALUE!` array of the argument's shape when the ref argument is an array; LARGE/SMALL are unchanged. Result: the first cell of H14/F1 is now `#VALUE!` (was 2) but G1:H1 / I14:J14 stay empty: 7 cells of the new probes fail in eval/baseline/probes.txt. Failed: the formula `=RANK(5,IF(..))` is a plain `CellFormula`, so the returned `CalcResult::Array` is not spilled (spill happens only for dynamic anchors, see `spill_dynamic_array` in model.rs ~1172). Next: find where a formula is classified as dynamic/array-producing (worksheet.rs `set_cell_with_dynamic_formula`, model.rs ~1121 `original_range`) and make RANK with an array ref argument (or any function returning an Array from a plain formula) spill; then re-run `make eval-baseline` and expect the 7 cells to pass.
