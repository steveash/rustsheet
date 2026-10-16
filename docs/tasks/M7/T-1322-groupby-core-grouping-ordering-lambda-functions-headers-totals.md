---
id: T-1322
title: "GROUPBY core: grouping, ordering, LAMBDA functions, headers, totals (SPEC-0006 step 1)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: M
tier: opus
depends_on: [T-1276]
spec: docs/specs/SPEC-0006-groupby-pivotby.md
---

## Goal
GROUPBY with a LAMBDA function computes the probed results for grouping, ordering, headers and grand totals.

## Context
SPEC-0006 Design steps 1-3, 5-7; evidence probes/groupby-pivotby sheet G and docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY". Build the shared `Grouping` core in functions/lookup_and_reference/groupby.rs, reusing `cell_key` and `sort_key_cmp` (spill_functions.rs; make them pub(crate), they are private today) and `Model::lambda_caller` / `call_lambda_caller` (T-1560; one caller per GROUPBY call). Cover keys of one or more columns, case folding, mixed-type order with blank last (shown as 0), source-order function inputs, total row over all values, field_headers 0-3 plus auto-detect, total_depth 0/1/-1, `""` fillers, several value columns, error values, and a LAMBDA returning an array giving `#VALUE!` per cell. Eta names wait for T-1273 / T-1325.

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
GROUPBY cells, `#NAME?` today: gen-m7/44ef4f5638e25bcd A24 (Excel `#SPILL!`), 6db767d720c5b55a A32 (`#VALUE!`: a TAKE field_headers argument) and A35 (`REDUCE(GROUPBY(..,_xleta.MIN),..)` `#SPILL!`), 93976dd9a3578ed9 A8 (`#VALUE!`, sort_order Data!E7 and filter Data!B9) and A10 (`#SPILL!`). Some need T-1323 (argument errors) or T-1273/T-1325 (eta names); score them with whichever step lands last.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/3ec66461400faf2f Calc!A17 `=(NUMBERVALUE(IFERROR(" x ",2.5),_xlfn.GROUPBY(Data!C1:D12,Data!C1:E12,LAMBDA(w,w),1,1,Data!C6:C6,..))-..)`: Excel `#VALUE!`, engine `#NAME?`.
- gen-m9/89693ab0cafc29df Calc!A9 `=_xlfn.GROUPBY(Data!C1:E12,Data!E1:E12,LAMBDA(acc,ISERROR((acc*Data!E5))))`: Excel `#SPILL!`, engine `#NAME?`.

## Acceptance criteria
- [x] The LAMBDA forms of the G-sheet anchors that need only these features pass (rewrite them as LAMBDA cases in a new probe if the eta names block scoring), and probes/m7-argument-modes F34-H34 (the LAMBDA row of F33-H35) pass; their baseline entries are removed.
- [x] An adversarial test (about 1e6 distinct keys) is refused or computed within `check_array_size` before the output is allocated (SPEC-0006 Design step 6).
- [x] Unit tests cover the grouping order (mixed types, case folding, blank last) and the `""` filler vs the 0 blank key.
- [x] gen-m9/3ec66461400faf2f A17 and 89693ab0cafc29df A9 pass (or move to the SPEC-0006 step that covers them), with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [x] `make check` green; reviewers approve (review runs with the PR).

## Out of scope
-

## Outcome
GROUPBY is `Function::Groupby` (`_xlfn.GROUPBY`, language tables and `language.bin` regenerated, static analysis:
vectors except field_headers/total_depth, `StaticResult::Unknown`). functions/lookup_and_reference/groupby.rs holds the
shared core: `Grouping`/`Group` (hash grouping on `cell_key`, -0 as 0, built once per row; `sort_by_keys` with
`sort_key_cmp` per field and a direction per field, numbers compared exactly: 0.3 before 0.1+0.2, followup G!A761), and
GROUPBY on it: one `LambdaCaller` per call (LAMBDA and `_xleta.` names alike), function inputs in source order, the total
over all values, field_headers 0-3 and the auto-detect rule, total_depth 0/±1 (±3 and ±2 with one key `#VALUE!`), `""`
fillers, several value columns, `#VALUE!` for array results, mismatched heights and bad options, and `check_array_size`
on the output before any call (1e6 distinct keys are refused; 20,000 are grouped). `cell_key`/`CellKey`/`sort_key_cmp`
are now `pub(crate)`; `LambdaCaller` is re-exported from functions/logical.
The eta forms already pass through `lambda_caller`, so the sheet G anchors score directly; probes/groupby-lambda-forms
(new, excel-graph 2026-10-16) has their LAMBDA forms plus unprobed corners, probes/groupby-blank-values (new) what the
LAMBDA gets for blank values and how `""` keys group (docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY").
Eval: probes known failures 5484 -> 4421 (1079 fixed, 0 regressions; +16 new-coverage failures in the two new cases):
groupby-pivotby 359 (sheet G 359/684 pass now), -followup 225, -followup2 98, -followup3 390, m7-argument-modes 6
(F33:H34), availability-17 1. Non-gate suites: gen-m7 956 -> 961 (44ef A24, 6db7 A32/A35, 9397 A8/A10/B10/C10),
gen-m9 920 -> 921 (89693ab0cafc29df A9); no newly failing gen cell.
Left: sort_order, filter_array (`#N/IMPL!`; T-1323, with gen-m9/3ec66461400faf2f A17 and groupby-blank-values J49),
total_depth ±2 and field_relationship (T-1324), function arrays, PERCENTOF and a one-cell reference result
(groupby-lambda-forms G!K193:K198; T-1325), PIVOTBY (T-1326, T-1327, m7-argument-modes F35:H35).
