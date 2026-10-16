---
id: T-1324
title: "GROUPBY subtotals (total_depth 2/-2) and field_relationship (SPEC-0006 step 3)"
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
- [ ] G!J49, G!S49, G!AB49, G!A73, G!S169 pass and leave the baseline; field_relationship 1 follows T-1328's evidence or stays a recorded known failure.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
