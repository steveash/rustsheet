---
id: T-1326
title: "PIVOTBY on the grouping core: grid, totals, sort, filter, headers (SPEC-0006 step 5)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1322]
spec: docs/specs/SPEC-0006-groupby-pivotby.md
---

## Goal
PIVOTBY computes the probed grid, totals, sort orders, filter and headers on the shared grouping core.

## Context
SPEC-0006 PIVOTBY section; anchors P!A1, P!M1, P!Y1, P!A25, P!M25, P!M73, P!Y73, P!A97, P!M97, P!M121 (P!Y97 pending T-1328). No-record cells are `""`; column subtotals have `""` in the second header row.

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
PIVOTBY cells, `#NAME?` today: gen-m7/bcad709c30cfdf64 A13 (`PRODUCT(..,PIVOTBY(..,LAMBDA(v,Data!E6)))/CEILING(..)`, Excel `#VALUE!`) and bf889c3917eb54c5 A12 (`PIVOTBY(..,_xleta.COUNT)<=SUM(..)`, Excel `#SPILL!`, also needs eta names).

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/696af2c8cee2f57e Calc!A13 `=_xlfn.PIVOTBY(Data!B1:E12,Data!D1:D12,Data!A1:B12,LAMBDA(w,w))`: Excel `#SPILL!`, engine `#NAME?`.
- gen-m9/7db7ae6cbab32f22 Calc!A38 `=_xlfn.PIVOTBY(Data!D1:E12,Data!C1:E12,Data!C1:D12,LAMBDA(v,MAX(v,MODE(v,..))),INDIRECT("Data!$C$5",..))`: Excel `#VALUE!`, engine `#NAME?`.
- gen-m9/b5352ebdb9c5f1a1 Calc!A30 `=(LOWER(_xlfn.PIVOTBY(Data!B1:D12,Data!A1:D12,Data!A1:C12,_xleta.MIN,Data!D7,1000000000000000.0))&..)`: Excel `#VALUE!`, engine `#NAME?` (also needs T-1273 for `_xleta.MIN`).

From T-1322 (groupby.rs): build PIVOTBY on `Grouping::new` (it takes the output-size `limit`) and `sort_by_keys`. Share
the "argument must be a LAMBDA" check with `bycol_byrow::by_slices_lambda` instead of writing a third copy (GROUPBY's
function check in `Model::groupby` is the second). static_analysis.rs is at 2146 of its 2147-line ratchet: move the
`args_signature_*` helpers into a `static_analysis/` module before adding the PIVOTBY arms. Open question: should
`sort_key_cmp` itself compare numbers exactly and `cell_key` normalise -0 (GROUPBY's `key_cmp`/`group_key` do it locally)?
That changes SORT/UNIQUE, so it needs their evidence first.

## Acceptance criteria
- [ ] Those anchors pass (LAMBDA forms until T-1325) and leave the baseline; probes/m7-argument-modes F35-H35 pass once eta names exist.
- [ ] A PIVOTBY cross-product of two high-cardinality fields is size-checked before allocation (SPEC-0006 Design step 6).
- [ ] gen-m9/696af2c8cee2f57e A13, 7db7ae6cbab32f22 A38 and b5352ebdb9c5f1a1 A30 pass (or move to the SPEC-0006 step that covers them), with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
