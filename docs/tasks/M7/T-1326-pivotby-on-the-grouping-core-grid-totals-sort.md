---
id: T-1326
title: "PIVOTBY on the grouping core: grid, totals, sort, filter, headers (SPEC-0006 step 5)"
milestone: M7
lane: functions-lookup
status: done
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
- [x] Those anchors pass (LAMBDA forms until T-1325) and leave the baseline; probes/m7-argument-modes F35-H35 pass once eta names exist.
  (P!A1, M1, Y1, A25, M25, M73, Y73, Y97, M121 pass; P!A97 and P!M97 are subtotals and moved to T-1324, P!A121 to T-1325.)
- [x] A PIVOTBY cross-product of two high-cardinality fields is size-checked before allocation (SPEC-0006 Design step 6).
- [x] gen-m9/696af2c8cee2f57e A13, 7db7ae6cbab32f22 A38 and b5352ebdb9c5f1a1 A30 pass (or move to the SPEC-0006 step that covers them), with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
PIVOTBY is `Function::Pivotby` (`_xlfn.PIVOTBY`, language tables and `language.bin` regenerated; static analysis: vectors
except field_headers and the two total depths, `StaticResult::Unknown`). functions/lookup_and_reference/pivotby.rs builds it
on two `Grouping`s of the same records (rows and columns): each `Grouping::new` gets the output-size limit (rows first with
one column group, then the columns with the real row count), and `check_array_size` runs on the whole grid before any sort,
LAMBDA call or allocation (a 20000 x 20000 cross product is refused while grouping; test `cross_product_is_size_checked_before_any_call`).
Done: the grid with `""` for empty combinations, totals 0/±1 on both axes, the two-values Total columns `""` (P!Y97),
shown and generated headers (column field names joined with ", "), key and value sorts on both axes with the probed
quirks (rows sort by value only with a Total column; column parents keep their key order), filter_array, and the argument
errors. Built on T-1323's readers, not copies: `Model::evaluate_options` (error values first), `Model::sort_order`,
`Model::filter_rows` and `Grouping::sort_by_aggregate`, which gained `parents_by_value` (false for PIVOTBY's columns).
`as_lambda` (bycol_byrow.rs) is the "is it a LAMBDA" check BYROW/BYCOL, GROUPBY and PIVOTBY share (through
`function_argument` in groupby.rs); `is_function_array`, `shown_key`, `width`, `empty_text`, `total_label_cells`,
`key_names`/`value_name`, `function_arrays_error` and `value_sort_shows_subtotals` are shared too. A value sort's aggregates
are reused for the Total column (rows, one value column) and the Total row (columns) instead of calling the function again. GROUPBY now gives `#N/IMPL!` for a
value sort over two keys with total_depth omitted (Excel shows subtotals there, pivotby-followup P!A921; T-1324).
static_analysis.rs: the `args_signature_*` helpers moved to static_analysis/signature.rs (2147 -> 1669 lines; ratchets updated by the scripts, decision 0010).
New probe probes/pivotby-followup (63 PIVOTBY questions, excel-graph 2026-10-16), recorded in docs/excel-semantics/arrays.md
"T-1326 follow-up"; P!A1321-A1601 confirm that T-1323's sort_order, error-precedence and filter rules (header-height
filter included) hold for PIVOTBY.
Eval (on current main): probes known failures 3517 -> 3269 (579 fixed, 0 regressions; +331 new-coverage failures, all
deferred cells of the new case): groupby-pivotby 330, -followup 215, -followup2 3, -followup3 27, m7-argument-modes 3
(F35:H35), catalog/availability-30 1. Non-gate: gen-m7 964/994, gen-m9 925/998 (all 8 PIVOTBY cells pass, among them
gen-m9 696af2c8cee2f57e A13, 7db7ae6cbab32f22 A38, b5352ebdb9c5f1a1 A30, gen-m7 bcad709c30cfdf64 A13, bf889c3917eb54c5 A12).
Left, `#N/IMPL!`: row/col_total_depth ±2 and a value sort over two fields with the depth omitted, which Excel shows with
subtotals (P!A97, P!M97, pivotby-followup P!A201/A901/A941, GROUPBY P!A921; noted in T-1324); function arrays (P!A121,
T-1325); PERCENTOF and relative_to (P!Y25, A49, M49, Y49, A73, pivotby-followup P!A681; T-1327).
