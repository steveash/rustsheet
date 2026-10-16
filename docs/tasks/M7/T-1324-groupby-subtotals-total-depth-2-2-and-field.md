---
id: T-1324
title: "GROUPBY subtotals (total_depth 2/-2) and field_relationship (SPEC-0006 step 3)"
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
GROUPBY with total_depth 2/-2 shows subtotals and "Grand Total" as Excel does, and field_relationship 0/1 follows the probe.

## Context
SPEC-0006 GROUPBY total_depth and field_relationship bullets; anchors G!J49, G!S49, G!AB49, G!A73, G!S169, G!J169 (the last pending T-1328).

From T-1322 (groupby.rs): total_depth ±2 with more than one key field returns `#N/IMPL!` from `Model::total_depth`
(`TODO(T-1324)`; ±2 with one key is already `#VALUE!`), and any given field_relationship is rejected by the `given(...)`
check in `Model::groupby`. Groups come sorted lexicographically by key prefix (`Grouping::sort_by_keys`), so the subtotal
rows go in with one linear pass.

Since T-1323: `Model::total_depth` returns ±2 and `Model::groupby` reports `#N/IMPL!` for it after every other check;
`Model::relationship` reads a scalar field_relationship (0 default, 1 `#VALUE!` with ±2 and the default layout otherwise),
and an array one (lifted per element, followup G!A861-A981) or a table sorted by a value column are `#N/IMPL!`
(`TODO(T-1324)`).

From T-1326 (pivotby.rs): PIVOTBY's row_total_depth / col_total_depth ±2 with two or more fields on that axis are
`#N/IMPL!` too (`Model::total_depth`), and so is a value sort (`SortOrder::Value`) over two or more fields with that
axis' depth omitted, because Excel then shows subtotals and "Grand Total" as with depth 2 (`TODO(T-1324)` in
`Model::pivotby_args`). Anchors: probes/groupby-pivotby P!A97 (row subtotals), P!M97 (column subtotals: the first-level key
over `""`, "Grand Total" over `""`); probes/pivotby-followup P!A201, P!A901 (rows), P!A941 (columns, parents sorted by
their aggregate). GROUPBY has the same quirk: `GROUPBY(A2:B11,D2:D11,SUM,,,3)` with total_depth omitted shows subtotals
(probes/pivotby-followup P!A921), unlike explicit 0 (followup2 G!A261). docs/excel-semantics/arrays.md "T-1326 follow-up".

## Acceptance criteria
- [x] G!J49, G!S49, G!AB49, G!A73, G!S169 pass and leave the baseline; field_relationship 1 follows T-1328's evidence or stays a recorded known failure.
- [x] `make check` green
- [ ] reviewers approve.

## Out of scope
-

## Outcome
Subtotals on the shared grouping core, which moved out of groupby.rs into functions/lookup_and_reference/grouping.rs
(`Grouping`, its sorts, plus the new `Grouping::lines`, `rows_of`, `subtotal_label_cells`, `sort_by_key_order`,
`ParentOrder`, `effective_depth`, `subtotal_levels`). `Grouping::lines` lays the sorted groups out with the subtotals of
the first |depth| - 1 key fields in one linear pass (a key-prefix change between neighbours opens or closes a subtotal):
after the groups for a positive depth (deepest first), before them for a negative one (shallowest first).
GROUPBY: `Model::groupby` was split into `groupby_inputs` (evaluation, `GroupbyInputs`), `groupby_args` (the checks,
`GroupbyArgs`) and `groupby` (layout). total_depth n is valid up to the number of keys (probed: 3 over three keys, 4 over
four, not 5); "Grand Total" with subtotals; a value sort with the depth omitted shows every level's subtotals; a table
relationship sorted by a value sorts flat (`ParentOrder::Flat`); an array field_relationship is lifted per element
(`lifted_groupby`, at most three GROUPBY runs whatever the array; `lifted_error` for an earlier error value).
PIVOTBY: row and column subtotals (`Slot::Subtotal`, `Model::grid_row` shared by group, subtotal and Total rows), labels
per axis, column parents by value with subtotals; column subtotals over several value columns stay `#N/IMPL!` (Excel's
cells there are not subtotals: follow-up T-1700).
Directive 5: the subtotal rows and columns are counted after the key sort and checked with `check_array_size` before any
LAMBDA call (`subtotal_rows_count_toward_the_output_size`).
Evidence: new probe probes/groupby-subtotals (49 questions, excel-graph); quirks in docs/excel-semantics/arrays.md
"GROUPBY and PIVOTBY" (T-1324 follow-up). field_relationship 1 with subtotals stays `#VALUE!` (T-1328's evidence).
Eval delta (probes): 1,181 known failures fixed, none regressed: groupby-pivotby 669 -> 313 (G!J49, G!S49, G!A73,
G!S169, P!A97, P!M97; the rest is function arrays, PERCENTOF and relative_to, T-1325/T-1327), groupby-pivotby-followup
150 -> 0, groupby-pivotby-followup2 365 -> 0, pivotby-followup 331 -> 21 (P!A681, relative_to, T-1327). New case
groupby-subtotals: 3,386 cells, 2,921 pass; its 465 known failures are the multi-value column subtotals (G!A781,
G!A1141, G!A1171, G!Z1201/AA1201; T-1700). Probes known failures 3,277 -> 2,561.
